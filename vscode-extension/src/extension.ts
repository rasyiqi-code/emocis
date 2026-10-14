import * as vscode from "vscode";
import * as path from "path";
import * as fs from "fs";
import { EmocisClient } from "./mcpClient";
import { MemoryTreeProvider } from "./treeViews/memoryTreeProvider";
import { HealthTreeProvider } from "./treeViews/healthTreeProvider";
import { MemoryCodeLensProvider } from "./codelens/memoryCodeLensProvider";
import { GraphVisualizerPanel } from "./webview/graphVisualizer";

let emocisClient: EmocisClient | undefined;

export function activate(context: vscode.ExtensionContext) {
  const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || process.cwd();
  const outputChannel = vscode.window.createOutputChannel("EMOCIS");
  outputChannel.appendLine(`[EMOCIS] Extension activated for workspace: ${workspaceRoot}`);

  emocisClient = new EmocisClient(workspaceRoot);

  // 1. Sidebar Tree Views
  const memoryProvider = new MemoryTreeProvider(emocisClient);
  const healthProvider = new HealthTreeProvider(emocisClient);

  context.subscriptions.push(
    outputChannel,
    vscode.window.registerTreeDataProvider("emocis.memoryView", memoryProvider),
    vscode.window.registerTreeDataProvider("emocis.healthView", healthProvider)
  );

  // 2. CodeLens Provider for multi-language ASTs
  const supportedLanguages = [
    { language: "typescript" },
    { language: "typescriptreact" },
    { language: "javascript" },
    { language: "javascriptreact" },
    { language: "rust" },
    { language: "python" },
  ];
  context.subscriptions.push(
    vscode.languages.registerCodeLensProvider(
      supportedLanguages,
      new MemoryCodeLensProvider()
    )
  );

  // 3. Command: Refresh
  context.subscriptions.push(
    vscode.commands.registerCommand("emocis.refresh", () => {
      memoryProvider.refresh();
      healthProvider.refresh();
      vscode.window.showInformationMessage("EMOCIS: Memory and Health graph refreshed.");
    })
  );

  // 3b. Command: Open Location with accurate line & center reveal
  context.subscriptions.push(
    vscode.commands.registerCommand("emocis.openLocation", async (file: string, line: number = 1) => {
      try {
        outputChannel.appendLine(`[EMOCIS] Navigating to: ${file}:${line}`);
        let fullPath = file;
        if (!path.isAbsolute(file)) {
          const folders = vscode.workspace.workspaceFolders || [];
          for (const folder of folders) {
            const candidate = path.join(folder.uri.fsPath, file);
            if (fs.existsSync(candidate)) {
              fullPath = candidate;
              break;
            }
          }
          if (!path.isAbsolute(fullPath)) {
            fullPath = path.resolve(workspaceRoot, file);
          }
        }

        const docUri = vscode.Uri.file(fullPath);
        const doc = await vscode.workspace.openTextDocument(docUri);
        const targetLine = Math.max(0, (line || 1) - 1);
        const pos = new vscode.Position(targetLine, 0);
        const range = new vscode.Range(pos, pos);

        const editor = await vscode.window.showTextDocument(doc, {
          preview: false,
          selection: range,
          viewColumn: vscode.ViewColumn.One,
        });

        editor.selection = new vscode.Selection(pos, pos);
        editor.revealRange(range, vscode.TextEditorRevealType.InCenter);
        outputChannel.appendLine(`[EMOCIS] Successfully opened and centered at ${fullPath}:${targetLine + 1}`);
      } catch (err) {
        outputChannel.appendLine(`[EMOCIS] Failed to open ${file}:${line} - ${err}`);
      }
    })
  );

  // 4. Command: Show Graph Visualizer
  context.subscriptions.push(
    vscode.commands.registerCommand("emocis.showGraph", (symbolName?: string) => {
      if (!symbolName) {
        const editor = vscode.window.activeTextEditor;
        if (editor) {
          const selection = editor.selection;
          symbolName = editor.document.getText(selection).trim() ||
                       editor.document.getText(editor.document.getWordRangeAtPosition(selection.active));
        }
      }
      GraphVisualizerPanel.createOrShow(context.extensionUri, emocisClient!, symbolName);
    })
  );

  // 5. Command: Analyze Impact
  context.subscriptions.push(
    vscode.commands.registerCommand("emocis.analyzeImpact", async (symbolName?: string) => {
      if (!symbolName) {
        const editor = vscode.window.activeTextEditor;
        if (editor) {
          const selection = editor.selection;
          symbolName = editor.document.getText(selection).trim() ||
                       editor.document.getText(editor.document.getWordRangeAtPosition(selection.active));
        }
      }

      if (!symbolName) {
        symbolName = await vscode.window.showInputBox({
          prompt: "Enter symbol name to analyze impact:",
          placeHolder: "e.g. parse_file or processPayment",
        });
      }

      if (!symbolName) return;

      vscode.window.withProgress(
        {
          location: vscode.ProgressLocation.Notification,
          title: `EMOCIS: Analyzing impact for '${symbolName}'...`,
        },
        async () => {
          const impact = await emocisClient!.getImpact(symbolName!);
          if (!impact) {
            vscode.window.showInformationMessage(`No impact detected for '${symbolName}'.`);
            return;
          }

          const direct = impact.direct_callers?.length || 0;
          const indirect = impact.indirect_callers?.length || 0;
          const totalRadius = direct + indirect;

          vscode.window.showInformationMessage(
            `Impact Radius for '${symbolName}': ${totalRadius} dependent functions (${direct} direct, ${indirect} transitive).`,
            "View in Graph"
          ).then((selection) => {
            if (selection === "View in Graph") {
              GraphVisualizerPanel.createOrShow(context.extensionUri, emocisClient!, symbolName);
            }
          });
        }
      );
    })
  );

  // 6. Command: Search Engineering Memory
  context.subscriptions.push(
    vscode.commands.registerCommand("emocis.search", async () => {
      const query = await vscode.window.showInputBox({
        prompt: "Search architectural decisions, constraints, warnings...",
        placeHolder: "e.g. idempotency, precision, physics",
      });

      if (!query) return;

      const results = await emocisClient!.search(query);
      if (!Array.isArray(results) || results.length === 0) {
        vscode.window.showInformationMessage(`No memory items found matching '${query}'.`);
        return;
      }

      const quickPickItems = results.map((item: any) => ({
        label: `[${item.knowledge_type?.toUpperCase()}] ${item.title}`,
        description: item.affects?.join(", "),
        detail: item.content,
        item,
      }));

      const selected = await vscode.window.showQuickPick(quickPickItems, {
        matchOnDescription: true,
        matchOnDetail: true,
        placeHolder: "Select a memory item to navigate to source",
      });

      if (selected && selected.item) {
        const file = selected.item.evidence?.[0]?.file || selected.item.affects?.[0];
        const line = selected.item.evidence?.[0]?.start_line ?? 1;
        if (file) {
          const docUri = vscode.Uri.file(`${workspaceRoot}/${file}`);
          const doc = await vscode.workspace.openTextDocument(docUri);
          const editor = await vscode.window.showTextDocument(doc);
          const range = new vscode.Range(Math.max(0, line - 1), 0, Math.max(0, line - 1), 0);
          editor.selection = new vscode.Selection(range.start, range.end);
          editor.revealRange(range, vscode.TextEditorRevealType.InCenter);
        }
      }
    })
  );

  // 7. Command: Copy Minimal Context Package for AI
  context.subscriptions.push(
    vscode.commands.registerCommand("emocis.copyContext", async () => {
      const intent = await vscode.window.showInputBox({
        prompt: "Describe your coding task or feature intent:",
        placeHolder: "e.g. Refactor payment retry logic with exponential backoff",
      });

      if (!intent) return;

      const ctx = await emocisClient!.getContext(intent);
      const text = typeof ctx === "string" ? ctx : JSON.stringify(ctx, null, 2);

      await vscode.env.clipboard.writeText(text);
      vscode.window.showInformationMessage(
        "⚡ EMOCIS Minimal Context Package copied to clipboard! Paste directly into Cursor / Claude / ChatGPT."
      );
    })
  );

  // 8. Command: Architecture Audit
  context.subscriptions.push(
    vscode.commands.registerCommand("emocis.audit", async () => {
      const audit = await emocisClient!.getAudit();
      const count = Array.isArray(audit) ? audit.length : 0;
      vscode.window.showInformationMessage(
        `EMOCIS Architecture Audit: ${count} active drift/boundary issues detected. View details in sidebar.`
      );
      healthProvider.refresh();
    })
  );
}

export function deactivate() {
  if (emocisClient) {
    emocisClient.dispose();
  }
}
