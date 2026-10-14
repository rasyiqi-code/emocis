import * as vscode from "vscode";
import * as path from "path";
import { EmocisClient } from "../mcpClient";

export interface KnowledgeItem {
  id: string;
  project_id: string;
  knowledge_type: string;
  title: string;
  content: string;
  status: string;
  confidence: number;
  affects: string[];
  tags: string[];
  evidence?: Array<{
    file?: string;
    start_line?: number;
    end_line?: number;
    snippet?: string;
  }>;
}

export class MemoryItem extends vscode.TreeItem {
  constructor(
    public readonly label: string,
    public readonly collapsibleState: vscode.TreeItemCollapsibleState,
    public readonly category?: string,
    public readonly data?: KnowledgeItem,
    public readonly filePath?: string,
    public readonly line?: number
  ) {
    super(label, collapsibleState);

    if (data) {
      this.tooltip = `${data.title}\n\n${data.content}\n\nTags: ${data.tags.join(", ")}`;
      let file: string | undefined;
      let startLine = 1;

      if (data.evidence && data.evidence.length > 0) {
        for (const ev of data.evidence) {
          if (ev.file) {
            file = ev.file;
            startLine = ev.start_line ?? 1;
            break;
          }
        }
      }
      if (!file && data.affects && data.affects.length > 0) {
        file = data.affects[0];
      }

      if (file) {
        this.description = `${file}:${startLine}`;
        this.command = {
          command: "emocis.openLocation",
          title: "Open Location",
          arguments: [file, startLine]
        };
      } else {
        this.description = data.affects.join(", ");
      }

      switch (data.knowledge_type?.toLowerCase()) {
        case "decision":
          this.iconPath = new vscode.ThemeIcon("check", new vscode.ThemeColor("charts.blue"));
          break;
        case "constraint":
          this.iconPath = new vscode.ThemeIcon("lock", new vscode.ThemeColor("charts.yellow"));
          break;
        case "warning":
        case "caveat":
          this.iconPath = new vscode.ThemeIcon("warning", new vscode.ThemeColor("charts.orange"));
          break;
        case "business_rule":
        case "businessrule":
          this.iconPath = new vscode.ThemeIcon("law", new vscode.ThemeColor("charts.purple"));
          break;
        case "failed_attempt":
          this.iconPath = new vscode.ThemeIcon("error", new vscode.ThemeColor("charts.red"));
          break;
        default:
          this.iconPath = new vscode.ThemeIcon("bookmark", new vscode.ThemeColor("charts.green"));
      }
    } else {
      // Category folder
      this.iconPath = new vscode.ThemeIcon("folder");
    }
  }
}

export class MemoryTreeProvider implements vscode.TreeDataProvider<MemoryItem> {
  private _onDidChangeTreeData: vscode.EventEmitter<MemoryItem | undefined | null | void> = new vscode.EventEmitter<MemoryItem | undefined | null | void>();
  readonly onDidChangeTreeData: vscode.Event<MemoryItem | undefined | null | void> = this._onDidChangeTreeData.event;

  private client: EmocisClient;
  private cachedItems: KnowledgeItem[] = [];

  constructor(client: EmocisClient) {
    this.client = client;
  }

  refresh(): void {
    this._onDidChangeTreeData.fire();
  }

  getTreeItem(element: MemoryItem): vscode.TreeItem {
    return element;
  }

  async getChildren(element?: MemoryItem): Promise<MemoryItem[]> {
    if (!element) {
      // Root categories
      try {
        const results = await this.client.search("");
        this.cachedItems = Array.isArray(results) ? results : [];
      } catch (e) {
        console.error("Error fetching memory items:", e);
        this.cachedItems = [];
      }

      const categories = [
        { label: "Decisions", type: "decision" },
        { label: "Constraints", type: "constraint" },
        { label: "Business Rules", type: "business_rule" },
        { label: "Warnings & Caveats", type: "warning" },
        { label: "Failed Attempts", type: "failed_attempt" },
        { label: "Architecture", type: "architecture" },
      ];

      return categories.map((cat) => {
        const count = this.cachedItems.filter(
          (item) => item.knowledge_type?.toLowerCase() === cat.type.toLowerCase() ||
                    item.tags?.includes(cat.type)
        ).length;

        return new MemoryItem(
          `${cat.label} (${count})`,
          count > 0 ? vscode.TreeItemCollapsibleState.Collapsed : vscode.TreeItemCollapsibleState.None,
          cat.type
        );
      });
    }

    // Children of a category
    if (element.category) {
      const filtered = this.cachedItems.filter(
        (item) => item.knowledge_type?.toLowerCase() === element.category?.toLowerCase() ||
                  item.tags?.includes(element.category || "")
      );

      return filtered.map((item) => {
        return new MemoryItem(
          item.title,
          vscode.TreeItemCollapsibleState.None,
          undefined,
          item
        );
      });
    }

    return [];
  }
}
