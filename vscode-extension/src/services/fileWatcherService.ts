import * as vscode from "vscode";
import * as path from "path";

const WATCHED_EXTENSIONS = new Set([
  ".ts", ".tsx", ".js", ".jsx", ".rs", ".py", ".md", ".vue", ".svelte"
]);

export function setupAutoRefreshWatcher(
  context: vscode.ExtensionContext,
  outputChannel: vscode.OutputChannel,
  onRefresh: () => void
): void {
  let autoRefreshTimeout: NodeJS.Timeout | undefined;

  context.subscriptions.push(
    vscode.workspace.onDidSaveTextDocument((doc) => {
      const ext = path.extname(doc.fileName).toLowerCase();
      if (WATCHED_EXTENSIONS.has(ext)) {
        if (autoRefreshTimeout) clearTimeout(autoRefreshTimeout);
        autoRefreshTimeout = setTimeout(() => {
          outputChannel.appendLine(`[EMOCIS] Document saved: ${path.basename(doc.fileName)}. Refreshing memory...`);
          onRefresh();
        }, 1500);
      }
    })
  );
}
