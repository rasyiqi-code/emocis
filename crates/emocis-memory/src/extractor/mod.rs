use emocis_core::{
    Evidence, IdentityGenerator, KnowledgeItem, KnowledgeStatus, KnowledgeType,
};
use regex::Regex;

pub struct KnowledgeExtractor {
    comment_pattern: Regex,
}

impl Default for KnowledgeExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl KnowledgeExtractor {
    pub fn new() -> Self {
        // Matches // @tag or // TAG: or /* @tag ... */ or # @tag
        let comment_pattern = Regex::new(
            r#"(?i)(?://|/\*|#)\s*(?:@|(?:\b))(decision|constraint|warning|caveat|rule|failed[-_]attempt|worklog|architecture)[:\s]+([^\n\r*]+)"#,
        ).unwrap();

        Self { comment_pattern }
    }

    /// Extract knowledge items from source code file
    pub fn extract_from_code(
        &self,
        project_id: &str,
        file_path: &str,
        content: &str,
    ) -> Vec<KnowledgeItem> {
        let mut items = Vec::new();

        for (line_idx, line) in content.lines().enumerate() {
            let line_num = line_idx as u32 + 1;

            if let Some(caps) = self.comment_pattern.captures(line) {
                let tag_raw = caps.get(1).map(|m| m.as_str().to_lowercase()).unwrap_or_default();
                let text = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("");

                if text.is_empty() {
                    continue;
                }

                let (k_type, default_title) = match tag_raw.as_str() {
                    "decision" => (KnowledgeType::Decision, "Architectural Decision"),
                    "constraint" => (KnowledgeType::Constraint, "System Constraint"),
                    "warning" | "caveat" => (KnowledgeType::Warning, "Engineering Warning"),
                    "rule" => (KnowledgeType::BusinessRule, "Business Rule"),
                    "failed-attempt" | "failed_attempt" => (KnowledgeType::FailedAttempt, "Failed Attempt"),
                    "worklog" => (KnowledgeType::Worklog, "Development Worklog"),
                    "architecture" => (KnowledgeType::Architecture, "Architecture Note"),
                    _ => (KnowledgeType::Decision, "Knowledge Note"),
                };

                let title = if let Some((head, _)) = text.split_once(':') {
                    head.trim()
                } else if text.len() > 50 {
                    &text[..50]
                } else {
                    text
                };

                let id = IdentityGenerator::for_knowledge(
                    &format!("{:?}", k_type).to_lowercase(),
                    &format!("{}_{}", file_path.replace('/', "_"), line_num),
                );

                let item = KnowledgeItem {
                    id,
                    project_id: project_id.to_string(),
                    knowledge_type: k_type,
                    title: if title.is_empty() { default_title.to_string() } else { title.to_string() },
                    content: text.to_string(),
                    status: KnowledgeStatus::Active,
                    confidence: 0.95,
                    evidence: vec![Evidence {
                        evidence_type: "comment".to_string(),
                        file: Some(file_path.to_string()),
                        start_line: Some(line_num),
                        end_line: Some(line_num),
                        commit_id: None,
                        snippet: Some(line.trim().to_string()),
                    }],
                    affects: vec![file_path.to_string()],
                    tags: vec![tag_raw],
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                };

                items.push(item);
            }
        }

        items
    }

    /// Extract knowledge items from markdown documentation files
    pub fn extract_from_markdown(
        &self,
        project_id: &str,
        file_path: &str,
        content: &str,
    ) -> Vec<KnowledgeItem> {
        let mut items = Vec::new();
        let header_regex = Regex::new(r#"^(#{1,4})\s+(.+)$"#).unwrap();

        let mut current_header = String::new();
        let mut current_content = Vec::new();
        let mut start_line = 1;

        for (idx, line) in content.lines().enumerate() {
            let line_num = idx as u32 + 1;
            if let Some(caps) = header_regex.captures(line) {
                // Flush previous section
                if !current_header.is_empty() && !current_content.is_empty() {
                    let content_text = current_content.join("\n").trim().to_string();
                    if !content_text.is_empty() {
                        let k_type = if current_header.to_lowercase().contains("architecture") {
                            KnowledgeType::Architecture
                        } else if current_header.to_lowercase().contains("decision") {
                            KnowledgeType::Decision
                        } else if current_header.to_lowercase().contains("constraint") {
                            KnowledgeType::Constraint
                        } else {
                            KnowledgeType::BusinessRule
                        };

                        let id = IdentityGenerator::for_knowledge(
                            &format!("{:?}", k_type).to_lowercase(),
                            &format!("{}_{}", file_path.replace('/', "_"), start_line),
                        );

                        items.push(KnowledgeItem {
                            id,
                            project_id: project_id.to_string(),
                            knowledge_type: k_type,
                            title: current_header.clone(),
                            content: content_text,
                            status: KnowledgeStatus::Active,
                            confidence: 0.90,
                            evidence: vec![Evidence {
                                evidence_type: "documentation".to_string(),
                                file: Some(file_path.to_string()),
                                start_line: Some(start_line),
                                end_line: Some(line_num - 1),
                                commit_id: None,
                                snippet: None,
                            }],
                            affects: vec![file_path.to_string()],
                            tags: vec!["markdown".to_string()],
                            created_at: chrono::Utc::now(),
                            updated_at: chrono::Utc::now(),
                        });
                    }
                }

                current_header = caps.get(2).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
                current_content.clear();
                start_line = line_num;
            } else if !current_header.is_empty() {
                current_content.push(line);
            }
        }

        // Flush last section
        if !current_header.is_empty() && !current_content.is_empty() {
            let content_text = current_content.join("\n").trim().to_string();
            if !content_text.is_empty() {
                let id = IdentityGenerator::for_knowledge(
                    "doc",
                    &format!("{}_{}", file_path.replace('/', "_"), start_line),
                );
                items.push(KnowledgeItem {
                    id,
                    project_id: project_id.to_string(),
                    knowledge_type: KnowledgeType::Architecture,
                    title: current_header,
                    content: content_text,
                    status: KnowledgeStatus::Active,
                    confidence: 0.90,
                    evidence: vec![Evidence {
                        evidence_type: "documentation".to_string(),
                        file: Some(file_path.to_string()),
                        start_line: Some(start_line),
                        end_line: Some(content.lines().count() as u32),
                        commit_id: None,
                        snippet: None,
                    }],
                    affects: vec![file_path.to_string()],
                    tags: vec!["markdown".to_string()],
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                });
            }
        }

        items
    }
}
