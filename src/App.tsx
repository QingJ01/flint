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

type ParameterOption = { value: string; label: string };

type ToolParameterMeta = {
  key: string;
  label: string;
  default: string | null;
  options: ParameterOption[];
};

type ToolMeta = {
  id: string;
  display_name: string;
  category: ToolCategory;
  requires_elevation: boolean;
  parameters: ToolParameterMeta[];
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

type ParamMap = Record<string, Record<string, string>>;

export default function App() {
  const [tools, setTools] = useState<ToolStatus[]>([]);
  const [meta, setMeta] = useState<ToolMeta[]>([]);
  const [params, setParams] = useState<ParamMap>({});
  const [busy, setBusy] = useState(false);
  const [busyTool, setBusyTool] = useState<string | null>(null);
  const [pct, setPct] = useState(0);
  const [logs, setLogs] = useState<string[]>([]);
  const settled = useRef(false);

  const missingCount = tools.filter((tool) => !tool.installed).length;
  const installedCount = tools.length - missingCount;

  const grouped = useMemo(() => {
    const buckets: Record<ToolCategory, { meta: ToolMeta; status: ToolStatus | undefined }[]> = {
      runtime: [],
      "ai-tool": [],
    };
    for (const m of meta) {
      buckets[m.category].push({ meta: m, status: tools.find((t) => t.id === m.id) });
    }
    return buckets;
  }, [meta, tools]);

  async function refresh() {
    try {
      const [status, m] = await Promise.all([
        invoke<ToolStatus[]>("detect_environment"),
        invoke<ToolMeta[]>("list_installable_tools"),
      ]);
      setTools(status);
      setMeta(m);
      setParams((cur) => {
        const next: ParamMap = { ...cur };
        for (const tool of m) {
          const slot = (next[tool.id] ??= {});
          for (const p of tool.parameters) {
            if (slot[p.key] === undefined && p.default) {
              slot[p.key] = p.default;
            }
          }
        }
        return next;
      });
    } catch (e) {
      setLogs((cur) => [...cur, `[error] ${String(e)}`]);
    }
  }

  useEffect(() => {
    void refresh();
  }, []);

  async function installTool(id: string) {
    if (busy) return;
    settled.current = false;
    setBusy(true);
    setBusyTool(id);
    setPct(0);
    setLogs([]);

    const ch = new Channel<InstallEvent>();
    ch.onmessage = (event) => {
      if (event.type === "Log") {
        setLogs((cur) => [...cur, event.line]);
        return;
      }
      if (event.type === "Progress") {
        setPct(event.pct);
        return;
      }
      if (settled.current) return;
      settled.current = true;
      setBusy(false);
      setBusyTool(null);
      setPct(100);
      if (event.ok) {
        const ver = event.version ? ` (${event.version})` : "";
        setLogs((cur) => [...cur, `✓ Done${ver}`]);
      } else {
        setLogs((cur) => [...cur, `✗ ${event.error ?? "Failed"}`]);
      }
      void refresh();
    };

    try {
      await invoke("install_tool", {
        id,
        params: params[id] ?? {},
        onEvent: ch,
      });
    } catch (err) {
      if (!settled.current) {
        settled.current = true;
        setBusy(false);
        setBusyTool(null);
        setLogs((cur) => [...cur, String(err)]);
        void refresh();
      }
    }
  }

  function setParam(toolId: string, key: string, value: string) {
    setParams((cur) => ({
      ...cur,
      [toolId]: { ...(cur[toolId] ?? {}), [key]: value },
    }));
  }

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
              disabled={busy}
              onClick={() => void refresh()}
            >
              {busy ? "刷新中" : "重新检测"}
            </button>
          </div>
        </header>

        <section className="grid gap-4 md:grid-cols-[1fr_320px]">
          <div className="grid gap-4">
            {(Object.keys(grouped) as ToolCategory[]).map((category) => (
              <section key={category} className="border-t border-[#cfd6ce] pt-3">
                <div className="mb-3 flex items-center justify-between">
                  <h2 className="text-sm font-semibold text-[#29322f]">{categoryLabel[category]}</h2>
                  <span className="text-xs text-[#65716b]">
                    {grouped[category].filter(({ status }) => status?.installed).length}/
                    {grouped[category].length}
                  </span>
                </div>

                <div className="grid gap-2 sm:grid-cols-2">
                  {grouped[category].map(({ meta: m, status }) => {
                    const installed = status?.installed ?? false;
                    const isBusy = busyTool === m.id;
                    return (
                      <article
                        key={m.id}
                        className="rounded-md border border-[#cfd6ce] bg-white px-4 py-3 shadow-sm"
                      >
                        <div className="flex items-center justify-between gap-3">
                          <div className="min-w-0">
                            <h3 className="truncate text-sm font-semibold text-[#171c1a]">{m.display_name}</h3>
                            <p
                              className={
                                installed
                                  ? "mt-1 text-xs text-[#1f7a5b]"
                                  : "mt-1 text-xs text-[#b94737]"
                              }
                            >
                              {status ? statusText(status) : "检测中…"}
                            </p>
                          </div>
                          <span
                            aria-label={installed ? "installed" : "missing"}
                            className={
                              installed
                                ? "h-2.5 w-2.5 shrink-0 rounded-full bg-[#1f9d68]"
                                : "h-2.5 w-2.5 shrink-0 rounded-full bg-[#d8523f]"
                            }
                          />
                        </div>
                        {!installed && m.parameters.length > 0 && (
                          <div className="mt-3 space-y-2">
                            {m.parameters.map((p) => (
                              <label key={p.key} className="block">
                                <span className="block text-[11px] uppercase tracking-wider text-[#65716b]">
                                  {p.label}
                                </span>
                                <select
                                  className="mt-1 h-8 w-full rounded border border-[#cfd6ce] bg-white px-2 text-xs text-[#1d2320] focus:border-[#6f8f82] focus:outline-none disabled:cursor-not-allowed disabled:opacity-60"
                                  value={
                                    params[m.id]?.[p.key] ?? p.default ?? p.options[0]?.value ?? ""
                                  }
                                  onChange={(e) => setParam(m.id, p.key, e.target.value)}
                                  disabled={busy}
                                >
                                  {p.options.map((opt) => (
                                    <option key={opt.value} value={opt.value}>
                                      {opt.label}
                                    </option>
                                  ))}
                                </select>
                              </label>
                            ))}
                          </div>
                        )}
                        {!installed && (
                          <button
                            className="mt-3 h-8 w-full rounded-md bg-[#244f45] px-3 text-sm font-medium text-white transition hover:bg-[#173d35] disabled:cursor-not-allowed disabled:opacity-50"
                            disabled={busy}
                            onClick={() => void installTool(m.id)}
                          >
                            {isBusy ? "安装中…" : "安装"}
                          </button>
                        )}
                      </article>
                    );
                  })}
                </div>
              </section>
            ))}
          </div>

          <aside className="rounded-md border border-[#cfd6ce] bg-[#202724] text-[#eff4ed] shadow-sm">
            <div className="border-b border-white/10 px-4 py-3">
              <div className="flex items-center justify-between">
                <h2 className="text-sm font-semibold">安装日志</h2>
                <span className="text-xs text-[#b7c6bf]">
                  {busy ? (busyTool ? `安装 ${busyTool} · ${pct}%` : `${pct}%`) : "空闲"}
                </span>
              </div>
              {busy && (
                <div className="mt-3 h-1.5 overflow-hidden rounded bg-white/10">
                  <div
                    className="h-full bg-[#58c58c] transition-all"
                    style={{ width: `${pct}%` }}
                  />
                </div>
              )}
            </div>
            <pre className="h-[480px] overflow-auto whitespace-pre-wrap px-4 py-3 text-xs leading-5 text-[#d8e2dc]">
              {logs.join("\n") || "暂无日志"}
            </pre>
          </aside>
        </section>
      </div>
    </main>
  );
}
