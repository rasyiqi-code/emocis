---
name: emocis
description: "EMOCIS Engineering Memory & Codebase Intelligence CLI. Use to inspect code logic graphs (CCLG), audit architecture drift, check blast radius/impact of functions, check modularity/health scores, and record failed attempts."
---

# EMOCIS — Engineering Memory & Codebase Intelligence

EMOCIS is a deterministic Codebase Logic Graph (CCLG) and Engineering Memory system.
Use EMOCIS CLI commands in the terminal to avoid blind edits, prevent architectural drift, check ripple effects, and maintain codebase health.

## 🚀 Key Workflows for AI Coding Agents

### 1. Before Modifying Existing Functions (Impact Analysis)
Before refactoring or altering a function's signature, check who calls it across the entire repository:
```bash
project context <symbol_or_function_name>
```
* Identifies direct and transitive callers across frontend & backend (e.g. Vue components, Pinia stores, Tauri IPC handlers).
* Prevents breaking callers in distant files.

### 2. After Generating or Editing Code (Architecture Verification)
Always re-index and run an audit to ensure no circular dependencies or dead logic were introduced:
```bash
project map && project audit
```
* `project map`: Incrementally parses ASTs and updates the local logic graph in `.emocis/emocis.db`. Use `--force` to re-index all files.
* `project audit`: Detects circular dependencies (Kosaraju SCC), unreferenced private code, and doc drift.

### 3. Checking Architecture Health & Quality Scores
Check overall codebase architecture scores:
```bash
project health
```
Outputs calibrated metrics (0-100):
* **Overall Health Score**: Aggregated codebase architectural health.
* **Modularity Score**: Evaluates module segregation and penalizes cyclic dependencies.
* **Cohesion Score**: Percentage of calls collaborating within the same subsystem.
* **Coupling Score**: Measures instability and cross-file entanglement.
* **Boundary Integrity**: Clean separation across architectural layers.
* **Technical Debt Index**: Low / Moderate / High maintainability rating.

### 4. Preventing Repeated Mistakes (Failed Attempts Memory)
If an architectural approach or fix fails (e.g., causes SQLite lock, Vue reactivity loop, or bundle crash), record it so neither you nor future AI sessions repeat the mistake:
```bash
project failed --title "Deadlock SQLite saat background sync" --description "Jangan memanggil syncService.syncAll di dalam transaksi local_db karena menyebabkan database terkunci."
```

### 5. Searching Architectural Knowledge & Rules
Search decisions (`@decision`), patterns (`@pattern`), and warnings (`@warning`) documented across code:
```bash
project search <keyword>
```

---

## 📋 Quick Command Reference

| Command | Purpose |
|---|---|
| `project map [--force]` | Scan & index codebase into CCLG logic graph |
| `project audit` | Detect circular dependencies and drift |
| `project health` | Display architecture scores & relationship ratios |
| `project context <symbol>` | Inspect blast radius / callers of a function |
| `project search <term>` | Search engineering decisions & memory |
| `project failed` | Record or list failed technical attempts |
| `project snapshot` | Create baseline snapshot for drift comparison |
| `clg` | Interactive CLI code logic graph explorer |
| `emocis mcp` | Run stdio Model Context Protocol (MCP) server |

## 🛡️ Vibecoding Guardrails
1. **Never edit blindly**: Always inspect blast radius with `project context <symbol>`.
2. **Never leave broken cycles**: Run `project audit` before concluding your turn.
3. **Record failures**: Use `project failed` when an idea proves invalid to preserve memory.
