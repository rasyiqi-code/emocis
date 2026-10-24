import { expect, test } from "bun:test";
import { displayScore, parseAudit, parseHealth } from "../src/contracts";
import { graphRows } from "../src/treeViews/healthRows";

const measuredHealth = {
  overall_score: null, modularity: null, cohesion: null, coupling: null, boundary_integrity: null,
  status: "partial_static_analysis", explanation: "Partial indexed graph",
  graph: {
    indexed_files: 3, indexed_nodes: 12, indexed_functions: 4, call_relationships: 5,
    resolved_calls: 4, unresolved_calls: 1, internal_file_calls: 1, cross_file_calls: 3,
    resolution_percent: 80, cross_file_percent: 75,
    dependency_cycles: [["a.ts", "b.ts", "c.ts"]],
    files: [{ file: "a.ts", fan_in: 1, fan_out: 1 }],
  },
};

test("audit preserves violations from backend object", () => {
  const result = parseAudit(JSON.stringify({
    total_violations: 1,
    violations: [{
      title: "Drift",
      description: "Missing symbol",
      location: "src/index.ts:42",
      recommendation: "Remove unused import or update docs",
      severity: "Medium",
    }],
  }));
  expect(result.total_violations).toBe(1);
  expect(result.violations[0].title).toBe("Drift");
  expect(result.violations[0].recommendation).toBe("Remove unused import or update docs");
  expect(result.violations[0].severity).toBe("Medium");
});

test("invalid or unavailable audit cannot masquerade as a clean result", () => {
  for (const raw of ["", "[]", "null", "Error accessing store", '{"total_violations":0,"violations":[{}]}']) {
    expect(() => parseAudit(raw)).toThrow();
  }
});

test("health reads backend field names and distinguishes unavailable scores", () => {
  const result = parseHealth(JSON.stringify({ overall_score: null, modularity: null, cohesion: null, coupling: null, boundary_integrity: null, status: "experimental", explanation: "Not measured" }));
  expect(displayScore(result.modularity)).toBe("Not measured");
  expect(displayScore(0)).toBe("0/100");
  expect(displayScore(85)).toBe("85/100");
  expect(() => parseHealth('{"modularity_score":85}')).toThrow();
});

test("measured graph values reach the sidebar without invented quality scores", () => {
  const result = parseHealth(JSON.stringify(measuredHealth));
  expect(result.overall_score).toBeNull();
  const rows = graphRows(result.graph!);
  expect(rows.find(row => row.label === "Resolved Relationships")?.value).toBe("4/5 (80.0%)");
  expect(rows.find(row => row.label === "Dependency Cycles")?.value).toBe("1");
  expect(rows.find(row => row.label === "a.ts")?.value).toBe("1 in / 1 out");
});

test("malformed graph counts and ratios are rejected", () => {
  for (const patch of [{ unresolved_calls: -1 }, { resolution_percent: 101 }, { resolved_calls: 8 }, { files: [{}] }, { dependency_cycles: [null] }]) {
    expect(() => parseHealth(JSON.stringify({ ...measuredHealth, graph: { ...measuredHealth.graph, ...patch } }))).toThrow();
  }
});

test("no relationships displays unavailable ratios instead of 100 percent", () => {
  const graph = { ...measuredHealth.graph, call_relationships: 0, resolved_calls: 0, unresolved_calls: 0,
    internal_file_calls: 0, cross_file_calls: 0, resolution_percent: null, cross_file_percent: null,
    dependency_cycles: [], files: [] };
  const result = parseHealth(JSON.stringify({ ...measuredHealth, graph }));
  expect(graphRows(result.graph!).find(row => row.label === "Resolved Relationships")?.value).toContain("N/A");
});
