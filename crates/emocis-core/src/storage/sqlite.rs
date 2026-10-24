use crate::error::Result;
use crate::models::*;
use rusqlite::{params, params_from_iter, types::Value as SqlValue, Connection};
use std::path::Path;

pub struct SqliteStore {
    conn: Connection,
}

impl SqliteStore {
    /// Open or create database at given path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    /// Open in-memory database (useful for tests and fast transient analysis)
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    /// Run SQLite schema migration
    pub fn migrate(&self) -> Result<()> {
        let _ = self.conn.query_row("PRAGMA journal_mode = WAL;", [], |_| Ok(()));
        let _ = self.conn.query_row("PRAGMA synchronous = NORMAL;", [], |_| Ok(()));
        let _ = self.conn.query_row("PRAGMA temp_store = MEMORY;", [], |_| Ok(()));
        let _ = self.conn.query_row("PRAGMA foreign_keys = ON;", [], |_| Ok(()));
        let schema = include_str!("schema.sql");
        self.conn.execute_batch(schema)?;
        let _ = self.conn.execute_batch(
            "INSERT INTO knowledge_fts(item_id, title, content, tags)
             SELECT id, title, content, COALESCE(tags, '') FROM knowledge_items
             WHERE id NOT IN (SELECT item_id FROM knowledge_fts);",
        );
        Ok(())
    }

    pub fn execute_batch(&self, sql: &str) -> Result<()> {
        self.conn.execute_batch(sql)?;
        Ok(())
    }

    /// Remove only source-derived data; manually recorded knowledge survives reindexing.
    /// The caller owns the transaction so a failed parse restores the previous index.
    pub fn clear_source_index(&self, project_id: &str) -> Result<()> {
        let mut stmt = self.conn.prepare(
            "SELECT k.id FROM knowledge_items k WHERE k.project_id = ?1
            AND EXISTS (SELECT 1 FROM evidence e WHERE e.target_type = 'knowledge'
              AND e.target_id = k.id AND e.evidence_type IN ('comment', 'documentation'))
            AND NOT EXISTS (SELECT 1 FROM evidence e WHERE e.target_type = 'knowledge'
              AND e.target_id = k.id AND e.evidence_type NOT IN ('comment', 'documentation'))",
        )?;
        let ids = stmt
            .query_map(params![project_id], |row| row.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        for id in ids {
            self.conn.execute(
                "DELETE FROM evidence WHERE target_type = 'knowledge' AND target_id = ?1",
                params![id],
            )?;
            self.conn
                .execute("DELETE FROM knowledge_items WHERE id = ?1", params![id])?;
        }
        self.conn.execute(
            "DELETE FROM evidence WHERE target_type = 'edge' AND target_id IN (
            SELECT id FROM edges WHERE from_node_id IN (SELECT id FROM nodes WHERE project_id = ?1)
            OR to_node_id IN (SELECT id FROM nodes WHERE project_id = ?1))",
            params![project_id],
        )?;
        self.conn.execute(
            "DELETE FROM edges WHERE from_node_id IN (SELECT id FROM nodes WHERE project_id = ?1)
            OR to_node_id IN (SELECT id FROM nodes WHERE project_id = ?1)",
            params![project_id],
        )?;
        self.conn.execute(
            "DELETE FROM evidence WHERE target_type = 'node' AND target_id IN
            (SELECT id FROM nodes WHERE project_id = ?1)",
            params![project_id],
        )?;
        self.conn.execute(
            "DELETE FROM nodes WHERE project_id = ?1",
            params![project_id],
        )?;
        self.conn.execute(
            "DELETE FROM file_hashes WHERE project_id = ?1",
            params![project_id],
        )?;
        Ok(())
    }

    /// Clear index data for a specific file (used for incremental re-indexing)
    pub fn clear_file_index(&self, project_id: &str, file_path: &str) -> Result<()> {
        let mut k_stmt = self.conn.prepare(
            "SELECT DISTINCT target_id FROM evidence
             WHERE target_type = 'knowledge' AND file_path = ?1",
        )?;
        let k_ids = k_stmt
            .query_map(params![file_path], |row| row.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        for id in k_ids {
            self.conn.execute(
                "DELETE FROM evidence WHERE target_type = 'knowledge' AND target_id = ?1",
                params![id],
            )?;
            self.conn.execute(
                "DELETE FROM knowledge_items WHERE id = ?1 AND project_id = ?2",
                params![id, project_id],
            )?;
        }

        self.conn.execute(
            "DELETE FROM evidence WHERE target_type = 'edge' AND target_id IN (
                SELECT id FROM edges WHERE from_node_id IN (SELECT id FROM nodes WHERE project_id = ?1 AND file_path = ?2)
                OR to_node_id IN (SELECT id FROM nodes WHERE project_id = ?1 AND file_path = ?2)
            )",
            params![project_id, file_path],
        )?;

        self.conn.execute(
            "DELETE FROM edges WHERE from_node_id IN (SELECT id FROM nodes WHERE project_id = ?1 AND file_path = ?2)
             OR to_node_id IN (SELECT id FROM nodes WHERE project_id = ?1 AND file_path = ?2)",
            params![project_id, file_path],
        )?;

        self.conn.execute(
            "DELETE FROM evidence WHERE target_type = 'node' AND target_id IN (
                SELECT id FROM nodes WHERE project_id = ?1 AND file_path = ?2
            )",
            params![project_id, file_path],
        )?;

        self.conn.execute(
            "DELETE FROM nodes WHERE project_id = ?1 AND file_path = ?2",
            params![project_id, file_path],
        )?;

        self.delete_file_hash(project_id, file_path)?;

        Ok(())
    }

    pub fn get_file_hashes(&self, project_id: &str) -> Result<std::collections::HashMap<String, String>> {
        let mut stmt = self.conn.prepare(
            "SELECT file_path, content_hash FROM file_hashes WHERE project_id = ?1",
        )?;
        let rows = stmt.query_map(params![project_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        let mut map = std::collections::HashMap::new();
        for r in rows {
            let (path, hash) = r?;
            map.insert(path, hash);
        }
        Ok(map)
    }

    pub fn set_file_hash(&self, project_id: &str, file_path: &str, hash: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO file_hashes (project_id, file_path, content_hash, updated_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(project_id, file_path) DO UPDATE SET content_hash = excluded.content_hash, updated_at = excluded.updated_at",
            params![project_id, file_path, hash, now],
        )?;
        Ok(())
    }

    pub fn delete_file_hash(&self, project_id: &str, file_path: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM file_hashes WHERE project_id = ?1 AND file_path = ?2",
            params![project_id, file_path],
        )?;
        Ok(())
    }

    pub fn find_nodes_by_file(&self, file: &str) -> Result<Vec<Node>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM nodes WHERE file_path = ?1 ORDER BY start_line, id")?;
        let ids = stmt.query_map(params![file], |row| row.get::<_, String>(0))?;
        let mut nodes = Vec::new();
        for id in ids {
            if let Some(node) = self.get_node(&id?)? {
                nodes.push(node);
            }
        }
        Ok(nodes)
    }

    pub fn replace_edge(&self, old_id: &str, edge: &Edge) -> Result<()> {
        self.conn
            .execute("DELETE FROM edges WHERE id = ?1", params![old_id])?;
        self.insert_edge(edge)
    }

    // ==========================================
    // Project & Snapshot Operations
    // ==========================================

    pub fn insert_project(&self, id: &str, name: &str, root_path: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO projects (id, name, root_path, created_at) VALUES (?1, ?2, ?3, ?4) ON CONFLICT(id) DO UPDATE SET name = excluded.name, root_path = excluded.root_path",
            params![id, name, root_path, now],
        )?;
        Ok(())
    }

    pub fn insert_snapshot(&self, snapshot: &Snapshot) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO snapshots (id, project_id, commit_hash, branch, author, message, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                snapshot.id,
                snapshot.project_id,
                snapshot.commit_hash,
                snapshot.branch,
                snapshot.author,
                snapshot.message,
                snapshot.created_at.to_rfc3339()
            ],
        )?;
        Ok(())
    }

    // ==========================================
    // Node Operations
    // ==========================================

    pub fn insert_node(&self, node: &Node) -> Result<()> {
        let (file_path, start_line, start_col, end_line, end_col) = match &node.source {
            Some(loc) => (
                Some(loc.file.clone()),
                Some(loc.start_line),
                Some(loc.start_column),
                Some(loc.end_line),
                Some(loc.end_column),
            ),
            None => (None, None, None, None, None),
        };

        let kind_str = serde_json::to_string(&node.kind)?.replace('"', "");
        let category_str = serde_json::to_string(&node.kind.category())?.replace('"', "");
        let status_str = serde_json::to_string(&node.state.status)?.replace('"', "");
        let certainty_str = serde_json::to_string(&node.provenance.certainty)?.replace('"', "");
        let metadata_str = serde_json::to_string(&node.metadata)?;

        self.conn.execute(
            "INSERT OR REPLACE INTO nodes (
                id, project_id, kind, category, name, language,
                file_path, start_line, start_column, end_line, end_column,
                qualified_name, signature, status, confidence, certainty, metadata
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            params![
                node.id,
                node.project_id,
                kind_str,
                category_str,
                node.name,
                node.language,
                file_path,
                start_line,
                start_col,
                end_line,
                end_col,
                node.identity.qualified_name,
                node.identity.signature,
                status_str,
                node.provenance.confidence,
                certainty_str,
                metadata_str
            ],
        )?;

        // Insert evidence
        for ev in &node.provenance.evidence {
            self.conn.execute(
                "INSERT INTO evidence (target_type, target_id, evidence_type, file_path, start_line, end_line, commit_id, snippet)
                 VALUES ('node', ?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    node.id,
                    ev.evidence_type,
                    ev.file,
                    ev.start_line,
                    ev.end_line,
                    ev.commit_id,
                    ev.snippet
                ],
            )?;
        }

        Ok(())
    }

    fn hydrate_node(row: &rusqlite::Row) -> Result<Node> {
        let kind_raw: String = row.get(2)?;
        let kind: NodeKind =
            serde_json::from_str(&format!("\"{}\"", kind_raw)).unwrap_or(NodeKind::Function);

        let file_path: Option<String> = row.get(5)?;
        let source = file_path.map(|file| SourceLocation {
            file,
            start_line: row.get::<_, Option<u32>>(6).unwrap_or(None).unwrap_or(0),
            start_column: row.get::<_, Option<u32>>(7).unwrap_or(None).unwrap_or(0),
            end_line: row.get::<_, Option<u32>>(8).unwrap_or(None).unwrap_or(0),
            end_column: row.get::<_, Option<u32>>(9).unwrap_or(None).unwrap_or(0),
        });

        let status_raw: String = row.get(12)?;
        let status: NodeStatus =
            serde_json::from_str(&format!("\"{}\"", status_raw)).unwrap_or(NodeStatus::Active);

        let certainty_raw: String = row.get(14)?;
        let certainty: Certainty = serde_json::from_str(&format!("\"{}\"", certainty_raw))
            .unwrap_or(Certainty::Certain);

        let metadata_raw: String = row.get(15)?;
        let metadata: serde_json::Value =
            serde_json::from_str(&metadata_raw).unwrap_or(serde_json::Value::Null);

        Ok(Node {
            id: row.get(0)?,
            project_id: row.get(1)?,
            kind,
            name: row.get(3)?,
            language: row.get(4)?,
            source,
            identity: NodeIdentity {
                qualified_name: row.get(10)?,
                signature: row.get(11)?,
            },
            state: NodeState { status },
            provenance: Provenance {
                source: ProvenanceSource::Parser,
                confidence: row.get(13)?,
                certainty,
                evidence: Vec::new(),
            },
            metadata,
        })
    }

    pub fn get_node(&self, id: &str) -> Result<Option<Node>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, kind, name, language,
                    file_path, start_line, start_column, end_line, end_column,
                    qualified_name, signature, status, confidence, certainty, metadata
             FROM nodes WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            return Ok(Some(Self::hydrate_node(row)?));
        }

        Ok(None)
    }

    pub fn find_nodes_by_project(&self, project_id: &str) -> Result<Vec<Node>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, kind, name, language,
                    file_path, start_line, start_column, end_line, end_column,
                    qualified_name, signature, status, confidence, certainty, metadata
             FROM nodes WHERE project_id = ?1",
        )?;

        let mut rows = stmt.query(params![project_id])?;
        let mut nodes = Vec::new();
        while let Some(row) = rows.next()? {
            nodes.push(Self::hydrate_node(row)?);
        }
        Ok(nodes)
    }

    pub fn find_nodes_by_name(&self, name: &str) -> Result<Vec<Node>> {
        let mut stmt = self.conn.prepare(
            "SELECT id FROM nodes
             WHERE name = ?1 OR qualified_name LIKE ?2
             ORDER BY
                (CASE WHEN name = ?1 THEN 0 ELSE 1 END),
                (CASE WHEN kind IN ('function', 'method', 'class') THEN 0 ELSE 1 END),
                id ASC",
        )?;
        let pattern = format!("%{}%", name);
        let mut rows = stmt.query(params![name, pattern])?;
        let mut nodes = Vec::new();

        while let Some(row) = rows.next()? {
            let id: String = row.get(0)?;
            if let Some(node) = self.get_node(&id)? {
                nodes.push(node);
            }
        }
        Ok(nodes)
    }

    pub fn count_nodes(&self, project_id: &str) -> Result<usize> {
        let mut stmt = self.conn.prepare("SELECT COUNT(*) FROM nodes WHERE project_id = ?1")?;
        let count: usize = stmt.query_row(params![project_id], |row| row.get(0))?;
        Ok(count)
    }

    pub fn count_knowledge_items(&self, project_id: &str) -> Result<usize> {
        let mut stmt = self.conn.prepare(
            "SELECT COUNT(*) FROM knowledge_items WHERE project_id = ?1 AND status NOT IN ('archived', 'superseded')",
        )?;
        let count: usize = stmt.query_row(params![project_id], |row| row.get(0))?;
        Ok(count)
    }

    // ==========================================
    // Edge Operations
    // ==========================================

    pub fn insert_edge(&self, edge: &Edge) -> Result<()> {
        let kind_str = serde_json::to_string(&edge.kind)?.replace('"', "");
        let category_str = serde_json::to_string(&edge.kind.category())?.replace('"', "");
        let certainty_str = serde_json::to_string(&edge.provenance.certainty)?.replace('"', "");
        let qualifier_str = serde_json::to_string(&edge.qualifier)?;

        self.conn.execute(
            "INSERT OR REPLACE INTO edges (
                id, from_node_id, to_node_id, kind, category, qualifier,
                confidence, certainty, valid_from, valid_until
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                edge.id,
                edge.from,
                edge.to,
                kind_str,
                category_str,
                qualifier_str,
                edge.provenance.confidence,
                certainty_str,
                edge.temporal.valid_from,
                edge.temporal.valid_until
            ],
        )?;

        Ok(())
    }

    fn hydrate_edge(row: &rusqlite::Row) -> Result<Edge> {
        let kind_raw: String = row.get(3)?;
        let kind: EdgeKind =
            serde_json::from_str(&format!("\"{}\"", kind_raw)).unwrap_or(EdgeKind::Calls);
        let qualifier_raw: String = row.get(4)?;
        let qualifier: serde_json::Value =
            serde_json::from_str(&qualifier_raw).unwrap_or(serde_json::Value::Null);
        let certainty_raw: String = row.get(6)?;
        let certainty: Certainty = serde_json::from_str(&format!("\"{}\"", certainty_raw))
            .unwrap_or(Certainty::Certain);

        Ok(Edge {
            id: row.get(0)?,
            from: row.get(1)?,
            to: row.get(2)?,
            kind,
            qualifier,
            provenance: Provenance {
                source: ProvenanceSource::Ast,
                confidence: row.get(5)?,
                certainty,
                evidence: Vec::new(),
            },
            temporal: TemporalRange {
                valid_from: row.get(7)?,
                valid_until: row.get(8)?,
            },
        })
    }

    pub fn get_all_edges(&self) -> Result<Vec<Edge>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, from_node_id, to_node_id, kind, qualifier, confidence, certainty, valid_from, valid_until
             FROM edges",
        )?;

        let mut rows = stmt.query([])?;
        let mut edges = Vec::new();

        while let Some(row) = rows.next()? {
            edges.push(Self::hydrate_edge(row)?);
        }

        Ok(edges)
    }

    pub fn get_edges_by_project(&self, project_id: &str) -> Result<Vec<Edge>> {
        let mut stmt = self.conn.prepare(
            "SELECT e.id, e.from_node_id, e.to_node_id, e.kind, e.qualifier, e.confidence, e.certainty, e.valid_from, e.valid_until
             FROM edges e
             JOIN nodes n ON e.from_node_id = n.id
             WHERE n.project_id = ?1",
        )?;

        let mut rows = stmt.query(params![project_id])?;
        let mut edges = Vec::new();

        while let Some(row) = rows.next()? {
            edges.push(Self::hydrate_edge(row)?);
        }

        Ok(edges)
    }

    pub fn get_edges_for_snapshot(&self, snapshot_id: &str) -> Result<Vec<Edge>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, from_node_id, to_node_id, kind, qualifier, confidence, certainty, valid_from, valid_until
             FROM edges
             WHERE valid_from = ?1 OR (?1 = 'active' AND (valid_until IS NULL OR valid_until = ''))",
        )?;

        let mut rows = stmt.query(params![snapshot_id])?;
        let mut edges = Vec::new();

        while let Some(row) = rows.next()? {
            edges.push(Self::hydrate_edge(row)?);
        }

        // If no edges matched snapshot_id directly, fallback to active edges
        if edges.is_empty() {
            return self.get_all_edges();
        }

        Ok(edges)
    }

    /// PRD Part II Section 39: `clg calls <FUNCTION>`
    pub fn get_callees(&self, caller_node_id: &str) -> Result<Vec<(Edge, Node)>> {
        let mut stmt = self.conn.prepare(
            "SELECT e.id, e.from_node_id, e.to_node_id, e.kind, e.qualifier, e.confidence, e.certainty, e.valid_from, e.valid_until
             FROM edges e
             WHERE e.from_node_id = ?1 AND e.kind = 'calls'",
        )?;

        let mut rows = stmt.query(params![caller_node_id])?;
        let mut results = Vec::new();

        while let Some(row) = rows.next()? {
            let to_id: String = row.get(2)?;
            let kind_raw: String = row.get(3)?;
            let kind: EdgeKind =
                serde_json::from_str(&format!("\"{}\"", kind_raw)).unwrap_or(EdgeKind::Calls);
            let qualifier_raw: String = row.get(4)?;
            let qualifier: serde_json::Value =
                serde_json::from_str(&qualifier_raw).unwrap_or(serde_json::Value::Null);
            let certainty_raw: String = row.get(6)?;
            let certainty: Certainty = serde_json::from_str(&format!("\"{}\"", certainty_raw))
                .unwrap_or(Certainty::Certain);

            let edge = Edge {
                id: row.get(0)?,
                from: row.get(1)?,
                to: to_id.clone(),
                kind,
                qualifier,
                provenance: Provenance {
                    source: ProvenanceSource::Ast,
                    confidence: row.get(5)?,
                    certainty,
                    evidence: Vec::new(),
                },
                temporal: TemporalRange {
                    valid_from: row.get(7)?,
                    valid_until: row.get(8)?,
                },
            };

            let target_node = match self.get_node(&to_id)? {
                Some(n) => n,
                None => {
                    let raw_name = edge
                        .qualifier
                        .get("raw_callee")
                        .and_then(|v| v.as_str())
                        .unwrap_or_else(|| to_id.split(':').nth(3).unwrap_or(&to_id));
                    let kind = if raw_name.contains('.') {
                        NodeKind::Method
                    } else {
                        NodeKind::Function
                    };
                    Node {
                        id: to_id.clone(),
                        kind,
                        name: raw_name.to_string(),
                        project_id: "".to_string(),
                        language: Some("typescript".to_string()),
                        source: None,
                        identity: NodeIdentity {
                            qualified_name: raw_name.to_string(),
                            signature: None,
                        },
                        state: NodeState::default(),
                        provenance: Provenance::default(),
                        metadata: serde_json::Value::Null,
                    }
                }
            };

            results.push((edge, target_node));
        }

        Ok(results)
    }

    /// PRD Part II Section 39: `clg callers <FUNCTION>`
    pub fn get_callers(&self, callee_node_id: &str) -> Result<Vec<(Edge, Node)>> {
        let mut stmt = self.conn.prepare(
            "SELECT e.id, e.from_node_id, e.to_node_id, e.kind, e.qualifier, e.confidence, e.certainty, e.valid_from, e.valid_until
             FROM edges e
             WHERE e.to_node_id = ?1 AND e.kind = 'calls'",
        )?;

        let mut rows = stmt.query(params![callee_node_id])?;
        let mut results = Vec::new();

        while let Some(row) = rows.next()? {
            let from_id: String = row.get(1)?;
            let kind_raw: String = row.get(3)?;
            let kind: EdgeKind =
                serde_json::from_str(&format!("\"{}\"", kind_raw)).unwrap_or(EdgeKind::Calls);
            let qualifier_raw: String = row.get(4)?;
            let qualifier: serde_json::Value =
                serde_json::from_str(&qualifier_raw).unwrap_or(serde_json::Value::Null);
            let certainty_raw: String = row.get(6)?;
            let certainty: Certainty = serde_json::from_str(&format!("\"{}\"", certainty_raw))
                .unwrap_or(Certainty::Certain);

            let edge = Edge {
                id: row.get(0)?,
                from: from_id.clone(),
                to: row.get(2)?,
                kind,
                qualifier,
                provenance: Provenance {
                    source: ProvenanceSource::Ast,
                    confidence: row.get(5)?,
                    certainty,
                    evidence: Vec::new(),
                },
                temporal: TemporalRange {
                    valid_from: row.get(7)?,
                    valid_until: row.get(8)?,
                },
            };

            if let Some(caller_node) = self.get_node(&from_id)? {
                if caller_node.kind != NodeKind::CallSite {
                    results.push((edge, caller_node));
                }
            }
        }

        Ok(results)
    }

    pub fn get_cross_file_calls(&self) -> Result<Vec<(String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT n1.file_path, n2.file_path
             FROM edges e
             JOIN nodes n1 ON e.from_node_id = n1.id
             JOIN nodes n2 ON e.to_node_id = n2.id
             WHERE e.kind = 'calls' AND n1.file_path IS NOT NULL AND n2.file_path IS NOT NULL AND n1.file_path != n2.file_path",
        )?;
        let mut rows = stmt.query([])?;
        let mut results = Vec::new();
        while let Some(row) = rows.next()? {
            let f1: String = row.get(0)?;
            let f2: String = row.get(1)?;
            results.push((f1, f2));
        }
        Ok(results)
    }

    pub fn get_uncalled_functions(&self, limit: usize) -> Result<Vec<(String, String, u32)>> {
        let mut stmt = self.conn.prepare(
            "SELECT n.name, n.file_path, COALESCE(n.start_line, 1)
             FROM nodes n
             WHERE n.kind = 'function' AND n.file_path IS NOT NULL
               AND NOT EXISTS (SELECT 1 FROM edges e WHERE e.to_node_id = n.id AND e.kind = 'calls')
             LIMIT ?1",
        )?;
        let mut rows = stmt.query(params![limit as i64])?;
        let mut results = Vec::new();
        while let Some(row) = rows.next()? {
            let name: String = row.get(0)?;
            let file: String = row.get(1)?;
            let line: u32 = row.get(2)?;
            results.push((name, file, line));
        }
        Ok(results)
    }

    // ==========================================
    // Knowledge Operations (Part I)
    // ==========================================

    pub fn insert_knowledge_item(&self, item: &KnowledgeItem) -> Result<()> {
        let type_str = serde_json::to_string(&item.knowledge_type)?.replace('"', "");
        let status_str = serde_json::to_string(&item.status)?.replace('"', "");
        let affects_str = serde_json::to_string(&item.affects)?;
        let tags_str = serde_json::to_string(&item.tags)?;

        self.conn.execute(
            "INSERT OR REPLACE INTO knowledge_items (
                id, project_id, knowledge_type, title, content,
                status, confidence, affects, tags, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                item.id,
                item.project_id,
                type_str,
                item.title,
                item.content,
                status_str,
                item.confidence,
                affects_str,
                tags_str,
                item.created_at.to_rfc3339(),
                item.updated_at.to_rfc3339()
            ],
        )?;

        // Clean up previous evidence for this item
        self.conn.execute(
            "DELETE FROM evidence WHERE target_type = 'knowledge' AND target_id = ?1",
            params![item.id],
        )?;

        // Insert evidence
        for ev in &item.evidence {
            self.conn.execute(
                "INSERT INTO evidence (target_type, target_id, evidence_type, file_path, start_line, end_line, commit_id, snippet)
                 VALUES ('knowledge', ?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    item.id,
                    ev.evidence_type,
                    ev.file,
                    ev.start_line,
                    ev.end_line,
                    ev.commit_id,
                    ev.snippet
                ],
            )?;
        }

        Ok(())
    }

    pub fn list_knowledge_items(&self, project_id: &str) -> Result<Vec<KnowledgeItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, knowledge_type, title, content, status, confidence, affects, tags, created_at, updated_at
             FROM knowledge_items WHERE project_id = ?1 ORDER BY created_at DESC",
        )?;

        let mut rows = stmt.query(params![project_id])?;
        let mut items = Vec::new();

        while let Some(row) = rows.next()? {
            items.push(self.hydrate_knowledge_item(row)?);
        }

        Ok(items)
    }

    fn hydrate_knowledge_item(&self, row: &rusqlite::Row) -> Result<KnowledgeItem> {
        let id: String = row.get(0)?;
        let type_raw: String = row.get(2)?;
        let k_type: KnowledgeType = serde_json::from_str(&format!("\"{}\"", type_raw))
            .unwrap_or(KnowledgeType::Decision);

        let status_raw: String = row.get(5)?;
        let status: KnowledgeStatus = serde_json::from_str(&format!("\"{}\"", status_raw))
            .unwrap_or(KnowledgeStatus::Active);

        let affects_raw: String = row.get(7)?;
        let affects: Vec<String> = serde_json::from_str(&affects_raw).unwrap_or_default();

        let tags_raw: String = row.get(8)?;
        let tags: Vec<String> = serde_json::from_str(&tags_raw).unwrap_or_default();

        let created_at_raw: String = row.get(9)?;
        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_raw)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now());

        let updated_at_raw: String = row.get(10)?;
        let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_raw)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now());

        // Fetch evidence
        let mut ev_stmt = self.conn.prepare(
            "SELECT evidence_type, file_path, start_line, end_line, commit_id, snippet
             FROM evidence WHERE target_type = 'knowledge' AND target_id = ?1",
        )?;
        let mut ev_rows = ev_stmt.query(params![id])?;
        let mut evidence = Vec::new();
        while let Some(ev_row) = ev_rows.next()? {
            evidence.push(Evidence {
                evidence_type: ev_row.get(0)?,
                file: ev_row.get(1)?,
                start_line: ev_row.get(2)?,
                end_line: ev_row.get(3)?,
                commit_id: ev_row.get(4)?,
                snippet: ev_row.get(5)?,
            });
        }

        Ok(KnowledgeItem {
            id,
            project_id: row.get(1)?,
            knowledge_type: k_type,
            title: row.get(3)?,
            content: row.get(4)?,
            status,
            confidence: row.get(6)?,
            evidence,
            affects,
            tags,
            created_at,
            updated_at,
        })
    }

    /// Search knowledge items by keyword matching title or content
    pub fn search_knowledge(&self, query: &str) -> Result<Vec<KnowledgeItem>> {
        self.search_knowledge_inner(None, query)
    }

    pub fn search_knowledge_for_project(
        &self,
        project_id: &str,
        query: &str,
    ) -> Result<Vec<KnowledgeItem>> {
        self.search_knowledge_inner(Some(project_id), query)
    }

    fn search_knowledge_inner(
        &self,
        project_id: Option<&str>,
        query: &str,
    ) -> Result<Vec<KnowledgeItem>> {
        let terms: Vec<String> = query
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|t| !t.is_empty())
            .map(str::to_lowercase)
            .collect();

        if terms.is_empty() {
            return self.search_knowledge_fallback(project_id, &terms);
        }

        // Try FTS5 first with token prefix matching
        let fts_query = terms
            .iter()
            .map(|t| format!("\"{t}\"*"))
            .collect::<Vec<_>>()
            .join(" OR ");

        if let Ok(items) = self.search_knowledge_fts(project_id, &fts_query) {
            if !items.is_empty() {
                return Ok(items);
            }
        }

        // Fallback to substring matching if FTS5 yields 0 or fails
        self.search_knowledge_fallback(project_id, &terms)
    }

    fn search_knowledge_fts(
        &self,
        project_id: Option<&str>,
        fts_query: &str,
    ) -> Result<Vec<KnowledgeItem>> {
        let sql = "SELECT k.id, k.project_id, k.knowledge_type, k.title, k.content, k.status, k.confidence, k.affects, k.tags, k.created_at, k.updated_at
            FROM knowledge_items k
            JOIN knowledge_fts f ON k.id = f.item_id
            WHERE f.knowledge_fts MATCH ?1
              AND k.status NOT IN ('archived', 'superseded')
              AND (?2 IS NULL OR k.project_id = ?2)
            ORDER BY bm25(knowledge_fts, 5.0, 2.0, 1.0) ASC, k.confidence DESC, k.id ASC";

        let mut stmt = self.conn.prepare(sql)?;
        let mut rows = stmt.query(params![fts_query, project_id])?;
        let mut items = Vec::new();

        while let Some(row) = rows.next()? {
            items.push(self.hydrate_knowledge_item(row)?);
        }
        Ok(items)
    }

    fn search_knowledge_fallback(
        &self,
        project_id: Option<&str>,
        terms: &[String],
    ) -> Result<Vec<KnowledgeItem>> {
        let filter = if terms.is_empty() {
            String::new()
        } else {
            format!(" AND ({})", terms.iter().enumerate().map(|(i, _)| {
                let p = i + 2;
                format!("instr(lower(title), ?{p}) > 0 OR instr(lower(content), ?{p}) > 0 OR instr(lower(tags), ?{p}) > 0")
            }).collect::<Vec<_>>().join(" OR "))
        };
        let sql = format!("SELECT id, project_id, knowledge_type, title, content, status, confidence, affects, tags, created_at, updated_at
            FROM knowledge_items WHERE status NOT IN ('archived', 'superseded') AND (?1 IS NULL OR project_id = ?1){filter} ORDER BY confidence DESC, id ASC");
        let mut stmt = self.conn.prepare(&sql)?;
        let mut values = vec![project_id.map_or(SqlValue::Null, |p| SqlValue::Text(p.to_owned()))];
        values.extend(terms.iter().cloned().map(SqlValue::Text));
        let mut rows = stmt.query(params_from_iter(values.iter()))?;
        let mut items = Vec::new();

        while let Some(row) = rows.next()? {
            items.push(self.hydrate_knowledge_item(row)?);
        }

        let score = |item: &KnowledgeItem| {
            let title = item.title.to_lowercase();
            let body = format!("{} {}", item.content, item.tags.join(" ")).to_lowercase();
            terms
                .iter()
                .map(|term| {
                    usize::from(title.contains(term)) * 2 + usize::from(body.contains(term))
                })
                .sum::<usize>()
        };
        if !terms.is_empty() {
            items.retain(|item| score(item) > 0);
            items.sort_by_key(|item| std::cmp::Reverse(score(item)));
        }
        Ok(items)
    }
}
