use crate::models::NodeKind;

/// Helper to generate canonical stable identity for nodes (PRD Part II Section 10)
pub struct IdentityGenerator;

impl IdentityGenerator {
    /// Formula: `symbol:<language>:<module_or_file>:<qualified_name>:<kind>`
    pub fn for_symbol(
        language: &str,
        module_path: &str,
        qualified_name: &str,
        kind: &NodeKind,
    ) -> String {
        let kind_str = format!("{:?}", kind).to_lowercase();
        format!(
            "symbol:{}:{}:{}:{}",
            language,
            module_path.trim_start_matches('/'),
            qualified_name,
            kind_str
        )
    }

    /// Formula for files: `file:<relative_path>`
    pub fn for_file(relative_path: &str) -> String {
        format!("file:{}", relative_path.trim_start_matches('/'))
    }

    /// Formula for modules: `module:<name>`
    pub fn for_module(module_name: &str) -> String {
        format!("module:{}", module_name)
    }

    /// Formula for call site: `callsite:<caller_id>:<callee_name>:<index>`
    pub fn for_callsite(caller_id: &str, callee_name: &str, index: u32) -> String {
        format!("callsite:{}:{}:{}", caller_id, callee_name, index)
    }

    /// Formula for edges: `edge:<kind>:<from>-><to>`
    pub fn for_edge(kind: &str, from_id: &str, to_id: &str) -> String {
        format!("edge:{}:{}->{}", kind, from_id, to_id)
    }

    /// Formula for knowledge items: `knowledge:<type>:<uuid>`
    pub fn for_knowledge(knowledge_type: &str, unique_id: &str) -> String {
        format!("knowledge:{}:{}", knowledge_type, unique_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stable_identity() {
        let id = IdentityGenerator::for_symbol(
            "typescript",
            "src/payment/service.ts",
            "PaymentService.createPayment",
            &NodeKind::Method,
        );
        assert_eq!(
            id,
            "symbol:typescript:src/payment/service.ts:PaymentService.createPayment:method"
        );
    }
}
