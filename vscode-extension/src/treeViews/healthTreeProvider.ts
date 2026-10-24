import { displayScore } from "../contracts";
import { graphRows } from "./healthRows";
import * as vscode from "vscode";
import { EmocisClient } from "../mcpClient";

export class HealthItem extends vscode.TreeItem {
  constructor(
    public readonly label: string,
    public readonly value?: string,
    public readonly iconName?: string,
    public readonly color?: string,
    public readonly details?: string,
    public readonly collapsibleState: vscode.TreeItemCollapsibleState = vscode.TreeItemCollapsibleState.None,
    public readonly rawData?: any
  ) {
    super(label, collapsibleState);

    if (value) {
      this.description = value;
    }
    if (details) {
      this.tooltip = details;
    }
    if (iconName) {
      this.iconPath = new vscode.ThemeIcon(
        iconName,
        color ? new vscode.ThemeColor(color) : undefined
      );
    }
  }
}

export class HealthTreeProvider implements vscode.TreeDataProvider<HealthItem> {
  private _onDidChangeTreeData: vscode.EventEmitter<HealthItem | undefined | null | void> = new vscode.EventEmitter<HealthItem | undefined | null | void>();
  readonly onDidChangeTreeData: vscode.Event<HealthItem | undefined | null | void> = this._onDidChangeTreeData.event;

  private client: EmocisClient;

  constructor(client: EmocisClient) {
    this.client = client;
  }

  refresh(): void {
    this._onDidChangeTreeData.fire();
  }

  getTreeItem(element: HealthItem): vscode.TreeItem {
    return element;
  }

  async getChildren(): Promise<HealthItem[]> {
    const items: HealthItem[] = [];

    try {
      const health = await this.client.getHealth();
      if (health) {
        items.push(new HealthItem("Assessment", health.status, "info", undefined, health.explanation));
        if (health.graph) {
          for (const row of graphRows(health.graph)) {
            const graphItem = new HealthItem(
              row.label,
              row.value,
              "graph",
              "charts.blue",
              `${row.details}\n\n💡 Klik baris atau tombol chat untuk kirim analisis metrik ini ke AI Copilot.`,
              vscode.TreeItemCollapsibleState.None,
              { label: row.label, value: row.value, details: row.details }
            );
            graphItem.contextValue = "healthMetric";
            graphItem.command = {
              command: "emocis.sendMetricToChat",
              title: "Analyze Metric with AI Copilot",
              arguments: [graphItem],
            };
            items.push(graphItem);
          }
        }

        // Only display quality scores if calibrated and measured by the engine
        if (health.overall_score !== null) {
          const score = health.overall_score;
          const icon = score >= 80 ? "pass-filled" : score >= 60 ? "warning" : "error";
          const color = score >= 80 ? "testing.iconPassed" : score >= 60 ? "charts.yellow" : "testing.iconFailed";
          const overallItem = new HealthItem("Overall Health Score", displayScore(score), icon, color, "Aggregated codebase architectural health\n\n💡 Klik baris atau tombol chat untuk kirim analisis metrik ini ke AI Copilot.", vscode.TreeItemCollapsibleState.None, { label: "Overall Health Score", value: displayScore(score), details: "Aggregated codebase architectural health" });
          overallItem.contextValue = "healthMetric";
          overallItem.command = {
            command: "emocis.sendMetricToChat",
            title: "Analyze Metric with AI Copilot",
            arguments: [overallItem],
          };
          items.push(overallItem);
        }

        if (health.modularity !== null) {
          const modItem = new HealthItem("Modularity", displayScore(health.modularity), "package", "charts.blue", "Single responsibility and module segregation\n\n💡 Klik baris atau tombol chat untuk kirim analisis metrik ini ke AI Copilot.", vscode.TreeItemCollapsibleState.None, { label: "Modularity", value: displayScore(health.modularity), details: "Single responsibility and module segregation" });
          modItem.contextValue = "healthMetric";
          modItem.command = {
            command: "emocis.sendMetricToChat",
            title: "Analyze Metric with AI Copilot",
            arguments: [modItem],
          };
          items.push(modItem);
        }

        if (health.cohesion !== null) {
          const cohItem = new HealthItem("Cohesion", displayScore(health.cohesion), "git-merge", "charts.green", "Internal relatedness of functions within modules\n\n💡 Klik baris atau tombol chat untuk kirim analisis metrik ini ke AI Copilot.", vscode.TreeItemCollapsibleState.None, { label: "Cohesion", value: displayScore(health.cohesion), details: "Internal relatedness of functions within modules" });
          cohItem.contextValue = "healthMetric";
          cohItem.command = {
            command: "emocis.sendMetricToChat",
            title: "Analyze Metric with AI Copilot",
            arguments: [cohItem],
          };
          items.push(cohItem);
        }

        if (health.coupling !== null) {
          const coupItem = new HealthItem("Coupling", displayScore(health.coupling), "references", "charts.purple", "Degree of interdependence between modules\n\n💡 Klik baris atau tombol chat untuk kirim analisis metrik ini ke AI Copilot.", vscode.TreeItemCollapsibleState.None, { label: "Coupling", value: displayScore(health.coupling), details: "Degree of interdependence between modules" });
          coupItem.contextValue = "healthMetric";
          coupItem.command = {
            command: "emocis.sendMetricToChat",
            title: "Analyze Metric with AI Copilot",
            arguments: [coupItem],
          };
          items.push(coupItem);
        }

        if (health.boundary_integrity !== null) {
          const boundItem = new HealthItem("Boundary Integrity", displayScore(health.boundary_integrity), "shield", "charts.orange", "Enforcement of architectural layer boundaries\n\n💡 Klik baris atau tombol chat untuk kirim analisis metrik ini ke AI Copilot.", vscode.TreeItemCollapsibleState.None, { label: "Boundary Integrity", value: displayScore(health.boundary_integrity), details: "Enforcement of architectural layer boundaries" });
          boundItem.contextValue = "healthMetric";
          boundItem.command = {
            command: "emocis.sendMetricToChat",
            title: "Analyze Metric with AI Copilot",
            arguments: [boundItem],
          };
          items.push(boundItem);
        }
      }

      const audit = await this.client.getAudit();
      if (audit.total_violations > 0) {
        const summaryItem = new HealthItem("Active Drift Issues", `${audit.total_violations} detected`, "alert", "charts.red", "Architectural or documentation drift detected.\n\n💡 Klik baris ini atau tombol chat di kanan untuk kirim semua temuan ke AI Copilot Chat.", vscode.TreeItemCollapsibleState.None, { audit });
        summaryItem.contextValue = "healthGroup";
        summaryItem.command = {
          command: "emocis.sendAllDriftToChat",
          title: "Send All Drift Issues to AI Chat",
        };
        items.push(summaryItem);

        for (const issue of audit.violations) {
          const title = issue.title || "Drift Warning";
          const desc = issue.description || issue.location || "";
          let file: string | undefined;
          let line = 1;
          if (issue.location) {
            const lastColon = issue.location.lastIndexOf(":");
            if (lastColon !== -1) {
              file = issue.location.slice(0, lastColon);
              const parsed = parseInt(issue.location.slice(lastColon + 1), 10);
              if (!isNaN(parsed)) line = parsed;
            } else {
              file = issue.location;
            }
          }

          const issueItem = new HealthItem(
            `  • ${title}`,
            issue.location ? `${issue.location} - ${desc}` : desc,
            "circle-filled",
            "charts.red",
            `[${title}]\n\nLokasi: ${issue.location || 'Tidak spesifik'}\n\nDeskripsi: ${desc}\n\n💡 Klik baris untuk buka kode sumber di editor.\n💬 Klik tombol chat di sebelah kanan (atau klik kanan) untuk langsung kirim ke AI Copilot Chat.`,
            vscode.TreeItemCollapsibleState.None,
            issue
          );
          issueItem.contextValue = "healthIssue";
          if (file) {
            issueItem.command = {
              command: "emocis.openLocation",
              title: "Open Code Location",
              arguments: [file, line],
            };
          } else {
            issueItem.command = {
              command: "emocis.sendIssueToChat",
              title: "Send Issue to AI Chat",
              arguments: [issueItem],
            };
          }
          items.push(issueItem);
        }
      } else {
        items.push(new HealthItem("Drift Audit", "No heuristic findings", "check", "testing.iconPassed", "No findings in the indexed graph; this does not certify architecture health"));
      }
    } catch (err) {
      items.push(new HealthItem("Engine Status", "Unavailable", "error", "charts.red", String(err)));
    }

    return items;
  }
}
