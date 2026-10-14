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
var vscode5 = __toESM(require("vscode"));
var path2 = __toESM(require("path"));
var fs2 = __toESM(require("fs"));

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
    const emocisWorkspaceBinary = "/media/rasyiqi/7653717A1C07B131/EMOCIS/target/release/emocis";
    const emocisWorkspaceDebugBinary = "/media/rasyiqi/7653717A1C07B131/EMOCIS/target/debug/emocis";
    let executable = "emocis";
    let args = ["mcp"];
    if (fs.existsSync(cargoBinary)) {
      executable = cargoBinary;
    } else if (fs.existsSync(releaseTargetBinary)) {
      executable = releaseTargetBinary;
    } else if (fs.existsSync(localTargetBinary)) {
      executable = localTargetBinary;
    } else if (fs.existsSync(emocisWorkspaceBinary)) {
      executable = emocisWorkspaceBinary;
    } else if (fs.existsSync(emocisWorkspaceDebugBinary)) {
      executable = emocisWorkspaceDebugBinary;
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
          req.resolve({ content: [] });
        }
        this.pendingRequests.clear();
      });
      this.process.on("exit", (code) => {
        console.log(`[EMOCIS Client] Subprocess exited with code ${code}`);
        this.process = null;
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
          resolve("");
        }
      }, 3000);
      this.pendingRequests.set(id, {
        resolve: (result) => {
          clearTimeout(timeout);
          const text = result?.content?.[0]?.text ?? "";
          resolve(text);
        },
        reject: () => {
          clearTimeout(timeout);
          resolve("");
        }
      });
      this.process?.stdin.write(payload);
    });
  }
  async search(query) {
    const raw = await this.callTool("emocis_search", { query });
    try {
      return JSON.parse(raw);
    } catch {
      return [];
    }
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
    try {
      return JSON.parse(raw);
    } catch {
      return null;
    }
  }
  async getCalls(fn) {
    const raw = await this.callTool("emocis_calls", { function: fn });
    try {
      return JSON.parse(raw);
    } catch {
      return [];
    }
  }
  async getCallers(fn) {
    const raw = await this.callTool("emocis_callers", { function: fn });
    try {
      return JSON.parse(raw);
    } catch {
      return [];
    }
  }
  async getHealth() {
    const raw = await this.callTool("emocis_health");
    try {
      return JSON.parse(raw);
    } catch {
      return null;
    }
  }
  async getAudit() {
    const raw = await this.callTool("emocis_audit");
    try {
      return JSON.parse(raw);
    } catch {
      return [];
    }
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

class MemoryItem extends vscode.TreeItem {
  label;
  collapsibleState;
  category;
  data;
  filePath;
  line;
  constructor(label, collapsibleState, category, data, filePath, line) {
    super(label, collapsibleState);
    this.label = label;
    this.collapsibleState = collapsibleState;
    this.category = category;
    this.data = data;
    this.filePath = filePath;
    this.line = line;
    if (data) {
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
        default:
          this.iconPath = new vscode.ThemeIcon("bookmark", new vscode.ThemeColor("charts.green"));
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
      const categories = [
        { label: "Decisions", type: "decision" },
        { label: "Constraints", type: "constraint" },
        { label: "Business Rules", type: "business_rule" },
        { label: "Warnings & Caveats", type: "warning" },
        { label: "Failed Attempts", type: "failed_attempt" },
        { label: "Architecture", type: "architecture" }
      ];
      return categories.map((cat) => {
        const count = this.cachedItems.filter((item) => item.knowledge_type?.toLowerCase() === cat.type.toLowerCase() || item.tags?.includes(cat.type)).length;
        return new MemoryItem(`${cat.label} (${count})`, count > 0 ? vscode.TreeItemCollapsibleState.Collapsed : vscode.TreeItemCollapsibleState.None, cat.type);
      });
    }
    if (element.category) {
      const filtered = this.cachedItems.filter((item) => item.knowledge_type?.toLowerCase() === element.category?.toLowerCase() || item.tags?.includes(element.category || ""));
      return filtered.map((item) => {
        return new MemoryItem(item.title, vscode.TreeItemCollapsibleState.None, undefined, item);
      });
    }
    return [];
  }
}

// src/treeViews/healthTreeProvider.ts
var vscode2 = __toESM(require("vscode"));

class HealthItem extends vscode2.TreeItem {
  label;
  value;
  iconName;
  color;
  details;
  constructor(label, value, iconName, color, details) {
    super(label, vscode2.TreeItemCollapsibleState.None);
    this.label = label;
    this.value = value;
    this.iconName = iconName;
    this.color = color;
    this.details = details;
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
        const score = health.overall_score ?? 0;
        const icon = score >= 80 ? "pass-filled" : score >= 60 ? "warning" : "error";
        const color = score >= 80 ? "testing.iconPassed" : score >= 60 ? "charts.yellow" : "testing.iconFailed";
        items.push(new HealthItem("Overall Health Score", `${score}/100`, icon, color, "Aggregated codebase architectural health"));
        items.push(new HealthItem("Modularity", `${health.modularity_score ?? 0}/100`, "package", "charts.blue", "Single responsibility and module segregation"));
        items.push(new HealthItem("Cohesion", `${health.cohesion_score ?? 0}/100`, "git-merge", "charts.green", "Internal relatedness of functions within modules"));
        items.push(new HealthItem("Coupling", `${health.coupling_score ?? 0}/100`, "references", "charts.purple", "Degree of interdependence between modules"));
        items.push(new HealthItem("Boundary Integrity", `${health.boundary_integrity_score ?? 0}/100`, "shield", "charts.orange", "Enforcement of architectural layer boundaries"));
      }
      const audit = await this.client.getAudit();
      if (Array.isArray(audit) && audit.length > 0) {
        items.push(new HealthItem("Active Drift Issues", `${audit.length} detected`, "alert", "charts.red", "Architectural or documentation drift detected"));
        for (const issue of audit.slice(0, 5)) {
          const title = issue.title || issue.issue_type || "Drift Warning";
          const desc = issue.description || issue.file_path || "";
          items.push(new HealthItem(`  • ${title}`, desc, "circle-filled", "charts.red", desc));
        }
      } else {
        items.push(new HealthItem("Drift Audit", "Clean (0 issues)", "check", "testing.iconPassed", "No architectural drift or dead logic detected"));
      }
    } catch (err) {
      items.push(new HealthItem("Engine Status", "Connecting...", "sync~spin", "charts.blue"));
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
    }
    return codeLenses;
  }
}

// src/webview/graphVisualizer.ts
var vscode4 = __toESM(require("vscode"));

class GraphVisualizerPanel {
  static currentPanel;
  _panel;
  _extensionUri;
  _client;
  _disposables = [];
  static createOrShow(extensionUri, client, initialSymbol) {
    const column = vscode4.window.activeTextEditor ? vscode4.window.activeTextEditor.viewColumn : undefined;
    if (GraphVisualizerPanel.currentPanel) {
      GraphVisualizerPanel.currentPanel._panel.reveal(column);
      if (initialSymbol) {
        GraphVisualizerPanel.currentPanel.loadSymbol(initialSymbol);
      }
      return;
    }
    const panel = vscode4.window.createWebviewPanel("emocisGraph", "EMOCIS Logic Graph (CCLG)", column || vscode4.ViewColumn.One, {
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
        <div class="badge badge-target">Active Focus</div>
        <p><strong>Direct Callers:</strong> \${callers.length}</p>
        <p><strong>Direct Callees:</strong> \${calls.length}</p>
        <p><strong>Impact Radius:</strong> \${impact ? (impact.direct_callers?.length + impact.indirect_callers?.length) : 'Clean'}</p>
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

// src/extension.ts
var emocisClient;
function activate(context) {
  const workspaceRoot = vscode5.workspace.workspaceFolders?.[0]?.uri.fsPath || process.cwd();
  const outputChannel = vscode5.window.createOutputChannel("EMOCIS");
  outputChannel.appendLine(`[EMOCIS] Extension activated for workspace: ${workspaceRoot}`);
  emocisClient = new EmocisClient(workspaceRoot);
  const memoryProvider = new MemoryTreeProvider(emocisClient);
  const healthProvider = new HealthTreeProvider(emocisClient);
  context.subscriptions.push(outputChannel, vscode5.window.registerTreeDataProvider("emocis.memoryView", memoryProvider), vscode5.window.registerTreeDataProvider("emocis.healthView", healthProvider));
  const supportedLanguages = [
    { language: "typescript" },
    { language: "typescriptreact" },
    { language: "javascript" },
    { language: "javascriptreact" },
    { language: "rust" },
    { language: "python" }
  ];
  context.subscriptions.push(vscode5.languages.registerCodeLensProvider(supportedLanguages, new MemoryCodeLensProvider));
  context.subscriptions.push(vscode5.commands.registerCommand("emocis.refresh", () => {
    memoryProvider.refresh();
    healthProvider.refresh();
    vscode5.window.showInformationMessage("EMOCIS: Memory and Health graph refreshed.");
  }));
  context.subscriptions.push(vscode5.commands.registerCommand("emocis.openLocation", async (file, line = 1) => {
    try {
      outputChannel.appendLine(`[EMOCIS] Navigating to: ${file}:${line}`);
      let fullPath = file;
      if (!path2.isAbsolute(file)) {
        const folders = vscode5.workspace.workspaceFolders || [];
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
      const docUri = vscode5.Uri.file(fullPath);
      const doc = await vscode5.workspace.openTextDocument(docUri);
      const targetLine = Math.max(0, (line || 1) - 1);
      const pos = new vscode5.Position(targetLine, 0);
      const range = new vscode5.Range(pos, pos);
      const editor = await vscode5.window.showTextDocument(doc, {
        preview: false,
        selection: range,
        viewColumn: vscode5.ViewColumn.One
      });
      editor.selection = new vscode5.Selection(pos, pos);
      editor.revealRange(range, vscode5.TextEditorRevealType.InCenter);
      outputChannel.appendLine(`[EMOCIS] Successfully opened and centered at ${fullPath}:${targetLine + 1}`);
    } catch (err) {
      outputChannel.appendLine(`[EMOCIS] Failed to open ${file}:${line} - ${err}`);
    }
  }));
  context.subscriptions.push(vscode5.commands.registerCommand("emocis.showGraph", (symbolName) => {
    if (!symbolName) {
      const editor = vscode5.window.activeTextEditor;
      if (editor) {
        const selection = editor.selection;
        symbolName = editor.document.getText(selection).trim() || editor.document.getText(editor.document.getWordRangeAtPosition(selection.active));
      }
    }
    GraphVisualizerPanel.createOrShow(context.extensionUri, emocisClient, symbolName);
  }));
  context.subscriptions.push(vscode5.commands.registerCommand("emocis.analyzeImpact", async (symbolName) => {
    if (!symbolName) {
      const editor = vscode5.window.activeTextEditor;
      if (editor) {
        const selection = editor.selection;
        symbolName = editor.document.getText(selection).trim() || editor.document.getText(editor.document.getWordRangeAtPosition(selection.active));
      }
    }
    if (!symbolName) {
      symbolName = await vscode5.window.showInputBox({
        prompt: "Enter symbol name to analyze impact:",
        placeHolder: "e.g. parse_file or processPayment"
      });
    }
    if (!symbolName)
      return;
    vscode5.window.withProgress({
      location: vscode5.ProgressLocation.Notification,
      title: `EMOCIS: Analyzing impact for '${symbolName}'...`
    }, async () => {
      const impact = await emocisClient.getImpact(symbolName);
      if (!impact) {
        vscode5.window.showInformationMessage(`No impact detected for '${symbolName}'.`);
        return;
      }
      const direct = impact.direct_callers?.length || 0;
      const indirect = impact.indirect_callers?.length || 0;
      const totalRadius = direct + indirect;
      vscode5.window.showInformationMessage(`Impact Radius for '${symbolName}': ${totalRadius} dependent functions (${direct} direct, ${indirect} transitive).`, "View in Graph").then((selection) => {
        if (selection === "View in Graph") {
          GraphVisualizerPanel.createOrShow(context.extensionUri, emocisClient, symbolName);
        }
      });
    });
  }));
  context.subscriptions.push(vscode5.commands.registerCommand("emocis.search", async () => {
    const query = await vscode5.window.showInputBox({
      prompt: "Search architectural decisions, constraints, warnings...",
      placeHolder: "e.g. idempotency, precision, physics"
    });
    if (!query)
      return;
    const results = await emocisClient.search(query);
    if (!Array.isArray(results) || results.length === 0) {
      vscode5.window.showInformationMessage(`No memory items found matching '${query}'.`);
      return;
    }
    const quickPickItems = results.map((item) => ({
      label: `[${item.knowledge_type?.toUpperCase()}] ${item.title}`,
      description: item.affects?.join(", "),
      detail: item.content,
      item
    }));
    const selected = await vscode5.window.showQuickPick(quickPickItems, {
      matchOnDescription: true,
      matchOnDetail: true,
      placeHolder: "Select a memory item to navigate to source"
    });
    if (selected && selected.item) {
      const file = selected.item.evidence?.[0]?.file || selected.item.affects?.[0];
      const line = selected.item.evidence?.[0]?.start_line ?? 1;
      if (file) {
        const docUri = vscode5.Uri.file(`${workspaceRoot}/${file}`);
        const doc = await vscode5.workspace.openTextDocument(docUri);
        const editor = await vscode5.window.showTextDocument(doc);
        const range = new vscode5.Range(Math.max(0, line - 1), 0, Math.max(0, line - 1), 0);
        editor.selection = new vscode5.Selection(range.start, range.end);
        editor.revealRange(range, vscode5.TextEditorRevealType.InCenter);
      }
    }
  }));
  context.subscriptions.push(vscode5.commands.registerCommand("emocis.copyContext", async () => {
    const intent = await vscode5.window.showInputBox({
      prompt: "Describe your coding task or feature intent:",
      placeHolder: "e.g. Refactor payment retry logic with exponential backoff"
    });
    if (!intent)
      return;
    const ctx = await emocisClient.getContext(intent);
    const text = typeof ctx === "string" ? ctx : JSON.stringify(ctx, null, 2);
    await vscode5.env.clipboard.writeText(text);
    vscode5.window.showInformationMessage("⚡ EMOCIS Minimal Context Package copied to clipboard! Paste directly into Cursor / Claude / ChatGPT.");
  }));
  context.subscriptions.push(vscode5.commands.registerCommand("emocis.audit", async () => {
    const audit = await emocisClient.getAudit();
    const count = Array.isArray(audit) ? audit.length : 0;
    vscode5.window.showInformationMessage(`EMOCIS Architecture Audit: ${count} active drift/boundary issues detected. View details in sidebar.`);
    healthProvider.refresh();
  }));
}
function deactivate() {
  if (emocisClient) {
    emocisClient.dispose();
  }
}
