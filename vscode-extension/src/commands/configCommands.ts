import * as vscode from "vscode";
import * as path from "path";
import * as fs from "fs";

export function registerConfigCommands(
  context: vscode.ExtensionContext,
  workspaceRoot: string
): void {
  context.subscriptions.push(
    vscode.commands.registerCommand("emocis.setupAntigravityMcp", async () => {
      const configObj = {
        mcpServers: {
          emocis: {
            command: "emocis",
            args: ["mcp"]
          }
        }
      };
      const formatted = JSON.stringify(configObj, null, 2);
      const configPath = path.join(workspaceRoot, "emocis-mcp-config.json");
      const cursorDir = path.join(workspaceRoot, ".cursor");
      const cursorPath = path.join(cursorDir, "mcp.json");

      try {
        fs.writeFileSync(configPath, formatted, "utf8");
        if (fs.existsSync(cursorDir)) {
          fs.writeFileSync(cursorPath, formatted, "utf8");
        }
        await vscode.env.clipboard.writeText(formatted);
        vscode.window.showInformationMessage(
          "✔ EMOCIS MCP configuration created & copied to clipboard! Ready for Antigravity IDE and Cursor AI Agent.",
          "Open Config"
        ).then((sel) => {
          if (sel === "Open Config") {
            vscode.workspace.openTextDocument(configPath).then(doc => vscode.window.showTextDocument(doc));
          }
        });
      } catch {
        await vscode.env.clipboard.writeText(formatted);
        vscode.window.showInformationMessage(
          "✔ EMOCIS MCP configuration snippet copied to clipboard! Paste it into Antigravity or Cursor mcp_config.json."
        );
      }
    })
  );
}
