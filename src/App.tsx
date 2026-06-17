import { useEffect, useMemo, useRef, useState } from "react";
import { invoke, Channel } from "@tauri-apps/api/core";

type ToolCategory = "runtime" | "ai-tool";

type ToolStatus = {
  id: string;
  display_name: string;
  category: ToolCategory;
  installed: boolean;
  version: string | null;
};

type InstallEvent =
  | { type: "Log"; line: string }
  | { type: "Progress"; pct: number }
  | { type: "Done"; ok: boolean; version: string | null; error: string | null };

const categoryLabel: Record<ToolCategory, string> = {
  runtime: "运行时与基础工具",
  "ai-tool": "AI 编程工具",
};

function statusText(tool: ToolStatus) {
  if (!tool.installed) return "未安装";
  return tool.version ? tool.version : "已安装";
}

export default function App() {
  const [tools, setTools] = useState<ToolStatus[]>([]);
  const [busy, setBusy] = useState(false);
  const [detecting, setDetecting] = useState(false);
  const [pct, setPct] = useState(0);
  const [logs, setLogs] = useState<string[]>([]);
  const settled = useRef(false);

  const missingCount = tools.filter((tool) => !tool.installed).length;
  const installedCount = tools.length - missingCount;

  const groupedTools = useMemo(() => {
    return tools.reduce<Record<ToolCategory, ToolStatus[]>>(
      (groups, tool) => {
        groups[tool.category].push(tool);
        return groups;
      },
      { runtime: [], "ai-tool": [] },
    );
  }, [tools]);

  async function refreshEnvironment() {
    setDetecting(true);
    try {
      const detected = await invoke<ToolStatus[]>("detect_environment");
      setTools(detected);
    } catch (e) {
      setLogs((current) => [...current, `检测失败: ${String(e)}`]);
    } finally {
      setDetecting(false);
    }
  }

  async function installNode() {
    setBusy(true);
    setPct(0);
    setLogs([]);
    settled.current = false;

    const ch = new Channel<InstallEvent>();
    ch.onmessage = (event) => {
      if (event.type === "Log") {
        setLogs((current) => [...current, event.line]);
        return;
      }

      if (event.type === "Progress") {
        setPct(event.pct);
        return;
      }

      if (settled.current) return;
      settled.current = true;
      setBusy(false);
      setPct(100);

      if (event.ok) {
        setLogs((current) => [...current, `Node.js ${event.version ?? ""} 已就绪`]);
      } else {
        setLogs((current) => [...current, event.error ?? "安装未完成"]);
      }
      void refreshEnvironment();
    };

    try {
      await invoke("install_node", { onEvent: ch });
    } catch (err) {
      if (!settled.current) {
        settled.current = true;
        setBusy(false);
        setLogs((current) => [...current, String(err)]);
      }
    }
  }

  useEffect(() => {
    void refreshEnvironment();
  }, []);

  return (
    <main className="min-h-screen bg-[#f4f5f2] text-[#1d2320]">
      <div className="mx-auto flex min-h-screen w-full max-w-6xl flex-col gap-6 px-6 py-6">
        <header className="flex flex-wrap items-end justify-between gap-4 border-b border-[#cfd6ce] pb-5">
          <div>
            <p className="text-xs font-semibold uppercase tracking-[0.18em] text-[#557067]">Flint</p>
            <h1 className="mt-2 text-3xl font-semibold tracking-normal text-[#151a18]">开发环境仪表盘</h1>
          </div>
          <div className="flex items-center gap-3">
            <div className="border-l border-[#cfd6ce] pl-4 text-sm">
              <span className="font-semibold text-[#1f7a5b]">{installedCount}</span>
              <span className="text-[#59635f]"> 已就绪</span>
              <span className="mx-2 text-[#9aa39f]">/</span>
              <span className="font-semibold text-[#b94737]">{missingCount}</span>
              <span className="text-[#59635f]"> 缺失</span>
            </div>
            <button
              className="h-9 rounded-md border border-[#aeb9b1] bg-white px-4 text-sm font-medium text-[#1d2320] shadow-sm transition hover:border-[#6f8f82] hover:bg-[#fbfcf8] disabled:cursor-not-allowed disabled:opacity-50"
              disabled={detecting || busy}
              onClick={refreshEnvironment}
            >
              {detecting ? "检测中" : "重新检测"}
            </button>
          </div>
        </header>

        <section className="grid gap-4 md:grid-cols-[1fr_320px]">
          <div className="grid gap-4">
            {(Object.keys(groupedTools) as ToolCategory[]).map((category) => (
              <section key={category} className="border-t border-[#cfd6ce] pt-3">
                <div className="mb-3 flex items-center justify-between">
                  <h2 className="text-sm font-semibold text-[#29322f]">{categoryLabel[category]}</h2>
                  <span className="text-xs text-[#65716b]">
                    {groupedTools[category].filter((tool) => tool.installed).length}/{groupedTools[category].length}
                  </span>
                </div>

                <div className="grid gap-2 sm:grid-cols-2">
                  {groupedTools[category].map((tool) => (
                    <article
                      key={tool.id}
                      className="rounded-md border border-[#cfd6ce] bg-white px-4 py-3 shadow-sm"
                    >
                      <div className="flex items-center justify-between gap-3">
                        <div className="min-w-0">
                          <h3 className="truncate text-sm font-semibold text-[#171c1a]">{tool.display_name}</h3>
                          <p className={tool.installed ? "mt-1 text-xs text-[#1f7a5b]" : "mt-1 text-xs text-[#b94737]"}>
                            {statusText(tool)}
                          </p>
                        </div>
                        <span
                          aria-label={tool.installed ? "installed" : "missing"}
                          className={
                            tool.installed
                              ? "h-2.5 w-2.5 shrink-0 rounded-full bg-[#1f9d68]"
                              : "h-2.5 w-2.5 shrink-0 rounded-full bg-[#d8523f]"
                          }
                        />
                      </div>
                      {tool.id === "node" && !tool.installed && (
                        <button
                          className="mt-3 h-8 w-full rounded-md bg-[#244f45] px-3 text-sm font-medium text-white transition hover:bg-[#173d35] disabled:cursor-not-allowed disabled:opacity-50"
                          disabled={busy}
                          onClick={installNode}
                        >
                          {busy ? "安装中" : "安装 Node LTS"}
                        </button>
                      )}
                    </article>
                  ))}
                </div>
              </section>
            ))}
          </div>

          <aside className="rounded-md border border-[#cfd6ce] bg-[#202724] text-[#eff4ed] shadow-sm">
            <div className="border-b border-white/10 px-4 py-3">
              <div className="flex items-center justify-between">
                <h2 className="text-sm font-semibold">安装日志</h2>
                <span className="text-xs text-[#b7c6bf]">{busy ? `${pct}%` : "空闲"}</span>
              </div>
              {busy && (
                <div className="mt-3 h-1.5 overflow-hidden rounded bg-white/10">
                  <div className="h-full bg-[#58c58c] transition-all" style={{ width: `${pct}%` }} />
                </div>
              )}
            </div>
            <pre className="h-[420px] overflow-auto whitespace-pre-wrap px-4 py-3 text-xs leading-5 text-[#d8e2dc]">
              {logs.join("\n") || "暂无日志"}
            </pre>
          </aside>
        </section>
      </div>
    </main>
  );
}
