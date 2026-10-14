import * as vscode from "vscode";

export class MemoryCodeLensProvider implements vscode.CodeLensProvider {
  private _onDidChangeCodeLenses: vscode.EventEmitter<void> = new vscode.EventEmitter<void>();
  public readonly onDidChangeCodeLenses: vscode.Event<void> = this._onDidChangeCodeLenses.event;

  public provideCodeLenses(
    document: vscode.TextDocument,
    _token: vscode.CancellationToken
  ): vscode.CodeLens[] | Thenable<vscode.CodeLens[]> {
    const codeLenses: vscode.CodeLens[] = [];
    const text = document.getText();

    // Match function, class, and method definitions in TS/JS, Rust, Python
    const fnRegex = /(?:function\s+([a-zA-Z0-9_]+)|(?:fn\s+([a-zA-Z0-9_]+))|(?:def\s+([a-zA-Z0-9_]+))|(?:class\s+([a-zA-Z0-9_]+)))/g;
    let match;

    while ((match = fnRegex.exec(text)) !== null) {
      const symbolName = match[1] || match[2] || match[3] || match[4];
      if (!symbolName) continue;

      const position = document.positionAt(match.index);
      const range = new vscode.Range(position, position);

      // 1. Impact Analysis CodeLens
      codeLenses.push(
        new vscode.CodeLens(range, {
          title: `⚡ EMOCIS: Analyze Impact (${symbolName})`,
          tooltip: `Analyze callers and impact radius for ${symbolName}`,
          command: "emocis.analyzeImpact",
          arguments: [symbolName],
        })
      );

      // 2. Logic Graph Visualizer CodeLens
      codeLenses.push(
        new vscode.CodeLens(range, {
          title: `🔍 View in Logic Graph`,
          tooltip: `Open interactive CCLG graph for ${symbolName}`,
          command: "emocis.showGraph",
          arguments: [symbolName],
        })
      );
    }

    return codeLenses;
  }
}
