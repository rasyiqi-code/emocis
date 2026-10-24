use crate::common::{current_project_id, open_or_init_store};
use emocis_cclg::GraphAnalyzer;
use emocis_memory::{ContextCompressor, DriftDetector, HealthScorer, MemoryEngine};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

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
                                "description": "Keyword search of codebase architectural decisions, constraints, warnings, and business rules.",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {
                                        "query": { "type": "string", "description": "Search keywords" }
                                    },
                                    "required": ["query"]
                                }
                            },
                            {
                                "name": "emocis_get_context",
                                "description": "Get a bounded, keyword-matched context package for a coding task. Token savings are not measured.",
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
                                "description": "Get measured static graph counts, resolution ratios, file fan-in/fan-out, and dependency cycles. Overall quality scores remain null until calibrated.",
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

                match handle_tool_call(tool_name, &args) {
                    Ok(tool_output) => json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": { "content": [{ "type": "text", "text": tool_output }] }
                    }),
                    Err(error) => json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "content": [{ "type": "text", "text": error.to_string() }],
                            "isError": true
                        }
                    }),
                }
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

fn handle_tool_call(name: &str, args: &Value) -> anyhow::Result<String> {
    let store = open_or_init_store()?;
    let project_id = current_project_id()?;

    // Auto-index on first tool call if the project has not been mapped yet
    if store.count_nodes(&project_id).unwrap_or(0) == 0 {
        let _ = crate::common::index_directory_ext(std::path::Path::new("."), &project_id, false);
    }

    let output = match name {
        "emocis_search" => {
            let query = args.get("query").and_then(|q| q.as_str()).unwrap_or("");
            let engine = MemoryEngine::new(&store);
            serde_json::to_string_pretty(&engine.search_for_project(&project_id, query)?)?
        }
        "emocis_get_context" => {
            let intent = args.get("intent").and_then(|i| i.as_str()).unwrap_or("");
            let compressor = ContextCompressor::new(&store);
            compressor
                .build_context_for_project(&project_id, intent)?
                .formatted_markdown
        }
        "emocis_impact" => {
            let symbol = args.get("symbol").and_then(|s| s.as_str()).unwrap_or("");
            let analyzer = GraphAnalyzer::new(&store);
            serde_json::to_string_pretty(&analyzer.analyze_impact(symbol)?)?
        }
        "emocis_calls" => {
            let function = args.get("function").and_then(|f| f.as_str()).unwrap_or("");
            let candidates = store.find_nodes_by_name(function)?;
            let callees = if let Some(target) = candidates.first() {
                store.get_callees(&target.id)?
            } else {
                Vec::new()
            };
            let list: Vec<String> = callees
                .into_iter()
                .map(|(_, n)| format!("{} ({:?})", n.name, n.kind))
                .collect();
            serde_json::to_string_pretty(&list)?
        }
        "emocis_callers" => {
            let function = args.get("function").and_then(|f| f.as_str()).unwrap_or("");
            let candidates = store.find_nodes_by_name(function)?;
            let callers = if let Some(target) = candidates.first() {
                store.get_callers(&target.id)?
            } else {
                Vec::new()
            };
            let list: Vec<String> = callers
                .into_iter()
                .map(|(_, n)| format!("{} ({:?})", n.name, n.kind))
                .collect();
            serde_json::to_string_pretty(&list)?
        }
        "emocis_audit" => {
            let detector = DriftDetector::new(&store);
            serde_json::to_string_pretty(&detector.run_audit(&project_id)?)?
        }
        "emocis_health" => {
            let scorer = HealthScorer::new(&store);
            serde_json::to_string_pretty(&scorer.compute_health(&project_id)?)?
        }
        _ => anyhow::bail!("Unknown tool name: {}", name),
    };
    Ok(output)
}
