-- EMOCIS Canonical Schema (PRD Part I & Part II Section 38)
CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    root_path TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS snapshots (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    commit_hash TEXT NOT NULL,
    branch TEXT,
    author TEXT,
    message TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS nodes (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    kind TEXT NOT NULL,
    category TEXT NOT NULL,
    name TEXT NOT NULL,
    language TEXT,
    file_path TEXT,
    start_line INTEGER,
    start_column INTEGER,
    end_line INTEGER,
    end_column INTEGER,
    qualified_name TEXT NOT NULL,
    signature TEXT,
    status TEXT NOT NULL,
    confidence REAL NOT NULL,
    certainty TEXT NOT NULL,
    metadata TEXT,
    FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_nodes_project ON nodes(project_id);
CREATE INDEX IF NOT EXISTS idx_nodes_kind ON nodes(kind);
CREATE INDEX IF NOT EXISTS idx_nodes_file ON nodes(file_path);
CREATE INDEX IF NOT EXISTS idx_nodes_name ON nodes(name);
CREATE INDEX IF NOT EXISTS idx_nodes_qualified ON nodes(qualified_name);

CREATE TABLE IF NOT EXISTS edges (
    id TEXT PRIMARY KEY,
    from_node_id TEXT NOT NULL,
    to_node_id TEXT NOT NULL,
    kind TEXT NOT NULL,
    category TEXT NOT NULL,
    qualifier TEXT,
    confidence REAL NOT NULL,
    certainty TEXT NOT NULL,
    valid_from TEXT NOT NULL,
    valid_until TEXT,
    FOREIGN KEY(from_node_id) REFERENCES nodes(id) ON DELETE CASCADE,
    FOREIGN KEY(to_node_id) REFERENCES nodes(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_edges_from ON edges(from_node_id);
CREATE INDEX IF NOT EXISTS idx_edges_to ON edges(to_node_id);
CREATE INDEX IF NOT EXISTS idx_edges_kind ON edges(kind);

CREATE TABLE IF NOT EXISTS evidence (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    target_type TEXT NOT NULL, -- 'node', 'edge', 'knowledge'
    target_id TEXT NOT NULL,
    evidence_type TEXT NOT NULL,
    file_path TEXT,
    start_line INTEGER,
    end_line INTEGER,
    commit_id TEXT,
    snippet TEXT
);

CREATE INDEX IF NOT EXISTS idx_evidence_target ON evidence(target_type, target_id);

CREATE TABLE IF NOT EXISTS knowledge_items (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    knowledge_type TEXT NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    status TEXT NOT NULL,
    confidence REAL NOT NULL,
    affects TEXT, -- JSON array of node IDs or file names
    tags TEXT,    -- JSON array of strings
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_knowledge_project ON knowledge_items(project_id);
CREATE INDEX IF NOT EXISTS idx_knowledge_type ON knowledge_items(knowledge_type);
CREATE INDEX IF NOT EXISTS idx_knowledge_status ON knowledge_items(status);
