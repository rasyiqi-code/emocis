# EMOCIS

Local engineering memory and experimental static code analysis for TypeScript/JavaScript, Rust, and Python.

## Build and index

```sh
cargo build --workspace --release
# Run from the repository you want to index:
/path/to/EMOCIS/target/release/project map
```

The index lives in `.emocis/emocis.db` under the current working directory. Mapping replaces source-derived knowledge and graph data in a transaction. Manual `project failed` records survive reindexing. A read or adapter failure rolls back the previous index. Symbolic links are skipped.

Existing indexes need one successful `project map` to acquire new import links. Records already deleted by an older version cannot be reconstructed without a backup or their original source.

## Available capabilities and limits

- **Memory search:** project-scoped ranked keyword matching across titles, content, and tags. This is not semantic/embedding search. Archived and superseded records are excluded.
- **Context:** keyword-matched constraints, business rules, decisions, architecture notes, warnings, and failures. Markdown output is bounded to 8,000 characters and explicitly marks truncation. Token counts are estimates; no fixed savings percentage is claimed.
- **Graph:** TypeScript/JavaScript function declarations, variable-bound arrow functions and function expressions, and direct default function/arrow exports are indexed. Relative named/default imports, aliases, namespace function calls, explicit re-exports, and local export aliases resolve through public export bindings after indexing. Type-only and private bindings are excluded. Export cycles terminate safely. Export-star, package/path aliases, dynamic dispatch, and cross-file Rust/Python calls remain unresolved. This is partial static analysis; impact traversal stops at depth five.
- **Refactoring:** function counts use source locations. Physical/nonblank line counts and identifier frequencies are size signals. Cyclomatic complexity is the maximum AST branch count plus one per parsed function in TypeScript/JavaScript, Rust, or Python; parse errors and unsupported languages display N/A. Review thresholds are heuristic and do not assert cohesion or design quality.
- **Duplication:** candidate similarity based on normalized AST shapes (for supported parsers), called-function name sets, or function names. AST shapes ignore identifiers and literals, so these candidates still require review of order, data flow, types, side effects, and outputs. No automatic consolidation is performed.
- **Health:** measured graph data is available in the CLI, MCP, and extension: indexed files/nodes/functions, unique caller-target relationships, resolved/unresolved relationships, cross-file relationships, file fan-in/fan-out, and strongly connected groups of files. Longer cycles (including three or more files) are detected and reported by the audit. All measurements are project-scoped. Overall architectural quality and debt scores remain uncalibrated (`null` / unavailable).

### Measurement definitions

- Counts use unique caller-target pairs, excluding callsite mirror edges. Repeated calls from one function to the same target count once.
- Resolution percentage = resolved pairs / all indexed pairs × 100. Unresolved pairs include external libraries and unsupported syntax; this is not runtime coverage.
- Cross-file percentage = cross-file resolved pairs / all resolved pairs × 100. This measures a structural relationship, not whether coupling is good or bad.
- Fan-in/fan-out count distinct other files connected by resolved calls. Import-only dependencies are not counted.
- Dependency cycle groups are strongly connected components containing at least two files; listed files are not an ordered execution trace.
- Ratios with no denominator are `null` / N/A rather than a perfect score. An empty graph is marked `not_indexed`.

Refresh in the extension reads the existing index. Run `project map` to incorporate source changes.

## Verification

```sh
cargo test --workspace
cd vscode-extension
bun run typecheck
bun test
bun run build
```

Regression tests use isolated temporary repositories, including Unicode comments, quoted directory names, rollback, manual memory retention, imported calls, file-scoped refactoring, default/arrow/re-export resolution, long dependency cycles, graph measurement formulas, project isolation, and MCP response contracts.
