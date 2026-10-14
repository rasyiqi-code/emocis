import { spawn, ChildProcessWithoutNullStreams } from "child_process";
import * as path from "path";
import * as fs from "fs";

export interface McpToolResponse {
  content: Array<{
    type: string;
    text: string;
  }>;
}

export class EmocisClient {
  private workspaceRoot: string;
  private process: ChildProcessWithoutNullStreams | null = null;
  private requestId = 0;
  private pendingRequests = new Map<number, { resolve: (res: any) => void; reject: (err: any) => void }>();
  private buffer = "";

  constructor(workspaceRoot: string) {
    this.workspaceRoot = workspaceRoot;
    this.initProcess();
  }

  private initProcess() {
    // Locate emocis mcp entrypoint: either local bin/mcp.js, or binary in target/debug or PATH
    const localMcpScript = path.join(this.workspaceRoot, "bin", "mcp.js");
    const targetBinary = path.join(this.workspaceRoot, "target", "debug", "emocis");

    if (fs.existsSync(localMcpScript)) {
      this.process = spawn("bun", [localMcpScript], {
        cwd: this.workspaceRoot,
        env: { ...process.env, RUST_LOG: "error" },
      });
    } else if (fs.existsSync(targetBinary)) {
      this.process = spawn(targetBinary, ["mcp"], {
        cwd: this.workspaceRoot,
        env: { ...process.env, RUST_LOG: "error" },
      });
    } else {
      // Fallback to npx/bunx emocis mcp
      this.process = spawn("bun", ["run", "mcp"], {
        cwd: this.workspaceRoot,
        env: { ...process.env, RUST_LOG: "error" },
      });
    }

    this.process.stdout.on("data", (chunk: Buffer) => {
      this.buffer += chunk.toString("utf8");
      const lines = this.buffer.split("\n");
      this.buffer = lines.pop() || "";

      for (const line of lines) {
        const trimmed = line.trim();
        if (!trimmed || !trimmed.startsWith("{")) continue;
        try {
          const json = JSON.parse(trimmed);
          if (json.id !== undefined && this.pendingRequests.has(json.id)) {
            const { resolve, reject } = this.pendingRequests.get(json.id)!;
            this.pendingRequests.delete(json.id);
            if (json.error) {
              reject(new Error(json.error.message || "MCP Error"));
            } else {
              resolve(json.result);
            }
          }
        } catch {
          // ignore non-json log lines
        }
      }
    });

    this.process.on("error", (err) => {
      console.error("[EMOCIS Client] Subprocess error:", err);
    });

    this.process.on("exit", (code) => {
      console.log(`[EMOCIS Client] Subprocess exited with code ${code}`);
      this.process = null;
    });
  }

  public async callTool(name: string, args: Record<string, any> = {}): Promise<string> {
    if (!this.process) {
      this.initProcess();
    }

    const id = ++this.requestId;
    const payload = JSON.stringify({
      jsonrpc: "2.0",
      id,
      method: "tools/call",
      params: {
        name,
        arguments: args,
      },
    }) + "\n";

    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        if (this.pendingRequests.has(id)) {
          this.pendingRequests.delete(id);
          reject(new Error(`Timeout calling EMOCIS tool '${name}'`));
        }
      }, 10000);

      this.pendingRequests.set(id, {
        resolve: (result: McpToolResponse) => {
          clearTimeout(timeout);
          const text = result?.content?.[0]?.text ?? "";
          resolve(text);
        },
        reject: (err) => {
          clearTimeout(timeout);
          reject(err);
        },
      });

      this.process?.stdin.write(payload);
    });
  }

  public async search(query: string) {
    const raw = await this.callTool("emocis_search", { query });
    try {
      return JSON.parse(raw);
    } catch {
      return [];
    }
  }

  public async getContext(intent: string) {
    const raw = await this.callTool("emocis_get_context", { intent });
    try {
      return JSON.parse(raw);
    } catch {
      return raw;
    }
  }

  public async getImpact(symbol: string) {
    const raw = await this.callTool("emocis_impact", { symbol });
    try {
      return JSON.parse(raw);
    } catch {
      return null;
    }
  }

  public async getCalls(fn: string) {
    const raw = await this.callTool("emocis_calls", { function: fn });
    try {
      return JSON.parse(raw);
    } catch {
      return [];
    }
  }

  public async getCallers(fn: string) {
    const raw = await this.callTool("emocis_callers", { function: fn });
    try {
      return JSON.parse(raw);
    } catch {
      return [];
    }
  }

  public async getHealth() {
    const raw = await this.callTool("emocis_health");
    try {
      return JSON.parse(raw);
    } catch {
      return null;
    }
  }

  public async getAudit() {
    const raw = await this.callTool("emocis_audit");
    try {
      return JSON.parse(raw);
    } catch {
      return [];
    }
  }

  public dispose() {
    if (this.process) {
      this.process.kill();
      this.process = null;
    }
  }
}
