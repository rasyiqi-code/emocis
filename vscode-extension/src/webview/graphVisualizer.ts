import * as vscode from "vscode";
import { EmocisClient } from "../mcpClient";

export class GraphVisualizerPanel {
  public static currentPanel: GraphVisualizerPanel | undefined;
  private readonly _panel: vscode.WebviewPanel;
  private readonly _extensionUri: vscode.Uri;
  private readonly _client: EmocisClient;
  private _disposables: vscode.Disposable[] = [];

  public static createOrShow(extensionUri: vscode.Uri, client: EmocisClient, initialSymbol?: string) {
    const column = vscode.window.activeTextEditor
      ? vscode.window.activeTextEditor.viewColumn
      : undefined;

    if (GraphVisualizerPanel.currentPanel) {
      GraphVisualizerPanel.currentPanel._panel.reveal(column);
      if (initialSymbol) {
        GraphVisualizerPanel.currentPanel.loadSymbol(initialSymbol);
      }
      return;
    }

    const panel = vscode.window.createWebviewPanel(
      "emocisGraph",
      "EMOCIS Logic Graph (CCLG)",
      column || vscode.ViewColumn.One,
      {
        enableScripts: true,
        retainContextWhenHidden: true,
      }
    );

    GraphVisualizerPanel.currentPanel = new GraphVisualizerPanel(panel, extensionUri, client, initialSymbol);
  }

  private constructor(
    panel: vscode.WebviewPanel,
    extensionUri: vscode.Uri,
    client: EmocisClient,
    initialSymbol?: string
  ) {
    this._panel = panel;
    this._extensionUri = extensionUri;
    this._client = client;

    this._update();

    this._panel.onDidDispose(() => this.dispose(), null, this._disposables);

    this._panel.webview.onDidReceiveMessage(
      async (message) => {
        switch (message.command) {
          case "search":
            this.loadSymbol(message.symbol);
            break;
          case "getImpact":
            const impact = await this._client.getImpact(message.symbol);
            this._panel.webview.postMessage({ command: "renderImpact", data: impact });
            break;
        }
      },
      null,
      this._disposables
    );

    if (initialSymbol) {
      setTimeout(() => this.loadSymbol(initialSymbol), 500);
    }
  }

  public async loadSymbol(symbol: string) {
    try {
      const calls = await this._client.getCalls(symbol);
      const callers = await this._client.getCallers(symbol);
      const impact = await this._client.getImpact(symbol);

      this._panel.webview.postMessage({
        command: "renderGraph",
        target: symbol,
        calls: Array.isArray(calls) ? calls : [],
        callers: Array.isArray(callers) ? callers : [],
        impact,
      });
    } catch (err) {
      console.error("Failed to load symbol in graph:", err);
    }
  }

  public dispose() {
    GraphVisualizerPanel.currentPanel = undefined;
    this._panel.dispose();
    while (this._disposables.length) {
      const x = this._disposables.pop();
      if (x) {
        x.dispose();
      }
    }
  }

  private _update() {
    this._panel.webview.html = this._getHtmlForWebview();
  }

  private _getHtmlForWebview(): string {
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
