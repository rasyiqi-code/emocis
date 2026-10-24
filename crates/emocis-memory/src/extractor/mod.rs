use emocis_core::{Evidence, IdentityGenerator, KnowledgeItem, KnowledgeStatus, KnowledgeType};
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
        // Matches // @tag or // TAG: or /* @tag ... */ or # @tag or * @tag, including natural dev tags
        // and Indonesian equivalents (keputusan, batasan, peringatan, perbaikan, aturan, alasan, etc.)
        let comment_pattern = Regex::new(
            r#"(?i)(?://|/\*|#|\*)\s*(?:@|(?:\b))(decision|adr|keputusan|alasan|reason|rationale|why|constraint|batasan|limitasi|idempotency|invariant|security|keamanan|warning|peringatan|perhatian|caveat|workaround|caution|danger|bahaya|bug[-_ ]?fix|bug|fix|perbaikan|hotfix|patch|critical|kritis|rule|aturan|kebijakan|policy|b\d+|failed[-_ ]?attempt|gagal|abandoned|worklog|log|riwayat|todo|architecture|arsitektur|arch|design|note|catatan|notice|important|penting|perf|performance|mitigation|hack)[:\s]+([^\n\r*]+)"#,
        ).unwrap();

        Self { comment_pattern }
    }

    /// Extract knowledge items from source code file with multi-line comment aggregation
    pub fn extract_from_code(
        &self,
        project_id: &str,
        file_path: &str,
        content: &str,
    ) -> Vec<KnowledgeItem> {
        let mut items = Vec::new();

        struct PendingItem {
            tag_raw: String,
            tag_norm: &'static str,
            k_type: KnowledgeType,
            default_title: &'static str,
            first_line_text: String,
            lines: Vec<String>,
            snippet_lines: Vec<String>,
            start_line: u32,
            end_line: u32,
            comment_style: CommentStyle,
        }

        #[derive(PartialEq)]
        enum CommentStyle {
            SlashSlash,
            Hash,
            Block,
        }

        let flush_pending = |pending: PendingItem, items: &mut Vec<KnowledgeItem>| {
            let clean_first = pending.first_line_text.trim();
            let title = if clean_first.chars().count() > 60 {
                format!(
                    "{}: {}...",
                    pending.tag_raw.to_uppercase(),
                    clean_first.chars().take(57).collect::<String>()
                )
            } else if !clean_first.is_empty() {
                format!("{}: {}", pending.tag_raw.to_uppercase(), clean_first)
            } else {
                pending.default_title.to_string()
            };

            let content_text = pending.lines.join("\n").trim().to_string();
            let snippet = pending.snippet_lines.join("\n");

            let id = IdentityGenerator::for_knowledge(
                &format!("{:?}", pending.k_type).to_lowercase(),
                &format!("{}_{}", file_path.replace('/', "_"), pending.start_line),
            );

            let mut tags = vec![pending.tag_norm.to_string()];
            let tag_lower = pending.tag_raw.to_lowercase();
            if tag_lower != pending.tag_norm {
                tags.push(tag_lower);
            }

            items.push(KnowledgeItem {
                id,
                project_id: project_id.to_string(),
                knowledge_type: pending.k_type,
                title,
                content: content_text,
                status: KnowledgeStatus::Active,
                confidence: 0.95,
                evidence: vec![Evidence {
                    evidence_type: "comment".to_string(),
                    file: Some(file_path.to_string()),
                    start_line: Some(pending.start_line),
                    end_line: Some(pending.end_line),
                    commit_id: None,
                    snippet: Some(snippet),
                }],
                affects: vec![file_path.to_string()],
                tags,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            });
        };

        let mut current_pending: Option<PendingItem> = None;
        let mut in_block_comment = false;

        for (line_idx, line) in content.lines().enumerate() {
            let line_num = line_idx as u32 + 1;
            let trimmed = line.trim();

            if let Some(caps) = self.comment_pattern.captures(line) {
                // If there's an active pending item, flush it first
                if let Some(pending) = current_pending.take() {
                    flush_pending(pending, &mut items);
                }

                let tag_raw = caps
                    .get(1)
                    .map(|m| m.as_str().to_lowercase())
                    .unwrap_or_default();
                let text = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("");

                if text.is_empty() {
                    continue;
                }

                let tag_clean = tag_raw.replace(['-', ' '], "_");
                let (k_type, default_title, tag_norm) = match tag_clean.as_str() {
                    "decision" | "adr" | "keputusan" | "alasan" | "reason" | "rationale" | "why" | "mitigation" => {
                        (KnowledgeType::Decision, "Architectural Decision", "decision")
                    }
                    "constraint" | "batasan" | "limitasi" | "idempotency" | "invariant" | "security" | "keamanan" => {
                        (KnowledgeType::Constraint, "System Constraint", "constraint")
                    }
                    "warning" | "peringatan" | "perhatian" | "caveat" | "workaround" | "caution" | "danger" | "bahaya" | "hack" | "notice" | "perf" | "performance" => {
                        (KnowledgeType::Warning, "Engineering Warning", "warning")
                    }
                    "bug_fix" | "bug" | "fix" | "perbaikan" | "hotfix" | "patch" => {
                        (KnowledgeType::Warning, "Bug Fix Note", "bugfix")
                    }
                    "critical" | "kritis" => {
                        (KnowledgeType::Warning, "Critical Warning", "critical")
                    }
                    "rule" | "aturan" | "kebijakan" | "policy" => {
                        (KnowledgeType::BusinessRule, "Business Rule", "rule")
                    }
                    t if t.starts_with('b') && t[1..].chars().all(|c| c.is_ascii_digit()) => {
                        (KnowledgeType::BusinessRule, "Business Rule", "rule")
                    }
                    "failed_attempt" | "gagal" | "abandoned" => {
                        (KnowledgeType::FailedAttempt, "Failed Attempt", "failed_attempt")
                    }
                    "worklog" | "log" | "riwayat" | "todo" => {
                        (KnowledgeType::Worklog, "Development Worklog", "worklog")
                    }
                    "architecture" | "arsitektur" | "arch" | "design" => {
                        (KnowledgeType::Architecture, "Architecture Note", "architecture")
                    }
                    "note" | "catatan" | "important" | "penting" => {
                        (KnowledgeType::Architecture, "Engineering Note", "note")
                    }
                    _ => (KnowledgeType::Decision, "Knowledge Note", "note"),
                };

                // Determine comment style and whether it's inline after code
                if trimmed.starts_with("//") {
                    current_pending = Some(PendingItem {
                        tag_raw,
                        tag_norm,
                        k_type,
                        default_title,
                        first_line_text: text.to_string(),
                        lines: vec![text.to_string()],
                        snippet_lines: vec![line.trim().to_string()],
                        start_line: line_num,
                        end_line: line_num,
                        comment_style: CommentStyle::SlashSlash,
                    });
                } else if trimmed.starts_with("#") {
                    current_pending = Some(PendingItem {
                        tag_raw,
                        tag_norm,
                        k_type,
                        default_title,
                        first_line_text: text.to_string(),
                        lines: vec![text.to_string()],
                        snippet_lines: vec![line.trim().to_string()],
                        start_line: line_num,
                        end_line: line_num,
                        comment_style: CommentStyle::Hash,
                    });
                } else if trimmed.starts_with("/*") || in_block_comment || trimmed.starts_with('*') {
                    in_block_comment = !trimmed.contains("*/");
                    if trimmed.contains("*/") {
                        // Single-line block comment: flush immediately
                        flush_pending(
                            PendingItem {
                                tag_raw,
                                tag_norm,
                                k_type,
                                default_title,
                                first_line_text: text.to_string(),
                                lines: vec![text.to_string()],
                                snippet_lines: vec![line.trim().to_string()],
                                start_line: line_num,
                                end_line: line_num,
                                comment_style: CommentStyle::Block,
                            },
                            &mut items,
                        );
                    } else {
                        current_pending = Some(PendingItem {
                            tag_raw,
                            tag_norm,
                            k_type,
                            default_title,
                            first_line_text: text.to_string(),
                            lines: vec![text.to_string()],
                            snippet_lines: vec![line.trim().to_string()],
                            start_line: line_num,
                            end_line: line_num,
                            comment_style: CommentStyle::Block,
                        });
                    }
                } else {
                    // Inline comment after code: flush immediately without multi-line aggregation
                    flush_pending(
                        PendingItem {
                            tag_raw,
                            tag_norm,
                            k_type,
                            default_title,
                            first_line_text: text.to_string(),
                            lines: vec![text.to_string()],
                            snippet_lines: vec![line.trim().to_string()],
                            start_line: line_num,
                            end_line: line_num,
                            comment_style: CommentStyle::SlashSlash,
                        },
                        &mut items,
                    );
                }
            } else if let Some(ref mut pending) = current_pending {
                // Check if this line continues the active comment block
                match pending.comment_style {
                    CommentStyle::SlashSlash => {
                        if trimmed.starts_with("//") {
                            let body = trimmed.strip_prefix("//").unwrap_or("");
                            let clean_body = body.strip_prefix(' ').unwrap_or(body);
                            pending.lines.push(clean_body.to_string());
                            pending.snippet_lines.push(line.trim().to_string());
                            pending.end_line = line_num;
                        } else {
                            let item = current_pending.take().unwrap();
                            flush_pending(item, &mut items);
                        }
                    }
                    CommentStyle::Hash => {
                        if trimmed.starts_with("#") && !trimmed.starts_with("#!") {
                            let body = trimmed.strip_prefix("#").unwrap_or("");
                            let clean_body = body.strip_prefix(' ').unwrap_or(body);
                            pending.lines.push(clean_body.to_string());
                            pending.snippet_lines.push(line.trim().to_string());
                            pending.end_line = line_num;
                        } else {
                            let item = current_pending.take().unwrap();
                            flush_pending(item, &mut items);
                        }
                    }
                    CommentStyle::Block => {
                        if trimmed.contains("*/") {
                            in_block_comment = false;
                            let before_close = trimmed.split("*/").next().unwrap_or("").trim_start_matches('*').trim();
                            if !before_close.is_empty() {
                                pending.lines.push(before_close.to_string());
                            }
                            pending.snippet_lines.push(line.trim().to_string());
                            pending.end_line = line_num;
                            let item = current_pending.take().unwrap();
                            flush_pending(item, &mut items);
                        } else {
                            let clean_body = trimmed.trim_start_matches('*').trim();
                            if !clean_body.is_empty() {
                                pending.lines.push(clean_body.to_string());
                            }
                            pending.snippet_lines.push(line.trim().to_string());
                            pending.end_line = line_num;
                        }
                    }
                }
            } else {
                if trimmed.starts_with("/*") {
                    in_block_comment = true;
                }
                if trimmed.contains("*/") {
                    in_block_comment = false;
                }
            }
        }

        // Flush any trailing pending item
        if let Some(pending) = current_pending.take() {
            flush_pending(pending, &mut items);
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

        let is_boilerplate = |h: &str| -> bool {
            let lower = h.to_lowercase();
            lower.contains("daftar isi")
                || lower.contains("table of content")
                || lower.contains("lisensi")
                || lower.contains("license")
                || lower.contains("prasyarat")
                || lower.contains("prerequisite")
                || lower.contains("faq")
                || lower.contains("troubleshooting")
                || lower.starts_with("opsi ")
                || lower.contains("fitur utama")
                || lower.contains("kenapa ")
                || lower.contains("why ")
                || lower.contains("instalasi")
                || lower.contains("installation")
                || lower.contains("getting started")
                || lower.contains("quick start")
                || lower.contains("changelog")
        };

        let categorize_header = |h: &str| -> KnowledgeType {
            let lower = h.to_lowercase();
            if lower.contains("architecture")
                || lower.contains("arsitektur")
                || lower.contains("alur kerja")
                || lower.contains("desain")
                || lower.contains("design")
            {
                KnowledgeType::Architecture
            } else if lower.contains("decision")
                || lower.contains("keputusan")
                || lower.contains("adr")
                || lower.contains("alasan")
                || lower.contains("reason")
            {
                KnowledgeType::Decision
            } else if lower.contains("constraint")
                || lower.contains("batasan")
                || lower.contains("keamanan")
                || lower.contains("security")
                || lower.contains("limitasi")
            {
                KnowledgeType::Constraint
            } else if lower.contains("rule")
                || lower.contains("aturan")
                || lower.contains("kebijakan")
                || lower.contains("policy")
            {
                KnowledgeType::BusinessRule
            } else if lower.contains("bug")
                || lower.contains("fix")
                || lower.contains("perbaikan")
                || lower.contains("warning")
                || lower.contains("peringatan")
            {
                KnowledgeType::Warning
            } else if lower.contains("failed") || lower.contains("gagal") {
                KnowledgeType::FailedAttempt
            } else if lower.contains("worklog") || lower.contains("riwayat") || lower.contains("log") {
                KnowledgeType::Worklog
            } else {
                KnowledgeType::Architecture
            }
        };

        for (idx, line) in content.lines().enumerate() {
            let line_num = idx as u32 + 1;
            if let Some(caps) = header_regex.captures(line) {
                // Flush previous section
                if !current_header.is_empty()
                    && !current_content.is_empty()
                    && !is_boilerplate(&current_header)
                {
                    let content_text = current_content.join("\n").trim().to_string();
                    if !content_text.is_empty() {
                        let k_type = categorize_header(&current_header);

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

                current_header = caps
                    .get(2)
                    .map(|m| m.as_str().trim().to_string())
                    .unwrap_or_default();
                current_content.clear();
                start_line = line_num;
            } else if !current_header.is_empty() {
                current_content.push(line);
            }
        }

        // Flush last section
        if !current_header.is_empty()
            && !current_content.is_empty()
            && !is_boilerplate(&current_header)
        {
            let content_text = current_content.join("\n").trim().to_string();
            if !content_text.is_empty() {
                let k_type = categorize_header(&current_header);

                let id = IdentityGenerator::for_knowledge(
                    &format!("{:?}", k_type).to_lowercase(),
                    &format!("{}_{}", file_path.replace('/', "_"), start_line),
                );
                items.push(KnowledgeItem {
                    id,
                    project_id: project_id.to_string(),
                    knowledge_type: k_type,
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
