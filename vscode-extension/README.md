# EMOCIS — VS Code & Cursor Extension

> **Engineering Memory & Codebase Intelligence System (Local-first Engine & CCLG)**

EMOCIS transforms how software engineers and AI coding assistants navigate and evolve complex software codebases by combining **Engineering Memory** (*why & how software evolved*) and **Canonical Code Logic Graph** (*how software behaves*).

## Key Features

- **Conceptual Table of Contents (TOC)**:
  Browse and search past architectural decisions (`@decision`), constraints (`@constraint`), warnings (`@warning`), business rules, and failed attempts directly inside the VS Code sidebar.
- **Interactive Logic Graph (CCLG)**:
  Inspect indexed functions, direct callers, callees, execution paths, and impact radius inside an interactive dark-mode graph canvas.
- **Impact Analysis CodeLens**:
  Inline badges above function definitions showing caller counts and allowing 1-click impact analysis before refactoring.
- **Static Graph Metrics & Heuristic Audit**:
  Inspect measured file/function counts, resolved and unresolved call relationships, cross-file ratios, file fan-in/fan-out, and dependency cycle groups. Overall quality scores remain uncalibrated. Engine errors are displayed as Unavailable.
- **Minimal Context Package for AI**:
  Generate keyword-matched context packages with a bounded output and explicit truncation notices. Token savings and semantic relevance are not guaranteed.

## Current analysis limits

- The graph is partial static analysis. Relative TypeScript/JavaScript named/default imports, namespace function calls, and explicit re-exports are linked through public export bindings after indexing. Variable-bound arrow functions and function expressions are indexed. Export-star, package aliases, dynamic dispatch, and cross-file Rust/Python resolution are not resolved by this linker. Missing callers are not proof of dead code.
- Run `project map` after source changes. Refresh reloads the existing index; it does not reindex files.
- Search matches keywords, not embeddings. Duplication candidates compare normalized AST shapes when supported, then call-name sets or function names; they do not establish behavioral equivalence.
- Health scores are nullable in the MCP contract. Audit returns `{total_violations, violations}`, not an array.

## Development checks

```sh
bun run typecheck
bun test
bun run build
```

Measured values describe unique caller-target pairs in the indexed graph, excluding callsite duplicates. Resolution percentage is resolved pairs / all pairs; cross-file percentage is cross-file pairs / resolved pairs. Empty denominators display N/A. Fan-in/out count distinct neighboring files. These are structural measurements, not calibrated quality scores or runtime coverage.
