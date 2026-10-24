use emocis_cclg::{LanguageAdapter, PythonAdapter};
use emocis_core::NodeKind;

#[test]
fn test_python_adapter_parsing() {
    let adapter = PythonAdapter::new();
    let code = r#"
from .db import get_session, engine as eng
from models.user import User
import os
import utils.logger as log
from helpers import *

def calculate_tax(amount):
    session = get_session()
    return amount * 0.11

def _internal_cache():
    pass

class TaxCalculator:
    pass
"#;

    let parsed = adapter.parse_file("proj_py", "pkg/services/tax.py", code).expect("Parse python");
    let file_node = parsed.nodes.iter().find(|n| n.kind == NodeKind::File).unwrap();
    let imports = file_node.metadata["imports"].as_array().expect("imports array");
    
    assert!(imports.iter().any(|i| i["module"] == ".db" && i["local"] == "get_session"));
    assert!(imports.iter().any(|i| i["module"] == ".db" && i["local"] == "eng" && i["imported"] == "engine"));
    assert!(imports.iter().any(|i| i["module"] == "models.user" && i["local"] == "User"));
    assert!(imports.iter().any(|i| i["module"] == "helpers" && i["local"] == "*"));

    let exports = file_node.metadata["exports"].as_array().expect("exports array");
    assert!(exports.iter().any(|e| e["exported"] == "calculate_tax"));
    assert!(exports.iter().any(|e| e["exported"] == "TaxCalculator"));
    assert!(!exports.iter().any(|e| e["exported"] == "_internal_cache"));
}

#[test]
fn test_python_aliases_and_submodules() {
    let adapter = PythonAdapter::new();
    let code = r#"
import utils.logger as log
import os.path as osp

def log_event(msg):
    log.info(msg)
"#;
    let parsed = adapter.parse_file("proj_py", "pkg/tracker.py", code).unwrap();
    let file_node = parsed.nodes.iter().find(|n| n.kind == NodeKind::File).unwrap();
    let imports = file_node.metadata["imports"].as_array().unwrap();
    assert!(imports.iter().any(|i| i["module"] == "utils.logger" && i["local"] == "log"));
    assert!(imports.iter().any(|i| i["module"] == "os.path" && i["local"] == "osp"));
}


