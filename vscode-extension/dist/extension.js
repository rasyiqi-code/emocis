var __create = Object.create;
var __getProtoOf = Object.getPrototypeOf;
var __defProp = Object.defineProperty;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __hasOwnProp = Object.prototype.hasOwnProperty;
function __accessProp(key) {
  return this[key];
}
var __toESMCache_node;
var __toESMCache_esm;
var __toESM = (mod, isNodeMode, target) => {
  var canCache = mod != null && typeof mod === "object";
  if (canCache) {
    var cache = isNodeMode ? __toESMCache_node ??= new WeakMap : __toESMCache_esm ??= new WeakMap;
    var cached = cache.get(mod);
    if (cached)
      return cached;
  }
  target = mod != null ? __create(__getProtoOf(mod)) : {};
  const to = isNodeMode || !mod || !mod.__esModule || !__hasOwnProp.call(mod, "default") ? __defProp(target, "default", { value: mod, enumerable: true }) : target;
  if (mod && typeof mod === "object" || typeof mod === "function") {
    for (let key of __getOwnPropNames(mod))
      if (!__hasOwnProp.call(to, key))
        __defProp(to, key, {
          get: __accessProp.bind(mod, key),
          enumerable: true
        });
  }
  if (canCache)
    cache.set(mod, to);
  return to;
};
var __toCommonJS = (from) => {
  var entry = (__moduleCache ??= new WeakMap).get(from), desc;
  if (entry)
    return entry;
  entry = __defProp({}, "__esModule", { value: true });
  if (from && typeof from === "object" || typeof from === "function") {
    for (var key of __getOwnPropNames(from))
      if (!__hasOwnProp.call(entry, key))
        __defProp(entry, key, {
          get: __accessProp.bind(from, key),
          enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable
        });
  }
  __moduleCache.set(from, entry);
  return entry;
};
var __moduleCache;
var __returnValue = (v) => v;
function __exportSetter(name, newValue) {
  this[name] = __returnValue.bind(null, newValue);
}
var __export = (target, all) => {
  for (var name in all)
    __defProp(target, name, {
      get: all[name],
      enumerable: true,
      configurable: true,
      set: __exportSetter.bind(all, name)
    });
};

// src/extension.ts
var exports_extension = {};
__export(exports_extension, {
  activate: () => activate,
  deactivate: () => deactivate
});
module.exports = __toCommonJS(exports_extension);
var vscode11 = __toESM(require("vscode"));

// src/contracts.ts
function parseAudit(raw) {
  const value = JSON.parse(raw);
  if (!value || !Array.isArray(value.violations) || !Number.isInteger(value.total_violations) || value.total_violations !== value.violations.length) {
    throw new Error("Invalid audit response");
  }
  return value;
}
function parseHealth(raw) {
  const value = JSON.parse(raw);
  if (!value || !["overall_score", "modularity", "cohesion", "coupling", "boundary_integrity"].every((key) => value[key] === null || typeof value[key] === "number" && Number.isFinite(value[key]))) {
    throw new Error("Invalid health response");
  }
  if (value.graph !== undefined) {
    const graph = value.graph;
    const count = (v) => typeof v === "number" && Number.isSafeInteger(v) && v >= 0;
    const ratio = (v) => v === null || typeof v === "number" && Number.isFinite(v) && v >= 0 && v <= 100;
    if (!graph || !["indexed_files", "indexed_nodes", "indexed_functions", "call_relationships", "resolved_calls", "unresolved_calls", "internal_file_calls", "cross_file_calls"].every((k) => count(graph[k])) || graph.resolved_calls + graph.unresolved_calls !== graph.call_relationships || graph.internal_file_calls + graph.cross_file_calls > graph.resolved_calls || !ratio(graph.resolution_percent) || !ratio(graph.cross_file_percent) || !Array.isArray(graph.dependency_cycles) || !graph.dependency_cycles.every((cycle) => Array.isArray(cycle) && cycle.length >= 2 && cycle.every((file) => typeof file === "string")) || !Array.isArray(graph.files) || !graph.files.every((file) => file && typeof file.file === "string" && count(file.fan_in) && count(file.fan_out))) {
      throw new Error("Invalid graph measurements");
    }
  }
  return value;
}
function displayScore(score) {
  return score === null ? "Not measured" : `${score}/100`;
}

// src/mcpClient.ts
var import_child_process = require("child_process");
var path = __toESM(require("path"));
var fs = __toESM(require("fs"));

class EmocisClient {
  workspaceRoot;
  process = null;
  requestId = 0;
  pendingRequests = new Map;
  buffer = "";
  constructor(workspaceRoot) {
    this.workspaceRoot = workspaceRoot;
    this.initProcess();
  }
  initProcess() {
    const homeDir = process.env.HOME || "";
    const cargoBinary = path.join(homeDir, ".cargo", "bin", "emocis");
    const localTargetBinary = path.join(this.workspaceRoot, "target", "debug", "emocis");
    const releaseTargetBinary = path.join(this.workspaceRoot, "target", "release", "emocis");
    let executable = "emocis";
    let args = ["mcp"];
    if (fs.existsSync(releaseTargetBinary)) {
      executable = releaseTargetBinary;
    } else if (fs.existsSync(localTargetBinary)) {
      executable = localTargetBinary;
    } else if (fs.existsSync(cargoBinary)) {
      executable = cargoBinary;
    } else {
      const bunCandidates = [
        path.join(homeDir, ".bun", "bin", "bun"),
        "/usr/local/bin/bun",
        "/usr/bin/bun"
      ];
      const bunBin = bunCandidates.find((b) => fs.existsSync(b)) || "bun";
      const mcpScript = path.join(this.workspaceRoot, "bin", "mcp.js");
      if (fs.existsSync(mcpScript)) {
        executable = bunBin;
        args = [mcpScript];
      }
    }
    const env = {
      ...process.env,
      PATH: `${homeDir}/.cargo/bin:${homeDir}/.bun/bin:${process.env.PATH || ""}`,
      RUST_LOG: "error"
    };
    try {
      this.process = import_child_process.spawn(executable, args, {
        cwd: this.workspaceRoot,
        env
      });
      this.process.stdout.on("data", (chunk) => {
        this.buffer += chunk.toString("utf8");
        const lines = this.buffer.split(`
`);
        this.buffer = lines.pop() || "";
        for (const line of lines) {
          const trimmed = line.trim();
          if (!trimmed || !trimmed.startsWith("{"))
            continue;
          try {
            const json = JSON.parse(trimmed);
            if (json.id !== undefined && this.pendingRequests.has(json.id)) {
              const { resolve, reject } = this.pendingRequests.get(json.id);
              this.pendingRequests.delete(json.id);
              if (json.error) {
                reject(new Error(json.error.message || "MCP Error"));
              } else {
                resolve(json.result);
              }
            }
          } catch {}
        }
      });
      this.process.on("error", (err) => {
        console.error("[EMOCIS Client] Subprocess error:", err);
        for (const [, req] of this.pendingRequests) {
          req.reject(err);
        }
        this.pendingRequests.clear();
      });
      this.process.on("exit", (code) => {
        console.log(`[EMOCIS Client] Subprocess exited with code ${code}`);
        this.process = null;
        for (const [, req] of this.pendingRequests)
          req.reject(new Error(`Engine exited: ${code}`));
        this.pendingRequests.clear();
      });
    } catch (e) {
      console.error("[EMOCIS Client] Failed to spawn:", e);
      this.process = null;
    }
  }
  async callTool(name, args = {}) {
    if (!this.process) {
      this.initProcess();
    }
    const id = ++this.requestId;
    const payload = JSON.stringify({
      jsonrpc: "2.0",
      id,
      method: "tools/call",
      params: {
        name,
        arguments: args
      }
    }) + `
`;
    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        if (this.pendingRequests.has(id)) {
          this.pendingRequests.delete(id);
          reject(new Error("EMOCIS request timed out"));
        }
      }, 1e4);
      this.pendingRequests.set(id, {
        resolve: (result) => {
          clearTimeout(timeout);
          const text = result?.content?.[0]?.text ?? "";
          if (result?.isError) {
            reject(new Error(text || "EMOCIS tool failed"));
            return;
          }
          resolve(text);
        },
        reject: (error) => {
          clearTimeout(timeout);
          reject(error);
        }
      });
      this.process?.stdin.write(payload);
    });
  }
  async search(query) {
    const raw = await this.callTool("emocis_search", { query });
    return JSON.parse(raw);
  }
  async getContext(intent) {
    const raw = await this.callTool("emocis_get_context", { intent });
    try {
      return JSON.parse(raw);
    } catch {
      return raw;
    }
  }
  async getImpact(symbol) {
    const raw = await this.callTool("emocis_impact", { symbol });
    return JSON.parse(raw);
  }
  async getCalls(fn) {
    const raw = await this.callTool("emocis_calls", { function: fn });
    return JSON.parse(raw);
  }
  async getCallers(fn) {
    const raw = await this.callTool("emocis_callers", { function: fn });
    return JSON.parse(raw);
  }
  async getHealth() {
    return parseHealth(await this.callTool("emocis_health"));
  }
  async getAudit() {
    return parseAudit(await this.callTool("emocis_audit"));
  }
  dispose() {
    if (this.process) {
      this.process.kill();
      this.process = null;
    }
  }
}

// src/treeViews/memoryTreeProvider.ts
var vscode = __toESM(require("vscode"));
function isDocumentationItem(item) {
  if (item.evidence && item.evidence.some((e) => {
    const f = e.file?.toLowerCase() || "";
    return f.endsWith(".md") || e.evidence_type === "documentation";
  })) {
    return true;
  }
  if (item.tags && item.tags.some((t) => t.toLowerCase() === "markdown")) {
    return true;
  }
  if (item.affects && item.affects.some((a) => a.toLowerCase().endsWith(".md"))) {
    return true;
  }
  return false;
}

class MemoryItem extends vscode.TreeItem {
  label;
  collapsibleState;
  rootGroup;
  category;
  data;
  filePath;
  line;
  constructor(label, collapsibleState, rootGroup, category, data, filePath, line) {
    super(label, collapsibleState);
    this.label = label;
    this.collapsibleState = collapsibleState;
    this.rootGroup = rootGroup;
    this.category = category;
    this.data = data;
    this.filePath = filePath;
    this.line = line;
    if (data) {
      this.contextValue = "memoryItem";
      this.tooltip = `${data.title}

${data.content}

Tags: ${data.tags.join(", ")}`;
      let file;
      let startLine = 1;
      if (data.evidence && data.evidence.length > 0) {
        for (const ev of data.evidence) {
          if (ev.file) {
            file = ev.file;
            startLine = ev.start_line ?? 1;
            break;
          }
        }
      }
      if (!file && data.affects && data.affects.length > 0) {
        file = data.affects[0];
      }
      if (file) {
        this.description = `${file}:${startLine}`;
        this.command = {
          command: "emocis.openLocation",
          title: "Open Location",
          arguments: [file, startLine]
        };
      } else {
        this.description = data.affects.join(", ");
      }
      const tags = data.tags?.map((t) => t.toLowerCase()) || [];
      const isBugfix = tags.includes("bugfix") || tags.includes("bug fix") || data.title.startsWith("BUG FIX");
      if (isBugfix) {
        this.iconPath = new vscode.ThemeIcon("debug-restart", new vscode.ThemeColor("charts.red"));
      } else {
        switch (data.knowledge_type?.toLowerCase()) {
          case "decision":
            this.iconPath = new vscode.ThemeIcon("check", new vscode.ThemeColor("charts.blue"));
            break;
          case "constraint":
            this.iconPath = new vscode.ThemeIcon("lock", new vscode.ThemeColor("charts.yellow"));
            break;
          case "warning":
          case "caveat":
            this.iconPath = new vscode.ThemeIcon("warning", new vscode.ThemeColor("charts.orange"));
            break;
          case "business_rule":
          case "businessrule":
            this.iconPath = new vscode.ThemeIcon("law", new vscode.ThemeColor("charts.purple"));
            break;
          case "failed_attempt":
            this.iconPath = new vscode.ThemeIcon("error", new vscode.ThemeColor("charts.red"));
            break;
          case "architecture":
            this.iconPath = new vscode.ThemeIcon("organization", new vscode.ThemeColor("charts.green"));
            break;
          default:
            this.iconPath = new vscode.ThemeIcon("bookmark", new vscode.ThemeColor("charts.green"));
        }
      }
    } else if (rootGroup && !category) {
      if (rootGroup === "docs") {
        this.iconPath = new vscode.ThemeIcon("book", new vscode.ThemeColor("charts.blue"));
      } else {
        this.iconPath = new vscode.ThemeIcon("code", new vscode.ThemeColor("charts.green"));
      }
    } else {
      this.iconPath = new vscode.ThemeIcon("folder");
    }
  }
}

class MemoryTreeProvider {
  _onDidChangeTreeData = new vscode.EventEmitter;
  onDidChangeTreeData = this._onDidChangeTreeData.event;
  client;
  cachedItems = [];
  constructor(client) {
    this.client = client;
  }
  refresh() {
    this._onDidChangeTreeData.fire();
  }
  getTreeItem(element) {
    return element;
  }
  async getChildren(element) {
    if (!element) {
      try {
        const results = await this.client.search("");
        this.cachedItems = Array.isArray(results) ? results : [];
      } catch (e) {
        console.error("Error fetching memory items:", e);
        this.cachedItems = [];
      }
      const docItems = this.cachedItems.filter(isDocumentationItem);
      const codeItems = this.cachedItems.filter((item) => !isDocumentationItem(item));
      return [
        new MemoryItem(`\uD83D\uDCC4 Project Documentation & Specs (${docItems.length})`, docItems.length > 0 ? vscode.TreeItemCollapsibleState.Expanded : vscode.TreeItemCollapsibleState.None, "docs"),
        new MemoryItem(`\uD83D\uDCBB Source Code Annotations (${codeItems.length})`, codeItems.length > 0 ? vscode.TreeItemCollapsibleState.Expanded : vscode.TreeItemCollapsibleState.None, "code")
      ];
    }
    const matchesCategory = (item, type) => {
      const t = item.knowledge_type?.toLowerCase() || "";
      const tags = item.tags?.map((x) => x.toLowerCase()) || [];
      if (type === "bugfix") {
        return tags.includes("bugfix") || tags.includes("bug fix") || item.title.startsWith("BUG FIX");
      }
      if (type === "warning") {
        const isBugfix = tags.includes("bugfix") || tags.includes("bug fix") || item.title.startsWith("BUG FIX");
        return !isBugfix && (t === "warning" || tags.includes("warning"));
      }
      return t === type || tags.includes(type);
    };
    if (element.rootGroup && !element.category) {
      const isDocs = element.rootGroup === "docs";
      const pool = this.cachedItems.filter((item) => isDocs ? isDocumentationItem(item) : !isDocumentationItem(item));
      const categories = isDocs ? [
        { label: "Architecture & System Design", type: "architecture" },
        { label: "Architectural Decisions (ADR)", type: "decision" },
        { label: "Constraints & Security Specs", type: "constraint" },
        { label: "Business Rules & Policies", type: "business_rule" },
        { label: "Bug Reports & Caveats", type: "warning" },
        { label: "Worklog & Changelogs", type: "worklog" }
      ] : [
        { label: "Decisions & Rationale", type: "decision" },
        { label: "System Constraints & Security", type: "constraint" },
        { label: "Business Logic Rules", type: "business_rule" },
        { label: "Warnings & Workarounds", type: "warning" },
        { label: "Bug Fix Notes", type: "bugfix" },
        { label: "Architecture Notes", type: "architecture" },
        { label: "Failed Attempts", type: "failed_attempt" },
        { label: "Worklog", type: "worklog" }
      ];
      return categories.map((cat) => {
        const count = pool.filter((item) => matchesCategory(item, cat.type)).length;
        return { cat, count };
      }).filter(({ count }) => count > 0).map(({ cat, count }) => {
        return new MemoryItem(`${cat.label} (${count})`, vscode.TreeItemCollapsibleState.Collapsed, element.rootGroup, cat.type);
      });
    }
    if (element.rootGroup && element.category) {
      const isDocs = element.rootGroup === "docs";
      const pool = this.cachedItems.filter((item) => isDocs ? isDocumentationItem(item) : !isDocumentationItem(item));
      const filtered = pool.filter((item) => matchesCategory(item, element.category));
      return filtered.map((item) => {
        return new MemoryItem(item.title, vscode.TreeItemCollapsibleState.None, element.rootGroup, element.category, item);
      });
    }
    return [];
  }
}

// src/treeViews/healthRows.ts
function graphRows(graph) {
  const percent = (value) => value === null ? "N/A (no relationships)" : `${value.toFixed(1)}%`;
  const rows = [
    { label: "Indexed Files", value: String(graph.indexed_files), details: "Files represented in the static code graph" },
    { label: "Indexed Nodes", value: String(graph.indexed_nodes), details: "All project nodes, including files, symbols, and callsites" },
    { label: "Indexed Functions", value: String(graph.indexed_functions), details: "Function and method nodes in this project" },
    { label: "Resolved Relationships", value: `${graph.resolved_calls}/${graph.call_relationships} (${percent(graph.resolution_percent)})`, details: "Unique caller-target pairs. Callsite mirror edges are excluded; this is not runtime coverage." },
    { label: "Unresolved Relationships", value: String(graph.unresolved_calls), details: "Includes external libraries and unsupported call targets, not necessarily errors" },
    { label: "Cross-file Relationships", value: `${graph.cross_file_calls} (${percent(graph.cross_file_percent)})`, details: "Cross-file pairs divided by resolved pairs; a structural ratio, not a quality score" },
    { label: "Dependency Cycles", value: String(graph.dependency_cycles.length), details: "Strongly connected groups of two or more files, including cycles longer than two files" }
  ];
  for (const file of [...graph.files].filter((f) => f.fan_in + f.fan_out > 0).sort((a, b) => b.fan_in + b.fan_out - (a.fan_in + a.fan_out) || a.file.localeCompare(b.file)).slice(0, 5)) {
    rows.push({ label: file.file, value: `${file.fan_in} in / ${file.fan_out} out`, details: "Distinct other files calling into this file (fan-in) / called by this file (fan-out)" });
  }
  return rows;
}

// src/treeViews/healthTreeProvider.ts
var vscode2 = __toESM(require("vscode"));

class HealthItem extends vscode2.TreeItem {
  label;
  value;
  iconName;
  color;
  details;
  collapsibleState;
  rawData;
  constructor(label, value, iconName, color, details, collapsibleState = vscode2.TreeItemCollapsibleState.None, rawData) {
    super(label, collapsibleState);
    this.label = label;
    this.value = value;
    this.iconName = iconName;
    this.color = color;
    this.details = details;
    this.collapsibleState = collapsibleState;
    this.rawData = rawData;
    if (value) {
      this.description = value;
    }
    if (details) {
      this.tooltip = details;
    }
    if (iconName) {
      this.iconPath = new vscode2.ThemeIcon(iconName, color ? new vscode2.ThemeColor(color) : undefined);
    }
  }
}

class HealthTreeProvider {
  _onDidChangeTreeData = new vscode2.EventEmitter;
  onDidChangeTreeData = this._onDidChangeTreeData.event;
  client;
  constructor(client) {
    this.client = client;
  }
  refresh() {
    this._onDidChangeTreeData.fire();
  }
  getTreeItem(element) {
    return element;
  }
  async getChildren() {
    const items = [];
    try {
      const health = await this.client.getHealth();
      if (health) {
        items.push(new HealthItem("Assessment", health.status, "info", undefined, health.explanation));
        if (health.graph) {
          for (const row of graphRows(health.graph)) {
            const graphItem = new HealthItem(row.label, row.value, "graph", "charts.blue", `${row.details}

\uD83D\uDCA1 Klik baris atau tombol chat untuk kirim analisis metrik ini ke AI Copilot.`, vscode2.TreeItemCollapsibleState.None, { label: row.label, value: row.value, details: row.details });
            graphItem.contextValue = "healthMetric";
            graphItem.command = {
              command: "emocis.sendMetricToChat",
              title: "Analyze Metric with AI Copilot",
              arguments: [graphItem]
            };
            items.push(graphItem);
          }
        }
        if (health.overall_score !== null) {
          const score = health.overall_score;
          const icon = score >= 80 ? "pass-filled" : score >= 60 ? "warning" : "error";
          const color = score >= 80 ? "testing.iconPassed" : score >= 60 ? "charts.yellow" : "testing.iconFailed";
          const overallItem = new HealthItem("Overall Health Score", displayScore(score), icon, color, `Aggregated codebase architectural health

\uD83D\uDCA1 Klik baris atau tombol chat untuk kirim analisis metrik ini ke AI Copilot.`, vscode2.TreeItemCollapsibleState.None, { label: "Overall Health Score", value: displayScore(score), details: "Aggregated codebase architectural health" });
          overallItem.contextValue = "healthMetric";
          overallItem.command = {
            command: "emocis.sendMetricToChat",
            title: "Analyze Metric with AI Copilot",
            arguments: [overallItem]
          };
          items.push(overallItem);
        }
        if (health.modularity !== null) {
          const modItem = new HealthItem("Modularity", displayScore(health.modularity), "package", "charts.blue", `Single responsibility and module segregation

\uD83D\uDCA1 Klik baris atau tombol chat untuk kirim analisis metrik ini ke AI Copilot.`, vscode2.TreeItemCollapsibleState.None, { label: "Modularity", value: displayScore(health.modularity), details: "Single responsibility and module segregation" });
          modItem.contextValue = "healthMetric";
          modItem.command = {
            command: "emocis.sendMetricToChat",
            title: "Analyze Metric with AI Copilot",
            arguments: [modItem]
          };
          items.push(modItem);
        }
        if (health.cohesion !== null) {
          const cohItem = new HealthItem("Cohesion", displayScore(health.cohesion), "git-merge", "charts.green", `Internal relatedness of functions within modules

\uD83D\uDCA1 Klik baris atau tombol chat untuk kirim analisis metrik ini ke AI Copilot.`, vscode2.TreeItemCollapsibleState.None, { label: "Cohesion", value: displayScore(health.cohesion), details: "Internal relatedness of functions within modules" });
          cohItem.contextValue = "healthMetric";
          cohItem.command = {
            command: "emocis.sendMetricToChat",
            title: "Analyze Metric with AI Copilot",
            arguments: [cohItem]
          };
          items.push(cohItem);
        }
        if (health.coupling !== null) {
          const coupItem = new HealthItem("Coupling", displayScore(health.coupling), "references", "charts.purple", `Degree of interdependence between modules

\uD83D\uDCA1 Klik baris atau tombol chat untuk kirim analisis metrik ini ke AI Copilot.`, vscode2.TreeItemCollapsibleState.None, { label: "Coupling", value: displayScore(health.coupling), details: "Degree of interdependence between modules" });
          coupItem.contextValue = "healthMetric";
          coupItem.command = {
            command: "emocis.sendMetricToChat",
            title: "Analyze Metric with AI Copilot",
            arguments: [coupItem]
          };
          items.push(coupItem);
        }
        if (health.boundary_integrity !== null) {
          const boundItem = new HealthItem("Boundary Integrity", displayScore(health.boundary_integrity), "shield", "charts.orange", `Enforcement of architectural layer boundaries

\uD83D\uDCA1 Klik baris atau tombol chat untuk kirim analisis metrik ini ke AI Copilot.`, vscode2.TreeItemCollapsibleState.None, { label: "Boundary Integrity", value: displayScore(health.boundary_integrity), details: "Enforcement of architectural layer boundaries" });
          boundItem.contextValue = "healthMetric";
          boundItem.command = {
            command: "emocis.sendMetricToChat",
            title: "Analyze Metric with AI Copilot",
            arguments: [boundItem]
          };
          items.push(boundItem);
        }
      }
      const audit = await this.client.getAudit();
      if (audit.total_violations > 0) {
        const summaryItem = new HealthItem("Active Drift Issues", `${audit.total_violations} detected`, "alert", "charts.red", `Architectural or documentation drift detected.

\uD83D\uDCA1 Klik baris ini atau tombol chat di kanan untuk kirim semua temuan ke AI Copilot Chat.`, vscode2.TreeItemCollapsibleState.None, { audit });
        summaryItem.contextValue = "healthGroup";
        summaryItem.command = {
          command: "emocis.sendAllDriftToChat",
          title: "Send All Drift Issues to AI Chat"
        };
        items.push(summaryItem);
        for (const issue of audit.violations) {
          const title = issue.title || "Drift Warning";
          const desc = issue.description || issue.location || "";
          let file;
          let line = 1;
          if (issue.location) {
            const lastColon = issue.location.lastIndexOf(":");
            if (lastColon !== -1) {
              file = issue.location.slice(0, lastColon);
              const parsed = parseInt(issue.location.slice(lastColon + 1), 10);
              if (!isNaN(parsed))
                line = parsed;
            } else {
              file = issue.location;
            }
          }
          const issueItem = new HealthItem(`  • ${title}`, issue.location ? `${issue.location} - ${desc}` : desc, "circle-filled", "charts.red", `[${title}]

Lokasi: ${issue.location || "Tidak spesifik"}

Deskripsi: ${desc}

\uD83D\uDCA1 Klik baris untuk buka kode sumber di editor.
\uD83D\uDCAC Klik tombol chat di sebelah kanan (atau klik kanan) untuk langsung kirim ke AI Copilot Chat.`, vscode2.TreeItemCollapsibleState.None, issue);
          issueItem.contextValue = "healthIssue";
          if (file) {
            issueItem.command = {
              command: "emocis.openLocation",
              title: "Open Code Location",
              arguments: [file, line]
            };
          } else {
            issueItem.command = {
              command: "emocis.sendIssueToChat",
              title: "Send Issue to AI Chat",
              arguments: [issueItem]
            };
          }
          items.push(issueItem);
        }
      } else {
        items.push(new HealthItem("Drift Audit", "No heuristic findings", "check", "testing.iconPassed", "No findings in the indexed graph; this does not certify architecture health"));
      }
    } catch (err) {
      items.push(new HealthItem("Engine Status", "Unavailable", "error", "charts.red", String(err)));
    }
    return items;
  }
}

// src/codelens/memoryCodeLensProvider.ts
var vscode3 = __toESM(require("vscode"));

class MemoryCodeLensProvider {
  _onDidChangeCodeLenses = new vscode3.EventEmitter;
  onDidChangeCodeLenses = this._onDidChangeCodeLenses.event;
  provideCodeLenses(document, _token) {
    const codeLenses = [];
    const text = document.getText();
    const fnRegex = /(?:function\s+([a-zA-Z0-9_]+)|(?:fn\s+([a-zA-Z0-9_]+))|(?:def\s+([a-zA-Z0-9_]+))|(?:class\s+([a-zA-Z0-9_]+)))/g;
    let match;
    while ((match = fnRegex.exec(text)) !== null) {
      const symbolName = match[1] || match[2] || match[3] || match[4];
      if (!symbolName)
        continue;
      const position = document.positionAt(match.index);
      const range = new vscode3.Range(position, position);
      codeLenses.push(new vscode3.CodeLens(range, {
        title: `⚡ EMOCIS: Analyze Impact (${symbolName})`,
        tooltip: `Analyze callers and impact radius for ${symbolName}`,
        command: "emocis.analyzeImpact",
        arguments: [symbolName]
      }));
      codeLenses.push(new vscode3.CodeLens(range, {
        title: `\uD83D\uDD0D View in Logic Graph`,
        tooltip: `Open interactive CCLG graph for ${symbolName}`,
        command: "emocis.showGraph",
        arguments: [symbolName]
      }));
      codeLenses.push(new vscode3.CodeLens(range, {
        title: `\uD83E\uDD16 Copilot Context`,
        tooltip: `Copy bounded EMOCIS context for ${symbolName} to AI Copilot / Antigravity`,
        command: "emocis.copilotContextForSymbol",
        arguments: [symbolName]
      }));
    }
    return codeLenses;
  }
}

// src/commands/navigationCommands.ts
var vscode4 = __toESM(require("vscode"));
var path2 = __toESM(require("path"));
var fs2 = __toESM(require("fs"));
function registerNavigationCommands(context, workspaceRoot, outputChannel) {
  context.subscriptions.push(vscode4.commands.registerCommand("emocis.openLocation", async (file, line = 1) => {
    try {
      outputChannel.appendLine(`[EMOCIS] Navigating to: ${file}:${line}`);
      let fullPath = file;
      if (!path2.isAbsolute(file)) {
        const folders = vscode4.workspace.workspaceFolders || [];
        for (const folder of folders) {
          const candidate = path2.join(folder.uri.fsPath, file);
          if (fs2.existsSync(candidate)) {
            fullPath = candidate;
            break;
          }
        }
        if (!path2.isAbsolute(fullPath)) {
          fullPath = path2.resolve(workspaceRoot, file);
        }
      }
      const docUri = vscode4.Uri.file(fullPath);
      const doc = await vscode4.workspace.openTextDocument(docUri);
      const targetLine = Math.max(0, (line || 1) - 1);
      const pos = new vscode4.Position(targetLine, 0);
      const range = new vscode4.Range(pos, pos);
      const editor = await vscode4.window.showTextDocument(doc, {
        preview: false,
        selection: range,
        viewColumn: vscode4.ViewColumn.One
      });
      editor.selection = new vscode4.Selection(pos, pos);
      editor.revealRange(range, vscode4.TextEditorRevealType.InCenter);
      outputChannel.appendLine(`[EMOCIS] Successfully opened and centered at ${fullPath}:${targetLine + 1}`);
    } catch (err) {
      outputChannel.appendLine(`[EMOCIS] Failed to open ${file}:${line} - ${err}`);
    }
  }));
}

// src/commands/memoryCommands.ts
var vscode6 = __toESM(require("vscode"));

// src/webview/graphVisualizer.ts
var vscode5 = __toESM(require("vscode"));

class GraphVisualizerPanel {
  static currentPanel;
  _panel;
  _extensionUri;
  _client;
  _disposables = [];
  static createOrShow(extensionUri, client, initialSymbol) {
    const column = vscode5.window.activeTextEditor ? vscode5.window.activeTextEditor.viewColumn : undefined;
    if (GraphVisualizerPanel.currentPanel) {
      GraphVisualizerPanel.currentPanel._panel.reveal(column);
      if (initialSymbol) {
        GraphVisualizerPanel.currentPanel.loadSymbol(initialSymbol);
      }
      return;
    }
    const panel = vscode5.window.createWebviewPanel("emocisGraph", "EMOCIS Logic Graph (CCLG)", column || vscode5.ViewColumn.One, {
      enableScripts: true,
      retainContextWhenHidden: true
    });
    GraphVisualizerPanel.currentPanel = new GraphVisualizerPanel(panel, extensionUri, client, initialSymbol);
  }
  constructor(panel, extensionUri, client, initialSymbol) {
    this._panel = panel;
    this._extensionUri = extensionUri;
    this._client = client;
    this._update();
    this._panel.onDidDispose(() => this.dispose(), null, this._disposables);
    this._panel.webview.onDidReceiveMessage(async (message) => {
      switch (message.command) {
        case "search":
          this.loadSymbol(message.symbol);
          break;
        case "getImpact":
          const impact = await this._client.getImpact(message.symbol);
          this._panel.webview.postMessage({ command: "renderImpact", data: impact });
          break;
      }
    }, null, this._disposables);
    if (initialSymbol) {
      setTimeout(() => this.loadSymbol(initialSymbol), 500);
    }
  }
  async loadSymbol(symbol) {
    try {
      const calls = await this._client.getCalls(symbol);
      const callers = await this._client.getCallers(symbol);
      const impact = await this._client.getImpact(symbol);
      this._panel.webview.postMessage({
        command: "renderGraph",
        target: symbol,
        calls: Array.isArray(calls) ? calls : [],
        callers: Array.isArray(callers) ? callers : [],
        impact
      });
    } catch (err) {
      console.error("Failed to load symbol in graph:", err);
    }
  }
  dispose() {
    GraphVisualizerPanel.currentPanel = undefined;
    this._panel.dispose();
    while (this._disposables.length) {
      const x = this._disposables.pop();
      if (x) {
        x.dispose();
      }
    }
  }
  _update() {
    this._panel.webview.html = this._getHtmlForWebview();
  }
  _getHtmlForWebview() {
    return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>EMOCIS Logic Graph</title>
  <style>
    :root {
      --bg: #0f141c;
      --card-bg: rgba(26, 34, 48, 0.85);
      --border: #2a364f;
      --accent-blue: #38bdf8;
      --accent-purple: #c084fc;
      --accent-green: #4ade80;
      --text: #f1f5f9;
      --text-muted: #94a3b8;
    }
    body {
      margin: 0;
      padding: 0;
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      background: var(--bg);
      color: var(--text);
      display: flex;
      flex-direction: column;
      height: 100vh;
      overflow: hidden;
    }
    header {
      padding: 12px 20px;
      background: var(--card-bg);
      border-bottom: 1px solid var(--border);
      display: flex;
      align-items: center;
      gap: 12px;
      backdrop-filter: blur(8px);
    }
    h2 {
      margin: 0;
      font-size: 16px;
      font-weight: 600;
      color: var(--accent-blue);
      display: flex;
      align-items: center;
      gap: 8px;
    }
    .search-box {
      display: flex;
      gap: 8px;
      margin-left: auto;
    }
    input {
      background: #090d14;
      border: 1px solid var(--border);
      color: var(--text);
      padding: 6px 12px;
      border-radius: 6px;
      font-size: 13px;
      outline: none;
    }
    input:focus {
      border-color: var(--accent-blue);
    }
    button {
      background: #2563eb;
      color: white;
      border: none;
      padding: 6px 14px;
      border-radius: 6px;
      cursor: pointer;
      font-size: 13px;
      font-weight: 500;
      transition: background 0.2s;
    }
    button:hover {
      background: #1d4ed8;
    }
    #canvas-container {
      flex: 1;
      position: relative;
      background: radial-gradient(circle at 50% 50%, #151d2a 0%, #0b0f17 100%);
      display: flex;
      justify-content: center;
      align-items: center;
      overflow: hidden;
    }
    svg {
      width: 100%;
      height: 100%;
    }
    .node {
      cursor: pointer;
      transition: transform 0.2s;
    }
    .node:hover {
      filter: drop-shadow(0 0 8px rgba(56, 189, 248, 0.6));
    }
    .node text {
      font-size: 12px;
      fill: var(--text);
      text-anchor: middle;
      font-family: ui-monospace, monospace;
      pointer-events: none;
    }
    .edge {
      stroke: #334155;
      stroke-width: 1.5;
      stroke-dasharray: 4, 4;
      animation: dash 30s linear infinite;
    }
    @keyframes dash {
      to { stroke-dashoffset: -1000; }
    }
    #sidebar-panel {
      position: absolute;
      top: 20px;
      right: 20px;
      width: 320px;
      background: var(--card-bg);
      border: 1px solid var(--border);
      border-radius: 8px;
      padding: 16px;
      backdrop-filter: blur(12px);
      box-shadow: 0 10px 25px rgba(0,0,0,0.5);
      font-size: 13px;
      display: none;
    }
    .badge {
      display: inline-block;
      padding: 2px 8px;
      border-radius: 4px;
      font-size: 11px;
      font-weight: 600;
      margin-bottom: 8px;
    }
    .badge-caller { background: #3b82f633; color: var(--accent-blue); border: 1px solid #3b82f655; }
    .badge-callee { background: #10b98133; color: var(--accent-green); border: 1px solid #10b98155; }
    .badge-target { background: #a855f733; color: var(--accent-purple); border: 1px solid #a855f755; }
  </style>
</head>
<body>
  <header>
    <h2>⚡ EMOCIS Canonical Code Logic Graph</h2>
    <div class="search-box">
      <input type="text" id="symbolInput" placeholder="Symbol name (e.g. parse_file)..." />
      <button onclick="searchSymbol()">Inspect Graph</button>
    </div>
  </header>

  <div id="canvas-container">
    <svg id="graphSvg"></svg>
    <div id="sidebar-panel">
      <h3 id="panelTitle" style="margin-top:0; color:var(--accent-blue);">Symbol Detail</h3>
      <div id="panelContent"></div>
    </div>
  </div>

  <script>
    const vscode = acquireVsCodeApi();

    function searchSymbol() {
      const input = document.getElementById('symbolInput');
      const val = input.value.trim();
      if (val) {
        vscode.postMessage({ command: 'search', symbol: val });
      }
    }

    document.getElementById('symbolInput').addEventListener('keydown', (e) => {
      if (e.key === 'Enter') searchSymbol();
    });

    window.addEventListener('message', (event) => {
      const msg = event.data;
      if (msg.command === 'renderGraph') {
        renderGraph(msg.target, msg.calls, msg.callers, msg.impact);
      }
    });

    function renderGraph(target, calls, callers, impact) {
      const svg = document.getElementById('graphSvg');
      svg.innerHTML = '';

      const width = svg.clientWidth || 800;
      const height = svg.clientHeight || 600;
      const centerX = width / 2;
      const centerY = height / 2;

      // Draw Target Center Node
      drawNode(svg, centerX, centerY, target, 'var(--accent-purple)', 'Target Function');

      // Draw Callers (Left side)
      const callerCount = callers.length;
      callers.forEach((c, idx) => {
        const angle = ((idx + 1) / (callerCount + 1) - 0.5) * Math.PI * 0.8;
        const x = centerX - 220;
        const y = centerY + (idx - (callerCount - 1) / 2) * 60;
        drawEdge(svg, x, y, centerX, centerY, 'caller');
        drawNode(svg, x, y, c.name || c, 'var(--accent-blue)', 'Caller');
      });

      // Draw Callees (Right side)
      const callCount = calls.length;
      calls.forEach((c, idx) => {
        const x = centerX + 220;
        const y = centerY + (idx - (callCount - 1) / 2) * 60;
        drawEdge(svg, centerX, centerY, x, y, 'callee');
        drawNode(svg, x, y, c.name || c, 'var(--accent-green)', 'Callee');
      });

      // Update sidebar panel
      const panel = document.getElementById('sidebar-panel');
      const panelTitle = document.getElementById('panelTitle');
      const panelContent = document.getElementById('panelContent');
      panel.style.display = 'block';
      panelTitle.innerText = target;
      panelContent.innerHTML = \`
        <div class="badge badge-target">Partial Static Graph</div>
        <p>Only supported, indexed relationships are shown. Zero callers does not prove a symbol is unused.</p>
        <p><strong>Direct Callers:</strong> \${callers.length}</p>
        <p><strong>Direct Callees:</strong> \${calls.length}</p>
        <p><strong>Impact Radius:</strong> \${impact ? (impact.direct_callers?.length + impact.indirect_callers?.length) : 'Unavailable'}</p>
      \`;
    }

    function drawNode(svg, x, y, text, color, role) {
      const g = document.createElementNS('http://www.w3.org/2000/svg', 'g');
      g.setAttribute('class', 'node');
      g.setAttribute('transform', \`translate(\${x}, \${y})\`);

      const circle = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
      circle.setAttribute('r', '22');
      circle.setAttribute('fill', '#1e293b');
      circle.setAttribute('stroke', color);
      circle.setAttribute('stroke-width', '2.5');

      const label = document.createElementNS('http://www.w3.org/2000/svg', 'text');
      label.setAttribute('y', '36');
      label.textContent = text.length > 20 ? text.substring(0, 18) + '...' : text;

      g.appendChild(circle);
      g.appendChild(label);

      g.onclick = () => {
        document.getElementById('symbolInput').value = text;
        vscode.postMessage({ command: 'search', symbol: text });
      };

      svg.appendChild(g);
    }

    function drawEdge(svg, x1, y1, x2, y2, type) {
      const line = document.createElementNS('http://www.w3.org/2000/svg', 'line');
      line.setAttribute('x1', x1);
      line.setAttribute('y1', y1);
      line.setAttribute('x2', x2);
      line.setAttribute('y2', y2);
      line.setAttribute('class', 'edge');
      line.setAttribute('stroke', type === 'caller' ? '#38bdf8' : '#4ade80');
      svg.appendChild(line);
    }
  </script>
</body>
</html>`;
  }
}

// src/commands/memoryCommands.ts
function registerMemoryCommands(context, workspaceRoot, getClient, memoryProvider, healthProvider) {
  context.subscriptions.push(vscode6.commands.registerCommand("emocis.refresh", () => {
    memoryProvider.refresh();
    healthProvider.refresh();
    vscode6.window.showInformationMessage("EMOCIS: Memory and Health graph refreshed.");
  }));
  context.subscriptions.push(vscode6.commands.registerCommand("emocis.showGraph", (symbolName) => {
    const client = getClient();
    if (!client)
      return;
    if (!symbolName) {
      const editor = vscode6.window.activeTextEditor;
      if (editor) {
        const selection = editor.selection;
        symbolName = editor.document.getText(selection).trim() || editor.document.getText(editor.document.getWordRangeAtPosition(selection.active));
      }
    }
    GraphVisualizerPanel.createOrShow(context.extensionUri, client, symbolName);
  }));
  context.subscriptions.push(vscode6.commands.registerCommand("emocis.analyzeImpact", async (symbolName) => {
    const client = getClient();
    if (!client)
      return;
    if (!symbolName) {
      const editor = vscode6.window.activeTextEditor;
      if (editor) {
        const selection = editor.selection;
        symbolName = editor.document.getText(selection).trim() || editor.document.getText(editor.document.getWordRangeAtPosition(selection.active));
      }
    }
    if (!symbolName) {
      symbolName = await vscode6.window.showInputBox({
        prompt: "Enter symbol name to analyze impact:",
        placeHolder: "e.g. parse_file or processPayment"
      });
    }
    if (!symbolName)
      return;
    vscode6.window.withProgress({
      location: vscode6.ProgressLocation.Notification,
      title: `EMOCIS: Analyzing impact for '${symbolName}'...`
    }, async () => {
      const impact = await client.getImpact(symbolName);
      if (!impact) {
        vscode6.window.showInformationMessage(`No impact detected for '${symbolName}'.`);
        return;
      }
      const direct = impact.direct_callers?.length || 0;
      const indirect = impact.indirect_callers?.length || 0;
      const totalRadius = direct + indirect;
      vscode6.window.showInformationMessage(`Impact Radius for '${symbolName}': ${totalRadius} dependent functions (${direct} direct, ${indirect} transitive). ${impact.unresolved_calls ?? 0} unresolved calls in index. ${impact.analysis_note ?? ""}`, "View in Graph").then((selection) => {
        if (selection === "View in Graph") {
          GraphVisualizerPanel.createOrShow(context.extensionUri, client, symbolName);
        }
      });
    });
  }));
  context.subscriptions.push(vscode6.commands.registerCommand("emocis.search", async () => {
    const client = getClient();
    if (!client)
      return;
    const query = await vscode6.window.showInputBox({
      prompt: "Search architectural decisions, constraints, warnings...",
      placeHolder: "e.g. idempotency, precision, physics"
    });
    if (!query)
      return;
    const results = await client.search(query);
    if (!Array.isArray(results) || results.length === 0) {
      vscode6.window.showInformationMessage(`No memory items found matching '${query}'.`);
      return;
    }
    const quickPickItems = results.map((item) => ({
      label: `[${item.knowledge_type?.toUpperCase()}] ${item.title}`,
      description: item.affects?.join(", "),
      detail: item.content,
      item
    }));
    const selected = await vscode6.window.showQuickPick(quickPickItems, {
      matchOnDescription: true,
      matchOnDetail: true,
      placeHolder: "Select a memory item to navigate to source"
    });
    if (selected && selected.item) {
      const file = selected.item.evidence?.[0]?.file || selected.item.affects?.[0];
      const line = selected.item.evidence?.[0]?.start_line ?? 1;
      if (file) {
        const docUri = vscode6.Uri.file(`${workspaceRoot}/${file}`);
        const doc = await vscode6.workspace.openTextDocument(docUri);
        const editor = await vscode6.window.showTextDocument(doc);
        const range = new vscode6.Range(Math.max(0, line - 1), 0, Math.max(0, line - 1), 0);
        editor.selection = new vscode6.Selection(range.start, range.end);
        editor.revealRange(range, vscode6.TextEditorRevealType.InCenter);
      }
    }
  }));
  context.subscriptions.push(vscode6.commands.registerCommand("emocis.copyContext", async () => {
    const client = getClient();
    if (!client)
      return;
    const intent = await vscode6.window.showInputBox({
      prompt: "Describe your coding task or feature intent:",
      placeHolder: "e.g. Refactor payment retry logic with exponential backoff"
    });
    if (!intent)
      return;
    const ctx = await client.getContext(intent);
    const text = typeof ctx === "string" ? ctx : JSON.stringify(ctx, null, 2);
    await vscode6.env.clipboard.writeText(text);
    vscode6.window.showInformationMessage("⚡ EMOCIS Minimal Context Package copied to clipboard! Paste directly into Cursor / Claude / ChatGPT.");
  }));
  context.subscriptions.push(vscode6.commands.registerCommand("emocis.audit", async () => {
    const client = getClient();
    if (!client)
      return;
    try {
      const audit = await client.getAudit();
      const count = audit.total_violations;
      vscode6.window.showInformationMessage(`EMOCIS Architecture Audit: ${count} active drift/boundary issues detected. View details in sidebar.`);
      healthProvider.refresh();
    } catch (error) {
      vscode6.window.showErrorMessage(`EMOCIS audit unavailable: ${error}`);
    }
  }));
  context.subscriptions.push(vscode6.commands.registerCommand("emocis.copilotContextForSymbol", async (symbolName) => {
    const client = getClient();
    if (!client)
      return;
    if (!symbolName) {
      const editor = vscode6.window.activeTextEditor;
      if (editor) {
        const selection = editor.selection;
        symbolName = editor.document.getText(selection).trim() || editor.document.getText(editor.document.getWordRangeAtPosition(selection.active));
      }
    }
    if (!symbolName)
      return;
    const intent = `Modify or inspect symbol: ${symbolName}`;
    const ctx = await client.getContext(intent);
    const text = typeof ctx === "string" ? ctx : JSON.stringify(ctx, null, 2);
    await vscode6.env.clipboard.writeText(text);
    vscode6.window.showInformationMessage(`\uD83E\uDD16 EMOCIS Context for '${symbolName}' copied to clipboard! Paste into your AI Copilot / Antigravity chat.`);
  }));
}

// src/commands/chatCommands.ts
var vscode8 = __toESM(require("vscode"));

// src/services/aiChatService.ts
var vscode7 = __toESM(require("vscode"));
async function sendToAiChat(prompt) {
  await vscode7.env.clipboard.writeText(prompt);
  let opened = false;
  try {
    await vscode7.commands.executeCommand("workbench.action.chat.open", { query: prompt });
    opened = true;
  } catch {
    try {
      await vscode7.commands.executeCommand("workbench.action.quickchat.open", { query: prompt });
      opened = true;
    } catch {
      try {
        await vscode7.commands.executeCommand("composer.open");
        opened = true;
      } catch {
        try {
          await vscode7.commands.executeCommand("workbench.panel.chat.view.copilot.focus");
          opened = true;
        } catch {}
      }
    }
  }
  if (opened) {
    vscode7.window.showInformationMessage("\uD83D\uDCAC Terkirim langsung ke AI Copilot Chat! (Teks juga tersimpan di clipboard)");
  } else {
    vscode7.window.showInformationMessage("\uD83D\uDCAC Teks telah disalin ke clipboard! Tekan Ctrl+L atau buka AI Chat panel untuk menempelkan.");
  }
}

// src/commands/chatCommands.ts
function registerChatCommands(context, getClient) {
  context.subscriptions.push(vscode8.commands.registerCommand("emocis.sendIssueToChat", async (item) => {
    let prompt = "";
    if (item && item.rawData) {
      const issue = item.rawData;
      prompt = `Tolong analisa dan berikan rekomendasi perbaikan untuk isu arsitektur/drift berikut dari EMOCIS:
- **Isu**: ${issue.title}
- **Lokasi**: ${issue.location || "Tidak spesifik"}
- **Deskripsi**: ${issue.description || ""}
${issue.recommendation ? `- **Rekomendasi**: ${issue.recommendation}
` : ""}
Apakah ini kode mati yang aman dihapus, atau bagaimana cara memperbaikinya dengan best practice?`;
    } else if (item && item.label) {
      prompt = `Tolong analisa isu arsitektur berikut dari EMOCIS:
${item.label}: ${item.details || item.value || ""}`;
    } else {
      const client = getClient();
      if (!client)
        return;
      try {
        const audit = await client.getAudit();
        if (!audit.violations || audit.violations.length === 0) {
          vscode8.window.showInformationMessage("Tidak ada isu drift arsitektur yang terdeteksi.");
          return;
        }
        const picks = audit.violations.map((v) => ({
          label: v.title,
          description: v.location || "",
          detail: v.description,
          issue: v
        }));
        const selected = await vscode8.window.showQuickPick(picks, {
          placeHolder: "Pilih isu drift untuk dikirim ke AI Copilot Chat"
        });
        if (selected) {
          const issue = selected.issue;
          prompt = `Tolong analisa dan berikan rekomendasi perbaikan untuk isu arsitektur/drift berikut dari EMOCIS:
- **Isu**: ${issue.title}
- **Lokasi**: ${issue.location || "Tidak spesifik"}
- **Deskripsi**: ${issue.description || ""}
${issue.recommendation ? `- **Rekomendasi**: ${issue.recommendation}
` : ""}
Apakah ini kode mati yang aman dihapus, atau bagaimana cara memperbaikinya dengan best practice?`;
        } else {
          return;
        }
      } catch (err) {
        vscode8.window.showErrorMessage(`Gagal mengambil data audit: ${err}`);
        return;
      }
    }
    if (prompt) {
      await sendToAiChat(prompt);
    }
  }));
  context.subscriptions.push(vscode8.commands.registerCommand("emocis.sendMetricToChat", async (item) => {
    let metric = item?.rawData?.label || item?.label;
    let val = item?.rawData?.value || item?.value;
    let details = item?.rawData?.details || item?.details;
    if (!metric) {
      const client = getClient();
      if (!client)
        return;
      try {
        const health = await client.getHealth();
        if (!health) {
          vscode8.window.showInformationMessage("Data kesehatan arsitektur belum tersedia.");
          return;
        }
        const picks = [];
        if (health.graph) {
          for (const row of graphRows(health.graph)) {
            picks.push({ label: row.label, value: row.value, details: row.details });
          }
        }
        if (health.overall_score !== null) {
          picks.push({ label: "Overall Health Score", value: displayScore(health.overall_score), details: "Aggregated codebase architectural health" });
        }
        if (health.modularity !== null) {
          picks.push({ label: "Modularity", value: displayScore(health.modularity), details: "Single responsibility and module segregation" });
        }
        if (health.cohesion !== null) {
          picks.push({ label: "Cohesion", value: displayScore(health.cohesion), details: "Internal relatedness of functions within modules" });
        }
        if (health.coupling !== null) {
          picks.push({ label: "Coupling", value: displayScore(health.coupling), details: "Degree of interdependence between modules" });
        }
        if (health.boundary_integrity !== null) {
          picks.push({ label: "Boundary Integrity", value: displayScore(health.boundary_integrity), details: "Enforcement of architectural layer boundaries" });
        }
        const selected = await vscode8.window.showQuickPick(picks, {
          placeHolder: "Pilih metrik arsitektur untuk dianalisis oleh AI Copilot"
        });
        if (selected) {
          metric = selected.label;
          val = selected.value;
          details = selected.details;
        } else {
          return;
        }
      } catch (err) {
        vscode8.window.showErrorMessage(`Gagal mengambil data kesehatan arsitektur: ${err}`);
        return;
      }
    }
    const prompt = `Tolong analisa metrik arsitektur berikut dari EMOCIS:
- **Metrik**: ${metric} (${val || "N/A"})
- **Penjelasan**: ${details || ""}
Bagaimana strategi dan langkah perbaikan teknis terbaik untuk meningkatkan metrik ini pada arsitektur codebase?`;
    await sendToAiChat(prompt);
  }));
  context.subscriptions.push(vscode8.commands.registerCommand("emocis.sendAllDriftToChat", async () => {
    const client = getClient();
    if (!client)
      return;
    try {
      const audit = await client.getAudit();
      let prompt = `Berikut adalah daftar ${audit.total_violations} isu arsitektur dan drift yang terdeteksi oleh EMOCIS pada codebase:

`;
      for (const [idx, v] of audit.violations.slice(0, 15).entries()) {
        prompt += `${idx + 1}. **${v.title}**
   - Lokasi: ${v.location || "-"}
   - Deskripsi: ${v.description}
`;
      }
      if (audit.violations.length > 15) {
        prompt += `
... dan ${audit.violations.length - 15} isu lainnya.
`;
      }
      prompt += `
Tolong tinjau temuan-temuan di atas, kelompokkan berdasarkan tingkat urgensi, dan berikan rencana tindakan perbaikan.`;
      await sendToAiChat(prompt);
    } catch (err) {
      vscode8.window.showErrorMessage(`Gagal mengambil data audit: ${err}`);
    }
  }));
  context.subscriptions.push(vscode8.commands.registerCommand("emocis.sendFullAuditToChat", async () => {
    const client = getClient();
    if (!client)
      return;
    try {
      const audit = await client.getAudit();
      const health = await client.getHealth();
      let prompt = `Berikut adalah ringkasan Architecture Health & Drift Audit dari EMOCIS:

`;
      if (health) {
        if (health.overall_score !== null) {
          prompt += `**Skor Arsitektur:**
`;
          prompt += `- Overall Health Score: ${displayScore(health.overall_score)}
`;
          if (health.modularity !== null)
            prompt += `- Modularity: ${displayScore(health.modularity)}
`;
          if (health.cohesion !== null)
            prompt += `- Cohesion: ${displayScore(health.cohesion)}
`;
          if (health.coupling !== null)
            prompt += `- Coupling: ${displayScore(health.coupling)}
`;
          if (health.boundary_integrity !== null)
            prompt += `- Boundary Integrity: ${displayScore(health.boundary_integrity)}
`;
          prompt += `
`;
        }
        if (health.graph) {
          prompt += `**Pengukuran Graf Statis:**
`;
          prompt += `- Status Analisis: ${health.status} (${health.explanation})
`;
          for (const r of graphRows(health.graph)) {
            prompt += `- ${r.label}: ${r.value}
`;
          }
          prompt += `
`;
        }
      }
      prompt += `**Daftar Temuan Drift (${audit.total_violations} isu):**
`;
      for (const [idx, v] of audit.violations.slice(0, 10).entries()) {
        prompt += `${idx + 1}. **${v.title}** (${v.location || "-"})
   ${v.description}
`;
      }
      if (audit.violations.length > 10) {
        prompt += `... dan ${audit.violations.length - 10} isu lainnya.
`;
      }
      prompt += `
Tolong berikan ulasan komprehensif mengenai kondisi codebase saat ini dan rekomendasi refaktorisasi prioritas utama.`;
      await sendToAiChat(prompt);
    } catch (err) {
      vscode8.window.showErrorMessage(`Gagal memuat audit arsitektur: ${err}`);
    }
  }));
  context.subscriptions.push(vscode8.commands.registerCommand("emocis.sendMemoryToChat", async (item) => {
    if (!item?.data)
      return;
    const d = item.data;
    const prompt = `Berikut adalah catatan Engineering Memory ([${d.knowledge_type?.toUpperCase()}]) dari EMOCIS:
- **Judul**: ${d.title}
- **Konten**: ${d.content}
- **Modul Terkait**: ${d.affects?.join(", ") || "-"}
- **Tags**: ${d.tags?.join(", ") || "-"}

Tolong pastikan kode atau fitur yang sedang kita kerjakan tetap selaras dengan keputusan/batasan arsitektur di atas.`;
    await sendToAiChat(prompt);
  }));
}

// src/commands/configCommands.ts
var vscode9 = __toESM(require("vscode"));
var path3 = __toESM(require("path"));
var fs3 = __toESM(require("fs"));
function registerConfigCommands(context, workspaceRoot) {
  context.subscriptions.push(vscode9.commands.registerCommand("emocis.setupAntigravityMcp", async () => {
    const configObj = {
      mcpServers: {
        emocis: {
          command: "emocis",
          args: ["mcp"]
        }
      }
    };
    const formatted = JSON.stringify(configObj, null, 2);
    const configPath = path3.join(workspaceRoot, "emocis-mcp-config.json");
    const cursorDir = path3.join(workspaceRoot, ".cursor");
    const cursorPath = path3.join(cursorDir, "mcp.json");
    try {
      fs3.writeFileSync(configPath, formatted, "utf8");
      if (fs3.existsSync(cursorDir)) {
        fs3.writeFileSync(cursorPath, formatted, "utf8");
      }
      await vscode9.env.clipboard.writeText(formatted);
      vscode9.window.showInformationMessage("✔ EMOCIS MCP configuration created & copied to clipboard! Ready for Antigravity IDE and Cursor AI Agent.", "Open Config").then((sel) => {
        if (sel === "Open Config") {
          vscode9.workspace.openTextDocument(configPath).then((doc) => vscode9.window.showTextDocument(doc));
        }
      });
    } catch {
      await vscode9.env.clipboard.writeText(formatted);
      vscode9.window.showInformationMessage("✔ EMOCIS MCP configuration snippet copied to clipboard! Paste it into Antigravity or Cursor mcp_config.json.");
    }
  }));
}

// src/services/fileWatcherService.ts
var vscode10 = __toESM(require("vscode"));
var path4 = __toESM(require("path"));
var WATCHED_EXTENSIONS = new Set([
  ".ts",
  ".tsx",
  ".js",
  ".jsx",
  ".rs",
  ".py",
  ".md",
  ".vue",
  ".svelte"
]);
function setupAutoRefreshWatcher(context, outputChannel, onRefresh) {
  let autoRefreshTimeout;
  context.subscriptions.push(vscode10.workspace.onDidSaveTextDocument((doc) => {
    const ext = path4.extname(doc.fileName).toLowerCase();
    if (WATCHED_EXTENSIONS.has(ext)) {
      if (autoRefreshTimeout)
        clearTimeout(autoRefreshTimeout);
      autoRefreshTimeout = setTimeout(() => {
        outputChannel.appendLine(`[EMOCIS] Document saved: ${path4.basename(doc.fileName)}. Refreshing memory...`);
        onRefresh();
      }, 1500);
    }
  }));
}

// src/services/skillService.ts
var path5 = __toESM(require("path"));
var fs4 = __toESM(require("fs"));
var EMOCIS_SKILL_CONTENT = `---
name: emocis
description: "EMOCIS Engineering Memory & Codebase Intelligence CLI. Use to inspect code logic graphs (CCLG), audit architecture drift, check blast radius/impact of functions, check modularity/health scores, and record failed attempts."
---

# EMOCIS — Engineering Memory & Codebase Intelligence

EMOCIS is a deterministic Codebase Logic Graph (CCLG) and Engineering Memory system.
Use EMOCIS CLI commands in the terminal to avoid blind edits, prevent architectural drift, check ripple effects, and maintain codebase health.

## \uD83D\uDE80 Key Workflows for AI Coding Agents

### 1. Before Modifying Existing Functions (Impact Analysis)
Before refactoring or altering a function's signature, check who calls it across the entire repository:
\`\`\`bash
project context <symbol_or_function_name>
\`\`\`
* Identifies direct and transitive callers across frontend & backend (e.g. Vue components, Pinia stores, Tauri IPC handlers).
* Prevents breaking callers in distant files.

### 2. After Generating or Editing Code (Architecture Verification)
Always re-index and run an audit to ensure no circular dependencies or dead logic were introduced:
\`\`\`bash
project map && project audit
\`\`\`
* \`project map\`: Incrementally parses ASTs and updates the local logic graph in \`.emocis/emocis.db\`. Use \`--force\` to re-index all files.
* \`project audit\`: Detects circular dependencies (Kosaraju SCC), unreferenced private code, and doc drift.

### 3. Checking Architecture Health & Quality Scores
Check overall codebase architecture scores:
\`\`\`bash
project health
\`\`\`
Outputs calibrated metrics (0-100):
* **Overall Health Score**: Aggregated codebase architectural health.
* **Modularity Score**: Evaluates module segregation and penalizes cyclic dependencies.
* **Cohesion Score**: Percentage of calls collaborating within the same subsystem.
* **Coupling Score**: Measures instability and cross-file entanglement.
* **Boundary Integrity**: Clean separation across architectural layers.
* **Technical Debt Index**: Low / Moderate / High maintainability rating.

### 4. Preventing Repeated Mistakes (Failed Attempts Memory)
If an architectural approach or fix fails (e.g., causes SQLite lock, Vue reactivity loop, or bundle crash), record it so neither you nor future AI sessions repeat the mistake:
\`\`\`bash
project failed --title "Deadlock SQLite saat background sync" --description "Jangan memanggil syncService.syncAll di dalam transaksi local_db karena menyebabkan database terkunci."
\`\`\`

### 5. Searching Architectural Knowledge & Rules
Search decisions (\`@decision\`), patterns (\`@pattern\`), and warnings (\`@warning\`) documented across code:
\`\`\`bash
project search <keyword>
\`\`\`

---

## \uD83D\uDCCB Quick Command Reference

| Command | Purpose |
|---|---|
| \`project map [--force]\` | Scan & index codebase into CCLG logic graph |
| \`project audit\` | Detect circular dependencies and drift |
| \`project health\` | Display architecture scores & relationship ratios |
| \`project context <symbol>\` | Inspect blast radius / callers of a function |
| \`project search <term>\` | Search engineering decisions & memory |
| \`project failed\` | Record or list failed technical attempts |
| \`project snapshot\` | Create baseline snapshot for drift comparison |
| \`clg\` | Interactive CLI code logic graph explorer |
| \`emocis mcp\` | Run stdio Model Context Protocol (MCP) server |

## \uD83D\uDEE1️ Vibecoding Guardrails
1. **Never edit blindly**: Always inspect blast radius with \`project context <symbol>\`.
2. **Never leave broken cycles**: Run \`project audit\` before concluding your turn.
3. **Record failures**: Use \`project failed\` when an idea proves invalid to preserve memory.
`;
function ensureEmocisSkill(workspaceRoot, outputChannel) {
  try {
    const emocisDir = path5.join(workspaceRoot, ".emocis");
    if (!fs4.existsSync(emocisDir)) {
      fs4.mkdirSync(emocisDir, { recursive: true });
    }
    const emocisSkillPath = path5.join(emocisDir, "SKILL.md");
    fs4.writeFileSync(emocisSkillPath, EMOCIS_SKILL_CONTENT, "utf8");
    const agentSkillDir = path5.join(workspaceRoot, ".agents", "skills", "emocis");
    if (!fs4.existsSync(agentSkillDir)) {
      fs4.mkdirSync(agentSkillDir, { recursive: true });
    }
    const agentSkillPath = path5.join(agentSkillDir, "SKILL.md");
    fs4.writeFileSync(agentSkillPath, EMOCIS_SKILL_CONTENT, "utf8");
    outputChannel?.appendLine(`[EMOCIS] Generated SKILL.md in .emocis/ and .agents/skills/emocis/`);
  } catch (err) {
    outputChannel?.appendLine(`[EMOCIS] Warning: Could not generate SKILL.md: ${err}`);
  }
}

// src/extension.ts
var emocisClient;
function activate(context) {
  const workspaceRoot = vscode11.workspace.workspaceFolders?.[0]?.uri.fsPath || process.cwd();
  const outputChannel = vscode11.window.createOutputChannel("EMOCIS");
  outputChannel.appendLine(`[EMOCIS] Extension activated for workspace: ${workspaceRoot}`);
  ensureEmocisSkill(workspaceRoot, outputChannel);
  emocisClient = new EmocisClient(workspaceRoot);
  const getClient = () => emocisClient;
  const memoryProvider = new MemoryTreeProvider(emocisClient);
  const healthProvider = new HealthTreeProvider(emocisClient);
  context.subscriptions.push(outputChannel, vscode11.window.registerTreeDataProvider("emocis.memoryView", memoryProvider), vscode11.window.registerTreeDataProvider("emocis.healthView", healthProvider));
  const supportedLanguages = [
    { language: "typescript" },
    { language: "typescriptreact" },
    { language: "javascript" },
    { language: "javascriptreact" },
    { language: "rust" },
    { language: "python" }
  ];
  context.subscriptions.push(vscode11.languages.registerCodeLensProvider(supportedLanguages, new MemoryCodeLensProvider));
  registerNavigationCommands(context, workspaceRoot, outputChannel);
  registerMemoryCommands(context, workspaceRoot, getClient, memoryProvider, healthProvider);
  registerChatCommands(context, getClient);
  registerConfigCommands(context, workspaceRoot);
  setupAutoRefreshWatcher(context, outputChannel, () => {
    memoryProvider.refresh();
    healthProvider.refresh();
  });
}
function deactivate() {
  if (emocisClient) {
    emocisClient.dispose();
    emocisClient = undefined;
  }
}
