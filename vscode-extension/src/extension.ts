import * as vscode from "vscode";
import { EmocisClient } from "./mcpClient";
import { MemoryTreeProvider } from "./treeViews/memoryTreeProvider";
import { HealthTreeProvider } from "./treeViews/healthTreeProvider";
import { MemoryCodeLensProvider } from "./codelens/memoryCodeLensProvider";
import { registerNavigationCommands } from "./commands/navigationCommands";
import { registerMemoryCommands } from "./commands/memoryCommands";
import { registerChatCommands } from "./commands/chatCommands";
import { registerConfigCommands } from "./commands/configCommands";
import { setupAutoRefreshWatcher } from "./services/fileWatcherService";
import { ensureEmocisSkill } from "./services/skillService";

let emocisClient: EmocisClient | undefined;

export function activate(context: vscode.ExtensionContext) {
  const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || process.cwd();
  const outputChannel = vscode.window.createOutputChannel("EMOCIS");
  outputChannel.appendLine(`[EMOCIS] Extension activated for workspace: ${workspaceRoot}`);

  // Automatically ensure SKILL.md is present for AI Coding Agents (Antigravity, Cursor, etc.)
  ensureEmocisSkill(workspaceRoot, outputChannel);

  emocisClient = new EmocisClient(workspaceRoot);
  const getClient = () => emocisClient;

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
    vscode.languages.registerCodeLensProvider(supportedLanguages, new MemoryCodeLensProvider())
  );

  // 3. Register Modular Command Handlers
  registerNavigationCommands(context, workspaceRoot, outputChannel);
  registerMemoryCommands(context, workspaceRoot, getClient, memoryProvider, healthProvider);
  registerChatCommands(context, getClient);
  registerConfigCommands(context, workspaceRoot);

  // 4. Background File Watcher
  setupAutoRefreshWatcher(context, outputChannel, () => {
    memoryProvider.refresh();
    healthProvider.refresh();
  });
}

export function deactivate() {
  if (emocisClient) {
    emocisClient.dispose();
    emocisClient = undefined;
  }
}
