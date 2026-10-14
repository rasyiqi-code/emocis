use emocis_core::*;

#[test]
fn test_sqlite_core_flow() {
    let store = SqliteStore::open_in_memory().expect("Failed to open in-memory store");

    // 1. Project
    store
        .insert_project("proj:emocis", "EMOCIS", "/path/to/project")
        .expect("Insert project failed");

    // 2. Snapshot
    let snapshot = Snapshot {
        id: "snapshot:commit_abc123".to_string(),
        project_id: "proj:emocis".to_string(),
        commit_hash: "abc123456789".to_string(),
        branch: Some("main".to_string()),
        author: Some("Dev".to_string()),
        message: Some("Initial indexing".to_string()),
        created_at: chrono::Utc::now(),
    };
    store.insert_snapshot(&snapshot).expect("Insert snapshot failed");

    // 3. Nodes: Caller & Callee (PRD Part II Section 46 checkout -> charge)
    let checkout_node = Node {
        id: IdentityGenerator::for_symbol(
            "typescript",
            "src/checkout.ts",
            "checkout",
            &NodeKind::Function,
        ),
        kind: NodeKind::Function,
        name: "checkout".to_string(),
        project_id: "proj:emocis".to_string(),
        language: Some("typescript".to_string()),
        source: Some(SourceLocation {
            file: "src/checkout.ts".to_string(),
            start_line: 1,
            start_column: 1,
            end_line: 12,
            end_column: 2,
        }),
        identity: NodeIdentity {
            qualified_name: "checkout".to_string(),
            signature: Some("checkout(orderId: string): Promise<Payment>".to_string()),
        },
        state: NodeState::default(),
        provenance: Provenance::default(),
        metadata: serde_json::json!({ "async": true }),
    };

    let charge_node = Node {
        id: IdentityGenerator::for_symbol(
            "typescript",
            "src/payment.ts",
            "charge",
            &NodeKind::Function,
        ),
        kind: NodeKind::Function,
        name: "charge".to_string(),
        project_id: "proj:emocis".to_string(),
        language: Some("typescript".to_string()),
        source: Some(SourceLocation {
            file: "src/payment.ts".to_string(),
            start_line: 45,
            start_column: 1,
            end_line: 60,
            end_column: 2,
        }),
        identity: NodeIdentity {
            qualified_name: "charge".to_string(),
            signature: Some("charge(amount: number): Promise<PaymentResult>".to_string()),
        },
        state: NodeState::default(),
        provenance: Provenance::default(),
        metadata: serde_json::Value::Null,
    };

    store.insert_node(&checkout_node).expect("Insert checkout node failed");
    store.insert_node(&charge_node).expect("Insert charge node failed");

    // 4. Edge: checkout --calls--> charge
    let call_edge = Edge {
        id: IdentityGenerator::for_edge("calls", &checkout_node.id, &charge_node.id),
        from: checkout_node.id.clone(),
        to: charge_node.id.clone(),
        kind: EdgeKind::Calls,
        qualifier: serde_json::json!({
            "argument_index": 0,
            "parameter": "amount"
        }),
        provenance: Provenance::default(),
        temporal: TemporalRange {
            valid_from: snapshot.id.clone(),
            valid_until: None,
        },
    };
    store.insert_edge(&call_edge).expect("Insert edge failed");

    // 5. Test Queries: Callees and Callers
    let callees = store.get_callees(&checkout_node.id).expect("Query callees failed");
    assert_eq!(callees.len(), 1);
    assert_eq!(callees[0].1.name, "charge");
    assert_eq!(callees[0].0.kind, EdgeKind::Calls);

    let callers = store.get_callers(&charge_node.id).expect("Query callers failed");
    assert_eq!(callers.len(), 1);
    assert_eq!(callers[0].1.name, "checkout");

    // 6. Knowledge Item: PRD Part I Section 38
    let knowledge = KnowledgeItem {
        id: IdentityGenerator::for_knowledge("decision", "001"),
        project_id: "proj:emocis".to_string(),
        knowledge_type: KnowledgeType::Decision,
        title: "Webhook is payment source of truth".to_string(),
        content: "Payment status must be determined from webhook events.".to_string(),
        status: KnowledgeStatus::Active,
        confidence: 0.95,
        evidence: vec![Evidence {
            evidence_type: "comment".to_string(),
            file: Some("src/payment/webhook.ts".to_string()),
            start_line: Some(42),
            end_line: Some(42),
            commit_id: Some("8a31f".to_string()),
            snippet: Some("// Webhook is payment source of truth".to_string()),
        }],
        affects: vec!["PaymentService".to_string(), "Checkout".to_string()],
        tags: vec!["payment".to_string(), "webhook".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    store.insert_knowledge_item(&knowledge).expect("Insert knowledge item failed");

    let items = store.list_knowledge_items("proj:emocis").expect("List knowledge items failed");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].title, "Webhook is payment source of truth");
    assert_eq!(items[0].evidence.len(), 1);

    let search_res = store.search_knowledge("webhook").expect("Search knowledge failed");
    assert_eq!(search_res.len(), 1);
}
