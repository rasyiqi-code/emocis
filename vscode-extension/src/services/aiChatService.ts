import * as vscode from "vscode";

export async function sendToAiChat(prompt: string): Promise<void> {
  await vscode.env.clipboard.writeText(prompt);

  let opened = false;
  try {
    await vscode.commands.executeCommand("workbench.action.chat.open", { query: prompt });
    opened = true;
  } catch {
    try {
      await vscode.commands.executeCommand("workbench.action.quickchat.open", { query: prompt });
      opened = true;
    } catch {
      try {
        await vscode.commands.executeCommand("composer.open");
        opened = true;
      } catch {
        try {
          await vscode.commands.executeCommand("workbench.panel.chat.view.copilot.focus");
          opened = true;
        } catch {}
      }
    }
  }

  if (opened) {
    vscode.window.showInformationMessage("💬 Terkirim langsung ke AI Copilot Chat! (Teks juga tersimpan di clipboard)");
  } else {
    vscode.window.showInformationMessage("💬 Teks telah disalin ke clipboard! Tekan Ctrl+L atau buka AI Chat panel untuk menempelkan.");
  }
}
