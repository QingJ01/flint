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

const categoryOrder: ToolCategory[] = ["runtime", "ai-tool"];

function statusText(tool: ToolStatus) {
  if (!tool.installed) return "未安装";
  return tool.version ? `v${tool.version}` : "已安装";
}

type ParamMap = Record<string, Record<string, string>>;

/** Maps a log line to a CSS color class based on its leading tag. */
function logClass(line: string): string {
  if (line.startsWith("[err]") || line.startsWith("✗") || line.startsWith("[error]"))
    return "text-log-err";
  if (line.startsWith("[ok]") || line.startsWith("✓") || line.startsWith("[skip]"))
    return "text-log-ok";
  if (line.startsWith("[warn]") || line.startsWith("[!]")) return "text-log-warn";
  if (line.startsWith("[out]")) return "text-log-faint";
  return "text-log-text";
}

export default function App() {
  const [tools, setTools] = useState<ToolStatus[]>([]);
  const [meta, setMeta] = useState<ToolMeta[]>([]);
  const [params, setParams] = useState<ParamMap>({});
  const [busy, setBusy] = useState(false);
  const [busyTool, setBusyTool] = useState<string | null>(null);
  const [pct, setPct] = useState(0);
  const [logs, setLogs] = useState<string[]>([]);
  const [refreshing, setRefreshing] = useState(false);
  const settled = useRef(false);
  const logEndRef = useRef<HTMLDivElement | null>(null);

  const statusById = useMemo(() => {
    const map = new Map<string, ToolStatus>();
    for (const t of tools) map.set(t.id, t);
    return map;
  }, [tools]);

  const missingCount = tools.filter((t) => !t.installed).length;
  const installedCount = tools.length - missingCount;
  const totalForSection = (cat: ToolCategory) =>
    meta.filter((m) => m.category === cat).length;
  const installedForSection = (cat: ToolCategory) =>
    meta.filter(
      (m) => m.category === cat && statusById.get(m.id)?.installed,
    ).length;

  async function refresh() {
    if (busy) return;
    setRefreshing(true);
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
    } finally {
      setRefreshing(false);
    }
  }

  useEffect(() => {
    void refresh();
  }, []);

  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: "smooth", block: "end" });
  }, [logs]);

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
        const ver = event.version ? ` · v${event.version}` : "";
        setLogs((cur) => [...cur, `✓ 安装成功${ver}`]);
      } else {
        setLogs((cur) => [...cur, `✗ ${event.error ?? "安装失败"}`]);
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
        setLogs((cur) => [...cur, `[error] ${String(err)}`]);
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
    <main className="min-h-screen bg-cream text-ink">
      <div className="mx-auto flex min-h-screen w-full max-w-6xl flex-col px-8 py-8">
        {/* Top bar — wordmark + counter + re-detect */}
        <header className="flex items-end justify-between gap-6 pb-7">
          <div>
            <h1 className="font-sans text-[26px] font-semibold leading-none tracking-[-0.01em] text-ink">
              Flint
              <span className="text-accent">.</span>
            </h1>
            <p className="mt-2 text-[13px] text-ink-muted">
              一击点燃你的开发环境
            </p>
          </div>
          <div className="flex items-center gap-5">
            <div className="text-[13px] tabular-nums text-ink-muted">
              <span className="font-medium text-success">{installedCount}</span>
              <span className="mx-1.5 text-ink-faint">/</span>
              <span className="font-medium text-ink">{tools.length}</span>
              <span className="ml-1.5 text-ink-faint">已就绪</span>
            </div>
            <button
              type="button"
              onClick={() => void refresh()}
              disabled={busy || refreshing}
              className="inline-flex h-8 items-center gap-1.5 rounded-lg border border-line bg-surface px-3.5 text-[13px] font-medium text-ink shadow-[0_1px_0_rgba(0,0,0,0.02)] transition hover:border-line-strong hover:bg-cream-deep disabled:cursor-not-allowed disabled:opacity-50"
            >
              <RefreshIcon className="h-3.5 w-3.5" />
              {refreshing ? "检测中" : "重新检测"}
            </button>
          </div>
        </header>

        {/* Slim top progress bar — only visible while a tool is installing */}
        <div
          aria-hidden
          className="-mx-8 mb-2 h-[2px] overflow-hidden bg-transparent transition-opacity"
          style={{ opacity: busy ? 1 : 0 }}
        >
          <div
            className="h-full bg-accent transition-[width] duration-300 ease-out"
            style={{ width: `${pct}%` }}
          />
        </div>

        {/* Body: cards on the left, log sidebar on the right */}
        <section className="grid flex-1 gap-6 lg:grid-cols-[1fr_360px]">
          <div className="flex flex-col gap-8">
            {categoryOrder.map((category) => {
              const sectionMetas = meta.filter((m) => m.category === category);
              if (sectionMetas.length === 0) return null;
              return (
                <section key={category}>
                  <div className="mb-3 flex items-baseline justify-between border-b border-line pb-2">
                    <h2 className="text-[11px] font-semibold uppercase tracking-[0.12em] text-ink-faint">
                      {categoryLabel[category]}
                    </h2>
                    <span className="text-[11px] tabular-nums text-ink-faint">
                      {installedForSection(category)} / {totalForSection(category)} 已就绪
                    </span>
                  </div>

                  <div className="grid gap-3 sm:grid-cols-2">
                    {sectionMetas.map((m) => {
                      const status = statusById.get(m.id);
                      const installed = status?.installed ?? false;
                      const isThisBusy = busyTool === m.id;
                      const isOtherBusy = busy && !isThisBusy;
                      return (
                        <article
                          key={m.id}
                          className="card-enter group relative flex flex-col gap-3 rounded-xl border border-line bg-surface p-4 shadow-[0_1px_2px_rgba(31,30,27,0.03)] transition hover:border-line-strong hover:shadow-[0_2px_6px_rgba(31,30,27,0.05)]"
                        >
                          <div className="flex items-start justify-between gap-3">
                            <div className="min-w-0 flex-1">
                              <h3 className="truncate text-[15px] font-medium text-ink">
                                {m.display_name}
                              </h3>
                              <p
                                className={
                                  "mt-0.5 font-mono text-[12px] " +
                                  (installed
                                    ? "text-success"
                                    : status
                                      ? "text-danger"
                                      : "text-ink-faint")
                                }
                              >
                                {status ? statusText(status) : "检测中…"}
                              </p>
                            </div>
                            <span
                              aria-label={installed ? "installed" : "missing"}
                              className={
                                "mt-1 h-2 w-2 shrink-0 rounded-full " +
                                (isThisBusy
                                  ? "bg-accent dot-pulse text-accent"
                                  : installed
                                    ? "bg-success"
                                    : status
                                      ? "bg-danger"
                                      : "bg-ink-faint/40")
                              }
                            />
                          </div>

                          {!installed && m.parameters.length > 0 && (
                            <div className="flex flex-col gap-2">
                              {m.parameters.map((p) => (
                                <label key={p.key} className="block">
                                  <span className="block text-[10.5px] font-medium uppercase tracking-[0.08em] text-ink-faint">
                                    {p.label}
                                  </span>
                                  <div className="relative mt-1">
                                    <select
                                      className="h-8 w-full appearance-none rounded-md border border-line bg-surface pl-2.5 pr-7 text-[12.5px] text-ink transition hover:border-line-strong focus:border-accent focus:outline-none disabled:cursor-not-allowed disabled:opacity-60"
                                      value={
                                        params[m.id]?.[p.key] ??
                                        p.default ??
                                        p.options[0]?.value ??
                                        ""
                                      }
                                      onChange={(e) =>
                                        setParam(m.id, p.key, e.target.value)
                                      }
                                      disabled={busy}
                                    >
                                      {p.options.map((opt) => (
                                        <option key={opt.value} value={opt.value}>
                                          {opt.label}
                                        </option>
                                      ))}
                                    </select>
                                    <ChevronIcon className="pointer-events-none absolute right-2 top-1/2 h-3 w-3 -translate-y-1/2 text-ink-faint" />
                                  </div>
                                </label>
                              ))}
                            </div>
                          )}

                          {!installed && (
                            <button
                              type="button"
                              onClick={() => void installTool(m.id)}
                              disabled={busy}
                              className={
                                "mt-auto inline-flex h-8 w-full items-center justify-center gap-1.5 rounded-lg px-3 text-[13px] font-medium transition " +
                                (isThisBusy
                                  ? "bg-accent text-white shadow-[0_1px_2px_rgba(204,120,92,0.4)]"
                                  : isOtherBusy
                                    ? "bg-ink/30 text-white/70 cursor-not-allowed"
                                    : "bg-ink text-white hover:bg-ink/90 active:bg-ink/95 shadow-[0_1px_2px_rgba(31,30,27,0.18)]")
                              }
                            >
                              {isThisBusy ? (
                                <>
                                  <SpinnerIcon className="h-3 w-3 animate-spin" />
                                  安装中…
                                </>
                              ) : (
                                "安装"
                              )}
                            </button>
                          )}
                        </article>
                      );
                    })}
                  </div>
                </section>
              );
            })}
          </div>

          {/* Log sidebar */}
          <aside className="self-start rounded-xl border border-line bg-log-bg text-log-text shadow-[0_2px_8px_rgba(31,30,27,0.05)] lg:sticky lg:top-6">
            <div className="flex items-center justify-between border-b border-white/5 px-4 py-3">
              <div className="flex items-center gap-2">
                <span className="h-2 w-2 rounded-full bg-log-faint" />
                <h2 className="text-[12px] font-semibold tracking-wide text-log-text">
                  安装日志
                </h2>
              </div>
              <span className="font-mono text-[11px] tabular-nums text-log-faint">
                {busy
                  ? busyTool
                    ? `${busyTool} · ${pct}%`
                    : `${pct}%`
                  : logs.length > 0
                    ? `${logs.length} 行`
                    : "空闲"}
              </span>
            </div>
            <pre className="log-scroll h-[560px] overflow-auto whitespace-pre-wrap break-words px-4 py-3 font-mono text-[11.5px] leading-[1.65]">
              {logs.length === 0 ? (
                <span className="text-log-faint">
                  暂无日志。点击任意工具卡的「安装」开始。
                </span>
              ) : (
                logs.map((line, i) => (
                  <div key={i} className={logClass(line)}>
                    {line}
                  </div>
                ))
              )}
              <div ref={logEndRef} />
            </pre>
          </aside>
        </section>

        <footer className="mt-10 flex items-center justify-between border-t border-line pt-5 text-[11px] text-ink-faint">
          <span>Flint · 一键点燃开发环境</span>
          <span className="font-mono">v0.1.0 · slice 1</span>
        </footer>
      </div>
    </main>
  );
}

/* ---------- inline icons (no extra deps) ---------- */

function RefreshIcon(props: { className?: string }) {
  return (
    <svg
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      {...props}
    >
      <path d="M13.5 8a5.5 5.5 0 1 1-1.61-3.89" />
      <path d="M13.5 2.5v3.5h-3.5" />
    </svg>
  );
}

function ChevronIcon(props: { className?: string }) {
  return (
    <svg
      viewBox="0 0 12 12"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      {...props}
    >
      <path d="M3 4.5 6 7.5 9 4.5" />
    </svg>
  );
}

function SpinnerIcon(props: { className?: string }) {
  return (
    <svg
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.75"
      strokeLinecap="round"
      {...props}
    >
      <path d="M8 2a6 6 0 0 1 6 6" />
    </svg>
  );
}
