import { parseAudit, parseHealth } from "./contracts";
import { spawn, ChildProcessWithoutNullStreams } from "child_process";
import * as path from "path";
import * as fs from "fs";

export interface McpToolResponse {
  isError?: boolean;
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
    const homeDir = process.env.HOME || "";
    const cargoBinary = path.join(homeDir, ".cargo", "bin", "emocis");
    const localTargetBinary = path.join(this.workspaceRoot, "target", "debug", "emocis");
    const releaseTargetBinary = path.join(this.workspaceRoot, "target", "release", "emocis");
    let executable = "emocis";
    let args: string[] = ["mcp"];

    if (fs.existsSync(releaseTargetBinary)) {
      executable = releaseTargetBinary;
    } else if (fs.existsSync(localTargetBinary)) {
      executable = localTargetBinary;
    } else if (fs.existsSync(cargoBinary)) {
      executable = cargoBinary;
    } else {
      const bunCandidates = [
        path.join(homeDir, ".bun", "bin", "bun"),
        "/usr/local/bin/bun",
        "/usr/bin/bun",
      ];
      const bunBin = bunCandidates.find((b) => fs.existsSync(b)) || "bun";
      const mcpScript = path.join(this.workspaceRoot, "bin", "mcp.js");
      if (fs.existsSync(mcpScript)) {
        executable = bunBin;
        args = [mcpScript];
      }
    }

    const env = {
      ...process.env,
      PATH: `${homeDir}/.cargo/bin:${homeDir}/.bun/bin:${process.env.PATH || ""}`,
      RUST_LOG: "error",
    };

    try {
      this.process = spawn(executable, args, {
        cwd: this.workspaceRoot,
        env,
      });

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
        for (const [, req] of this.pendingRequests) {
          req.reject(err);
        }
        this.pendingRequests.clear();
      });

      this.process.on("exit", (code) => {
        console.log(`[EMOCIS Client] Subprocess exited with code ${code}`);
        this.process = null;
        for (const [, req] of this.pendingRequests) req.reject(new Error(`Engine exited: ${code}`));
        this.pendingRequests.clear();
      });
    } catch (e) {
      console.error("[EMOCIS Client] Failed to spawn:", e);
      this.process = null;
    }
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
          reject(new Error("EMOCIS request timed out"));
        }
      }, 10000);

      this.pendingRequests.set(id, {
        resolve: (result: McpToolResponse) => {
          clearTimeout(timeout);
          const text = result?.content?.[0]?.text ?? "";
          if (result?.isError) {
            reject(new Error(text || "EMOCIS tool failed"));
            return;
          }
          resolve(text);
        },
        reject: (error: Error) => {
          clearTimeout(timeout);
          reject(error);
        },
      });

      this.process?.stdin.write(payload);
    });
  }

  public async search(query: string) {
    const raw = await this.callTool("emocis_search", { query });
    return JSON.parse(raw);
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
    return JSON.parse(raw);
  }

  public async getCalls(fn: string) {
    const raw = await this.callTool("emocis_calls", { function: fn });
    return JSON.parse(raw);
  }

  public async getCallers(fn: string) {
    const raw = await this.callTool("emocis_callers", { function: fn });
    return JSON.parse(raw);
  }

  public async getHealth() {
    return parseHealth(await this.callTool("emocis_health"));
  }

  public async getAudit() {
    return parseAudit(await this.callTool("emocis_audit"));
  }

  public dispose() {
    if (this.process) {
      this.process.kill();
      this.process = null;
    }
  }
}
