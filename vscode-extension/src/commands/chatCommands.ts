import * as vscode from "vscode";
import { EmocisClient } from "../mcpClient";
import { HealthItem } from "../treeViews/healthTreeProvider";
import { MemoryItem } from "../treeViews/memoryTreeProvider";
import { displayScore } from "../contracts";
import { graphRows } from "../treeViews/healthRows";
import { sendToAiChat } from "../services/aiChatService";

export function registerChatCommands(
  context: vscode.ExtensionContext,
  getClient: () => EmocisClient | undefined
): void {
  // 1. Send single issue to AI Chat
  context.subscriptions.push(
    vscode.commands.registerCommand("emocis.sendIssueToChat", async (item?: HealthItem) => {
      let prompt = "";
      if (item && item.rawData) {
        const issue = item.rawData;
        prompt = `Tolong analisa dan berikan rekomendasi perbaikan untuk isu arsitektur/drift berikut dari EMOCIS:
- **Isu**: ${issue.title}
- **Lokasi**: ${issue.location || 'Tidak spesifik'}
- **Deskripsi**: ${issue.description || ''}
${issue.recommendation ? `- **Rekomendasi**: ${issue.recommendation}\n` : ''}
Apakah ini kode mati yang aman dihapus, atau bagaimana cara memperbaikinya dengan best practice?`;
      } else if (item && item.label) {
        prompt = `Tolong analisa isu arsitektur berikut dari EMOCIS:
${item.label}: ${item.details || item.value || ''}`;
      } else {
        const client = getClient();
        if (!client) return;
        try {
          const audit = await client.getAudit();
          if (!audit.violations || audit.violations.length === 0) {
            vscode.window.showInformationMessage("Tidak ada isu drift arsitektur yang terdeteksi.");
            return;
          }
          const picks = audit.violations.map((v) => ({
            label: v.title,
            description: v.location || "",
            detail: v.description,
            issue: v,
          }));
          const selected = await vscode.window.showQuickPick(picks, {
            placeHolder: "Pilih isu drift untuk dikirim ke AI Copilot Chat",
          });
          if (selected) {
            const issue = selected.issue;
            prompt = `Tolong analisa dan berikan rekomendasi perbaikan untuk isu arsitektur/drift berikut dari EMOCIS:
- **Isu**: ${issue.title}
- **Lokasi**: ${issue.location || 'Tidak spesifik'}
- **Deskripsi**: ${issue.description || ''}
${issue.recommendation ? `- **Rekomendasi**: ${issue.recommendation}\n` : ''}
Apakah ini kode mati yang aman dihapus, atau bagaimana cara memperbaikinya dengan best practice?`;
          } else {
            return;
          }
        } catch (err) {
          vscode.window.showErrorMessage(`Gagal mengambil data audit: ${err}`);
          return;
        }
      }

      if (prompt) {
        await sendToAiChat(prompt);
      }
    })
  );

  // 2. Send architecture metric to AI Chat
  context.subscriptions.push(
    vscode.commands.registerCommand("emocis.sendMetricToChat", async (item?: HealthItem) => {
      let metric = item?.rawData?.label || item?.label;
      let val = item?.rawData?.value || item?.value;
      let details = item?.rawData?.details || item?.details;

      if (!metric) {
        const client = getClient();
        if (!client) return;
        try {
          const health = await client.getHealth();
          if (!health) {
            vscode.window.showInformationMessage("Data kesehatan arsitektur belum tersedia.");
            return;
          }
          const picks: Array<{ label: string; value: string; details: string }> = [];
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
          const selected = await vscode.window.showQuickPick(picks, {
            placeHolder: "Pilih metrik arsitektur untuk dianalisis oleh AI Copilot",
          });
          if (selected) {
            metric = selected.label;
            val = selected.value;
            details = selected.details;
          } else {
            return;
          }
        } catch (err) {
          vscode.window.showErrorMessage(`Gagal mengambil data kesehatan arsitektur: ${err}`);
          return;
        }
      }

      const prompt = `Tolong analisa metrik arsitektur berikut dari EMOCIS:
- **Metrik**: ${metric} (${val || 'N/A'})
- **Penjelasan**: ${details || ''}
Bagaimana strategi dan langkah perbaikan teknis terbaik untuk meningkatkan metrik ini pada arsitektur codebase?`;
      await sendToAiChat(prompt);
    })
  );

  // 3. Send all drift issues to AI Chat
  context.subscriptions.push(
    vscode.commands.registerCommand("emocis.sendAllDriftToChat", async () => {
      const client = getClient();
      if (!client) return;
      try {
        const audit = await client.getAudit();
        let prompt = `Berikut adalah daftar ${audit.total_violations} isu arsitektur dan drift yang terdeteksi oleh EMOCIS pada codebase:\n\n`;
        for (const [idx, v] of audit.violations.slice(0, 15).entries()) {
          prompt += `${idx + 1}. **${v.title}**\n   - Lokasi: ${v.location || '-'}\n   - Deskripsi: ${v.description}\n`;
        }
        if (audit.violations.length > 15) {
          prompt += `\n... dan ${audit.violations.length - 15} isu lainnya.\n`;
        }
        prompt += `\nTolong tinjau temuan-temuan di atas, kelompokkan berdasarkan tingkat urgensi, dan berikan rencana tindakan perbaikan.`;
        await sendToAiChat(prompt);
      } catch (err) {
        vscode.window.showErrorMessage(`Gagal mengambil data audit: ${err}`);
      }
    })
  );

  // 4. Send full audit and health report to AI Chat
  context.subscriptions.push(
    vscode.commands.registerCommand("emocis.sendFullAuditToChat", async () => {
      const client = getClient();
      if (!client) return;
      try {
        const audit = await client.getAudit();
        const health = await client.getHealth();

        let prompt = `Berikut adalah ringkasan Architecture Health & Drift Audit dari EMOCIS:\n\n`;
        if (health) {
          if (health.overall_score !== null) {
            prompt += `**Skor Arsitektur:**\n`;
            prompt += `- Overall Health Score: ${displayScore(health.overall_score)}\n`;
            if (health.modularity !== null) prompt += `- Modularity: ${displayScore(health.modularity)}\n`;
            if (health.cohesion !== null) prompt += `- Cohesion: ${displayScore(health.cohesion)}\n`;
            if (health.coupling !== null) prompt += `- Coupling: ${displayScore(health.coupling)}\n`;
            if (health.boundary_integrity !== null) prompt += `- Boundary Integrity: ${displayScore(health.boundary_integrity)}\n`;
            prompt += `\n`;
          }
          if (health.graph) {
            prompt += `**Pengukuran Graf Statis:**\n`;
            prompt += `- Status Analisis: ${health.status} (${health.explanation})\n`;
            for (const r of graphRows(health.graph)) {
              prompt += `- ${r.label}: ${r.value}\n`;
            }
            prompt += `\n`;
          }
        }
        prompt += `**Daftar Temuan Drift (${audit.total_violations} isu):**\n`;
        for (const [idx, v] of audit.violations.slice(0, 10).entries()) {
          prompt += `${idx + 1}. **${v.title}** (${v.location || '-'})\n   ${v.description}\n`;
        }
        if (audit.violations.length > 10) {
          prompt += `... dan ${audit.violations.length - 10} isu lainnya.\n`;
        }
        prompt += `\nTolong berikan ulasan komprehensif mengenai kondisi codebase saat ini dan rekomendasi refaktorisasi prioritas utama.`;
        await sendToAiChat(prompt);
      } catch (err) {
        vscode.window.showErrorMessage(`Gagal memuat audit arsitektur: ${err}`);
      }
    })
  );

  // 5. Send memory item to AI Chat
  context.subscriptions.push(
    vscode.commands.registerCommand("emocis.sendMemoryToChat", async (item: MemoryItem) => {
      if (!item?.data) return;
      const d = item.data;
      const prompt = `Berikut adalah catatan Engineering Memory ([${d.knowledge_type?.toUpperCase()}]) dari EMOCIS:
- **Judul**: ${d.title}
- **Konten**: ${d.content}
- **Modul Terkait**: ${d.affects?.join(", ") || '-'}
- **Tags**: ${d.tags?.join(", ") || '-'}

Tolong pastikan kode atau fitur yang sedang kita kerjakan tetap selaras dengan keputusan/batasan arsitektur di atas.`;
      await sendToAiChat(prompt);
    })
  );
}
