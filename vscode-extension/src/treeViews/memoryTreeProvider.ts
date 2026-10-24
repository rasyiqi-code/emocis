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
    evidence_type?: string;
  }>;
}

function isDocumentationItem(item: KnowledgeItem): boolean {
  if (
    item.evidence &&
    item.evidence.some((e) => {
      const f = e.file?.toLowerCase() || "";
      return f.endsWith(".md") || e.evidence_type === "documentation";
    })
  ) {
    return true;
  }
  if (item.tags && item.tags.some((t) => t.toLowerCase() === "markdown")) {
    return true;
  }
  if (item.affects && item.affects.some((a) => a.toLowerCase().endsWith(".md"))) {
    return true;
  }
  return false;
}

export class MemoryItem extends vscode.TreeItem {
  constructor(
    public readonly label: string,
    public readonly collapsibleState: vscode.TreeItemCollapsibleState,
    public readonly rootGroup?: "docs" | "code",
    public readonly category?: string,
    public readonly data?: KnowledgeItem,
    public readonly filePath?: string,
    public readonly line?: number
  ) {
    super(label, collapsibleState);

    if (data) {
      this.contextValue = "memoryItem";
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
          arguments: [file, startLine],
        };
      } else {
        this.description = data.affects.join(", ");
      }

      const tags = data.tags?.map((t) => t.toLowerCase()) || [];
      const isBugfix = tags.includes("bugfix") || tags.includes("bug fix") || data.title.startsWith("BUG FIX");

      if (isBugfix) {
        this.iconPath = new vscode.ThemeIcon("debug-restart", new vscode.ThemeColor("charts.red"));
      } else {
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
          case "architecture":
            this.iconPath = new vscode.ThemeIcon("organization", new vscode.ThemeColor("charts.green"));
            break;
          default:
            this.iconPath = new vscode.ThemeIcon("bookmark", new vscode.ThemeColor("charts.green"));
        }
      }
    } else if (rootGroup && !category) {
      // Top-level root folder
      if (rootGroup === "docs") {
        this.iconPath = new vscode.ThemeIcon("book", new vscode.ThemeColor("charts.blue"));
      } else {
        this.iconPath = new vscode.ThemeIcon("code", new vscode.ThemeColor("charts.green"));
      }
    } else {
      // Subcategory folder
      this.iconPath = new vscode.ThemeIcon("folder");
    }
  }
}

export class MemoryTreeProvider implements vscode.TreeDataProvider<MemoryItem> {
  private _onDidChangeTreeData: vscode.EventEmitter<MemoryItem | undefined | null | void> =
    new vscode.EventEmitter<MemoryItem | undefined | null | void>();
  readonly onDidChangeTreeData: vscode.Event<MemoryItem | undefined | null | void> =
    this._onDidChangeTreeData.event;

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
      // Root level: separate Documentation from Code Annotations
      try {
        const results = await this.client.search("");
        this.cachedItems = Array.isArray(results) ? results : [];
      } catch (e) {
        console.error("Error fetching memory items:", e);
        this.cachedItems = [];
      }

      const docItems = this.cachedItems.filter(isDocumentationItem);
      const codeItems = this.cachedItems.filter((item) => !isDocumentationItem(item));

      return [
        new MemoryItem(
          `📄 Project Documentation & Specs (${docItems.length})`,
          docItems.length > 0 ? vscode.TreeItemCollapsibleState.Expanded : vscode.TreeItemCollapsibleState.None,
          "docs"
        ),
        new MemoryItem(
          `💻 Source Code Annotations (${codeItems.length})`,
          codeItems.length > 0 ? vscode.TreeItemCollapsibleState.Expanded : vscode.TreeItemCollapsibleState.None,
          "code"
        ),
      ];
    }

    const matchesCategory = (item: KnowledgeItem, type: string) => {
      const t = item.knowledge_type?.toLowerCase() || "";
      const tags = item.tags?.map((x) => x.toLowerCase()) || [];
      if (type === "bugfix") {
        return tags.includes("bugfix") || tags.includes("bug fix") || item.title.startsWith("BUG FIX");
      }
      if (type === "warning") {
        const isBugfix = tags.includes("bugfix") || tags.includes("bug fix") || item.title.startsWith("BUG FIX");
        return !isBugfix && (t === "warning" || tags.includes("warning"));
      }
      return t === type || tags.includes(type);
    };

    // Subcategories under a Root folder
    if (element.rootGroup && !element.category) {
      const isDocs = element.rootGroup === "docs";
      const pool = this.cachedItems.filter((item) =>
        isDocs ? isDocumentationItem(item) : !isDocumentationItem(item)
      );

      const categories = isDocs
        ? [
            { label: "Architecture & System Design", type: "architecture" },
            { label: "Architectural Decisions (ADR)", type: "decision" },
            { label: "Constraints & Security Specs", type: "constraint" },
            { label: "Business Rules & Policies", type: "business_rule" },
            { label: "Bug Reports & Caveats", type: "warning" },
            { label: "Worklog & Changelogs", type: "worklog" },
          ]
        : [
            { label: "Decisions & Rationale", type: "decision" },
            { label: "System Constraints & Security", type: "constraint" },
            { label: "Business Logic Rules", type: "business_rule" },
            { label: "Warnings & Workarounds", type: "warning" },
            { label: "Bug Fix Notes", type: "bugfix" },
            { label: "Architecture Notes", type: "architecture" },
            { label: "Failed Attempts", type: "failed_attempt" },
            { label: "Worklog", type: "worklog" },
          ];

      return categories
        .map((cat) => {
          const count = pool.filter((item) => matchesCategory(item, cat.type)).length;
          return { cat, count };
        })
        .filter(({ count }) => count > 0)
        .map(({ cat, count }) => {
          return new MemoryItem(
            `${cat.label} (${count})`,
            vscode.TreeItemCollapsibleState.Collapsed,
            element.rootGroup,
            cat.type
          );
        });
    }

    // Leaf KnowledgeItems under a subcategory
    if (element.rootGroup && element.category) {
      const isDocs = element.rootGroup === "docs";
      const pool = this.cachedItems.filter((item) =>
        isDocs ? isDocumentationItem(item) : !isDocumentationItem(item)
      );

      const filtered = pool.filter((item) => matchesCategory(item, element.category!));

      return filtered.map((item) => {
        return new MemoryItem(
          item.title,
          vscode.TreeItemCollapsibleState.None,
          element.rootGroup,
          element.category,
          item
        );
      });
    }

    return [];
  }
}
