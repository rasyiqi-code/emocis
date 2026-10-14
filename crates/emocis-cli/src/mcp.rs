use std::io::{self, BufRead, Write};
use emocis_cclg::GraphAnalyzer;
use emocis_memory::{ContextCompressor, DriftDetector, HealthScorer, MemoryEngine};
use serde_json::{json, Value};
use crate::common::open_or_init_store;

pub fn run_mcp_server() -> anyhow::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let request: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(e) => {
                let err_res = json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": { "code": -32700, "message": format!("Parse error: {}", e) }
                });
                writeln!(stdout, "{}", serde_json::to_string(&err_res)?)?;
                stdout.flush()?;
                continue;
            }
        };

        let id = request.get("id").cloned();
        let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");

        // Handle JSON-RPC method
        let response = match method {
            "initialize" => {
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": {
                            "tools": {}
                        },
                        "serverInfo": {
                            "name": "emocis-engine",
                            "version": "0.1.0"
                        }
                    }
                })
            }
            "notifications/initialized" => {
                continue; // notification, no reply needed
            }
            "ping" => {
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {}
                })
            }
            "tools/list" => {
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "tools": [
                            {
                                "name": "emocis_search",
                                "description": "Search codebase architectural decisions, constraints, warnings, and business rules.",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {
                                        "query": { "type": "string", "description": "Search keyword or natural language question" }
                                    },
                                    "required": ["query"]
                                }
                            },
                            {
                                "name": "emocis_get_context",
                                "description": "Get compressed Minimal Context Package for a coding task/intent (saves 60%+ tokens).",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {
                                        "intent": { "type": "string", "description": "Target coding task or feature intent" }
                                    },
                                    "required": ["intent"]
                                }
                            },
                            {
                                "name": "emocis_impact",
                                "description": "Analyze impact of modifying a symbol (callers, transitive dependents, exposed APIs).",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {
                                        "symbol": { "type": "string", "description": "Function or symbol name to analyze" }
                                    },
                                    "required": ["symbol"]
                                }
                            },
                            {
                                "name": "emocis_calls",
                                "description": "Find functions and methods called by the target function.",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {
                                        "function": { "type": "string", "description": "Caller function name" }
                                    },
                                    "required": ["function"]
                                }
                            },
                            {
                                "name": "emocis_callers",
                                "description": "Find functions that call the target function.",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {
                                        "function": { "type": "string", "description": "Target function name" }
                                    },
                                    "required": ["function"]
                                }
                            },
                            {
                                "name": "emocis_audit",
                                "description": "Run drift and architecture boundary violation audit on the codebase.",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {}
                                }
                            },
                            {
                                "name": "emocis_health",
                                "description": "Get codebase modularity, coupling, cohesion, and health scores.",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {}
                                }
                            }
                        ]
                    }
                })
            }
            "tools/call" => {
                let params = request.get("params").cloned().unwrap_or(Value::Null);
                let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let args = params.get("arguments").cloned().unwrap_or(json!({}));

                let tool_output = handle_tool_call(tool_name, &args);
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [
                            {
                                "type": "text",
                                "text": tool_output
                            }
                        ]
                    }
                })
            }
            _ => {
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": { "code": -32601, "message": format!("Method not found: {}", method) }
                })
            }
        };

        writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
        stdout.flush()?;
    }

    Ok(())
}

fn handle_tool_call(name: &str, args: &Value) -> String {
    let store = match open_or_init_store() {
        Ok(s) => s,
        Err(e) => return format!("Error accessing EMOCIS store: {}", e),
    };

    match name {
        "emocis_search" => {
            let query = args.get("query").and_then(|q| q.as_str()).unwrap_or("");
            let engine = MemoryEngine::new(&store);
            match engine.search(query) {
                Ok(items) => {
                    if items.is_empty() {
                        format!("No knowledge items found matching '{}'", query)
                    } else {
                        serde_json::to_string_pretty(&items).unwrap_or_default()
                    }
                }
                Err(e) => format!("Search error: {}", e),
            }
        }
        "emocis_get_context" => {
            let intent = args.get("intent").and_then(|i| i.as_str()).unwrap_or("");
            let compressor = ContextCompressor::new(&store);
            match compressor.build_context(intent) {
                Ok(pkg) => pkg.formatted_markdown,
                Err(e) => format!("Context compression error: {}", e),
            }
        }
        "emocis_impact" => {
            let symbol = args.get("symbol").and_then(|s| s.as_str()).unwrap_or("");
            let analyzer = GraphAnalyzer::new(&store);
            match analyzer.analyze_impact(symbol) {
                Ok(report) => serde_json::to_string_pretty(&report).unwrap_or_default(),
                Err(e) => format!("Impact analysis error: {}", e),
            }
        }
        "emocis_calls" => {
            let function = args.get("function").and_then(|f| f.as_str()).unwrap_or("");
            match store.find_nodes_by_name(function) {
                Ok(candidates) => {
                    if let Some(target) = candidates.first() {
                        match store.get_callees(&target.id) {
                            Ok(callees) => {
                                let list: Vec<String> = callees.into_iter().map(|(_, n)| format!("{} ({:?})", n.name, n.kind)).collect();
                                serde_json::to_string_pretty(&list).unwrap_or_default()
                            }
                            Err(e) => format!("Query error: {}", e),
                        }
                    } else {
                        format!("Symbol '{}' not found in code graph", function)
                    }
                }
                Err(e) => format!("Find error: {}", e),
            }
        }
        "emocis_callers" => {
            let function = args.get("function").and_then(|f| f.as_str()).unwrap_or("");
            match store.find_nodes_by_name(function) {
                Ok(candidates) => {
                    if let Some(target) = candidates.first() {
                        match store.get_callers(&target.id) {
                            Ok(callers) => {
                                let list: Vec<String> = callers.into_iter().map(|(_, n)| format!("{} ({:?})", n.name, n.kind)).collect();
                                serde_json::to_string_pretty(&list).unwrap_or_default()
                            }
                            Err(e) => format!("Query error: {}", e),
                        }
                    } else {
                        format!("Symbol '{}' not found in code graph", function)
                    }
                }
                Err(e) => format!("Find error: {}", e),
            }
        }
        "emocis_audit" => {
            let detector = DriftDetector::new(&store);
            match detector.run_audit("project") {
                Ok(report) => serde_json::to_string_pretty(&report).unwrap_or_default(),
                Err(e) => format!("Audit error: {}", e),
            }
        }
        "emocis_health" => {
            let scorer = HealthScorer::new(&store);
            match scorer.compute_health("project") {
                Ok(metrics) => serde_json::to_string_pretty(&metrics).unwrap_or_default(),
                Err(e) => format!("Health error: {}", e),
            }
        }
        _ => format!("Unknown tool name: {}", name),
    }
}
