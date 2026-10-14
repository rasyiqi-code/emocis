use emocis_cclg::{LanguageAdapter, PythonAdapter};
use emocis_core::NodeKind;

#[test]
fn test_python_adapter_parsing() {
    let adapter = PythonAdapter::new();
    let py_code = r#"
class PaymentProcessor:
    def process(self, amount):
        self.validate(amount)
        return True

    def validate(self, amount):
        print(amount)
"#;

    let parsed = adapter.parse_file("proj1", "services/payment.py", py_code).expect("Should parse python code");
    
    // Check nodes
    let names: Vec<&str> = parsed.nodes.iter().map(|n| n.name.as_str()).collect();
    assert!(names.contains(&"services/payment.py"));
    assert!(names.contains(&"PaymentProcessor"));
    assert!(names.contains(&"process"));
    assert!(names.contains(&"validate"));

    // Check kinds
    let class_node = parsed.nodes.iter().find(|n| n.name == "PaymentProcessor").unwrap();
    assert_eq!(class_node.kind, NodeKind::Class);

    let method_node = parsed.nodes.iter().find(|n| n.name == "process").unwrap();
    assert_eq!(method_node.kind, NodeKind::Method);

    // Check callsites
    let callsites: Vec<&str> = parsed.nodes.iter()
        .filter(|n| n.kind == NodeKind::CallSite)
        .map(|n| n.name.as_str())
        .collect();
    assert!(callsites.contains(&"self.validate"));
    assert!(callsites.contains(&"print"));

    // Check edges
    assert!(!parsed.edges.is_empty());
}
