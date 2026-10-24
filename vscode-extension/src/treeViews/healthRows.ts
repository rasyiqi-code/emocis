import { GraphMeasurements } from "../contracts";

export function graphRows(graph: GraphMeasurements): Array<{ label: string; value: string; details: string }> {
  const percent = (value: number | null) => value === null ? "N/A (no relationships)" : `${value.toFixed(1)}%`;
  const rows = [
    { label: "Indexed Files", value: String(graph.indexed_files), details: "Files represented in the static code graph" },
    { label: "Indexed Nodes", value: String(graph.indexed_nodes), details: "All project nodes, including files, symbols, and callsites" },
    { label: "Indexed Functions", value: String(graph.indexed_functions), details: "Function and method nodes in this project" },
    { label: "Resolved Relationships", value: `${graph.resolved_calls}/${graph.call_relationships} (${percent(graph.resolution_percent)})`, details: "Unique caller-target pairs. Callsite mirror edges are excluded; this is not runtime coverage." },
    { label: "Unresolved Relationships", value: String(graph.unresolved_calls), details: "Includes external libraries and unsupported call targets, not necessarily errors" },
    { label: "Cross-file Relationships", value: `${graph.cross_file_calls} (${percent(graph.cross_file_percent)})`, details: "Cross-file pairs divided by resolved pairs; a structural ratio, not a quality score" },
    { label: "Dependency Cycles", value: String(graph.dependency_cycles.length), details: "Strongly connected groups of two or more files, including cycles longer than two files" },
  ];
  for (const file of [...graph.files].filter(f => f.fan_in + f.fan_out > 0)
      .sort((a, b) => (b.fan_in + b.fan_out) - (a.fan_in + a.fan_out) || a.file.localeCompare(b.file)).slice(0, 5)) {
    rows.push({ label: file.file, value: `${file.fan_in} in / ${file.fan_out} out`, details: "Distinct other files calling into this file (fan-in) / called by this file (fan-out)" });
  }
  return rows;
}
