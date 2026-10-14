import * as vscode from "vscode";
import { EmocisClient } from "../mcpClient";

export class HealthItem extends vscode.TreeItem {
  constructor(
    public readonly label: string,
    public readonly value?: string,
    public readonly iconName?: string,
    public readonly color?: string,
    public readonly details?: string
  ) {
    super(label, vscode.TreeItemCollapsibleState.None);

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
        const score = health.overall_score ?? 0;
        const icon = score >= 80 ? "pass-filled" : score >= 60 ? "warning" : "error";
        const color = score >= 80 ? "testing.iconPassed" : score >= 60 ? "charts.yellow" : "testing.iconFailed";

        items.push(new HealthItem("Overall Health Score", `${score}/100`, icon, color, "Aggregated codebase architectural health"));
        items.push(new HealthItem("Modularity", `${health.modularity_score ?? 0}/100`, "package", "charts.blue", "Single responsibility and module segregation"));
        items.push(new HealthItem("Cohesion", `${health.cohesion_score ?? 0}/100`, "git-merge", "charts.green", "Internal relatedness of functions within modules"));
        items.push(new HealthItem("Coupling", `${health.coupling_score ?? 0}/100`, "references", "charts.purple", "Degree of interdependence between modules"));
        items.push(new HealthItem("Boundary Integrity", `${health.boundary_integrity_score ?? 0}/100`, "shield", "charts.orange", "Enforcement of architectural layer boundaries"));
      }

      const audit = await this.client.getAudit();
      if (Array.isArray(audit) && audit.length > 0) {
        items.push(new HealthItem("Active Drift Issues", `${audit.length} detected`, "alert", "charts.red", "Architectural or documentation drift detected"));
        for (const issue of audit.slice(0, 5)) {
          const title = issue.title || issue.issue_type || "Drift Warning";
          const desc = issue.description || issue.file_path || "";
          items.push(new HealthItem(`  • ${title}`, desc, "circle-filled", "charts.red", desc));
        }
      } else {
        items.push(new HealthItem("Drift Audit", "Clean (0 issues)", "check", "testing.iconPassed", "No architectural drift or dead logic detected"));
      }
    } catch (err) {
      items.push(new HealthItem("Engine Status", "Connecting...", "sync~spin", "charts.blue"));
    }

    return items;
  }
}
