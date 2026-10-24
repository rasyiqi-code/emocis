export interface AuditViolation {
  title: string;
  description: string;
  location?: string;
  recommendation?: string;
  severity?: string;
}

export interface AuditReport {
  total_violations: number;
  violations: AuditViolation[];
}

export interface HealthMetrics {
  graph?: GraphMeasurements;
  overall_score: number | null;
  modularity: number | null;
  cohesion: number | null;
  coupling: number | null;
  boundary_integrity: number | null;
  status: string;
  explanation: string;
}

export interface GraphMeasurements {
  indexed_files: number;
  indexed_nodes: number;
  indexed_functions: number;
  call_relationships: number;
  resolved_calls: number;
  unresolved_calls: number;
  internal_file_calls: number;
  cross_file_calls: number;
  resolution_percent: number | null;
  cross_file_percent: number | null;
  dependency_cycles: string[][];
  files: Array<{ file: string; fan_in: number; fan_out: number }>;
}

export function parseAudit(raw: string): AuditReport {
  const value = JSON.parse(raw);
  if (!value || !Array.isArray(value.violations) ||
      !Number.isInteger(value.total_violations) || value.total_violations !== value.violations.length) {
    throw new Error("Invalid audit response");
  }
  return value;
}

export function parseHealth(raw: string): HealthMetrics {
  const value = JSON.parse(raw);
  if (!value || !["overall_score", "modularity", "cohesion", "coupling", "boundary_integrity"]
      .every(key => value[key] === null || (typeof value[key] === "number" && Number.isFinite(value[key])))) {
    throw new Error("Invalid health response");
  }
  if (value.graph !== undefined) {
    const graph = value.graph;
    const count = (v: unknown) => typeof v === "number" && Number.isSafeInteger(v) && v >= 0;
    const ratio = (v: unknown) => v === null || (typeof v === "number" && Number.isFinite(v) && v >= 0 && v <= 100);
    if (!graph || !["indexed_files", "indexed_nodes", "indexed_functions", "call_relationships", "resolved_calls", "unresolved_calls", "internal_file_calls", "cross_file_calls"].every(k => count(graph[k])) ||
        graph.resolved_calls + graph.unresolved_calls !== graph.call_relationships ||
        graph.internal_file_calls + graph.cross_file_calls > graph.resolved_calls ||
        !ratio(graph.resolution_percent) || !ratio(graph.cross_file_percent) ||
        !Array.isArray(graph.dependency_cycles) || !graph.dependency_cycles.every((cycle: unknown) => Array.isArray(cycle) && cycle.length >= 2 && cycle.every(file => typeof file === "string")) ||
        !Array.isArray(graph.files) || !graph.files.every((file: any) => file && typeof file.file === "string" && count(file.fan_in) && count(file.fan_out))) {
      throw new Error("Invalid graph measurements");
    }
  }
  return value;
}

export function displayScore(score: number | null): string {
  return score === null ? "Not measured" : `${score}/100`;
}
