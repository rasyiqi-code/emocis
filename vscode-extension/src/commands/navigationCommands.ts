import * as vscode from "vscode";
import * as path from "path";
import * as fs from "fs";

export function registerNavigationCommands(
  context: vscode.ExtensionContext,
  workspaceRoot: string,
  outputChannel: vscode.OutputChannel
): void {
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
}
