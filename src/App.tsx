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

type PresetMeta = {
  id: string;
  display_name: string;
  description: string;
  emoji: string;
};

type PresetFull = {
  meta: PresetMeta;
  tools: { ids: string[]; params: Record<string, Record<string, string>> };
};

type WslStatus = {
  state: "not-installed" | "enabled" | "ready" | "unknown";
  default_distro: string | null;
  distros: string[];
  kernel_version: string | null;
  raw: string;
};

type MirrorStatus = {
  npm: string | null;
  pip: string | null;
};

const NPM_MIRRORS: { value: string; label: string }[] = [
  { value: "https://registry.npmjs.org/", label: "官方源 (npmjs.org)" },
  { value: "https://registry.npmmirror.com/", label: "淘宝镜像 (npmmirror.com)" },
  { value: "https://mirrors.huaweicloud.com/repository/npm/", label: "华为云镜像" },
  { value: "https://mirrors.cloud.tencent.com/npm/", label: "腾讯云镜像" },
];

const PIP_MIRRORS: { value: string; label: string }[] = [
  { value: "https://pypi.org/simple", label: "官方源 (PyPI)" },
  { value: "https://pypi.tuna.tsinghua.edu.cn/simple", label: "清华源" },
  { value: "https://mirrors.aliyun.com/pypi/simple", label: "阿里源" },
  { value: "https://mirrors.huaweicloud.com/repository/pypi/simple", label: "华为云镜像" },
  { value: "https://mirrors.cloud.tencent.com/pypi/simple", label: "腾讯云镜像" },
];

type InstallEvent =
  | { type: "Log"; line: string }
  | { type: "Progress"; pct: number }
  | { type: "Done"; ok: boolean; version: string | null; error: string | null };

type View = "dashboard" | "presets" | "wsl" | "mirrors";

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
  const [view, setView] = useState<View>("dashboard");
  const [tools, setTools] = useState<ToolStatus[]>([]);
  const [meta, setMeta] = useState<ToolMeta[]>([]);
  const [presets, setPresets] = useState<PresetMeta[]>([]);
  const [params, setParams] = useState<ParamMap>({});
  const [busy, setBusy] = useState(false);
  const [busyTool, setBusyTool] = useState<string | null>(null);
  const [pct, setPct] = useState(0);
  const [logs, setLogs] = useState<string[]>([]);
  const [refreshing, setRefreshing] = useState(false);
  const [presetProgress, setPresetProgress] = useState<{
    presetName: string;
    index: number;
    total: number;
    currentTool: string;
    succeeded: string[];
    failed: string[];
    skipped: string[];
  } | null>(null);
  const [wsl, setWsl] = useState<WslStatus | null>(null);
  const [wslBusy, setWslBusy] = useState(false);
  const [mirror, setMirror] = useState<MirrorStatus | null>(null);
  const [mirrorBusy, setMirrorBusy] = useState(false);
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
      const [status, m, p, ws, mi] = await Promise.all([
        invoke<ToolStatus[]>("detect_environment"),
        invoke<ToolMeta[]>("list_installable_tools"),
        invoke<PresetMeta[]>("list_presets"),
        invoke<WslStatus>("wsl_status"),
        invoke<MirrorStatus>("mirror_status").catch(() => ({ npm: null, pip: null })),
      ]);
      setTools(status);
      setMeta(m);
      setPresets(p);
      setWsl(ws);
      setMirror(mi);
      setParams((cur) => {
        const next: ParamMap = { ...cur };
        for (const tool of m) {
          const slot = (next[tool.id] ??= {});
          for (const param of tool.parameters) {
            if (slot[param.key] === undefined && param.default) {
              slot[param.key] = param.default;
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

  /** Run a single tool's install via the backend command and resolve when
   *  the `Done` event arrives. Returns `true` on success. */
  function runOneInstall(
    id: string,
    toolParams: Record<string, string>,
  ): Promise<{ ok: boolean; version: string | null; error: string | null }> {
    return new Promise((resolve) => {
      settled.current = false;
      setBusyTool(id);
      setPct(0);
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
        setPct(100);
        resolve({ ok: event.ok, version: event.version, error: event.error });
      };
      invoke("install_tool", { id, params: toolParams, onEvent: ch }).catch(
        (err) => {
          if (!settled.current) {
            settled.current = true;
            resolve({ ok: false, version: null, error: String(err) });
          }
        },
      );
    });
  }

  /** Run a backend command that streams `InstallEvent`s (any of the `*` IPCs
   *  that take a `Channel`). Returns when `Done` arrives. */
  function runStreamingCommand(
    cmd: "install_tool" | "wsl_enable" | "wsl_install_dev_tools",
    args: Record<string, unknown>,
  ): Promise<{ ok: boolean; version: string | null; error: string | null }> {
    return new Promise((resolve) => {
      settled.current = false;
      setPct(0);
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
        setPct(100);
        resolve({ ok: event.ok, version: event.version, error: event.error });
      };
      invoke(cmd, { ...args, onEvent: ch }).catch((err) => {
        if (!settled.current) {
          settled.current = true;
          resolve({ ok: false, version: null, error: String(err) });
        }
      });
    });
  }

  async function wslEnable() {
    if (wslBusy) return;
    setWslBusy(true);
    setLogs([]);
    setBusyTool("wsl-enable");
    const res = await runStreamingCommand("wsl_enable", {});
    setBusyTool(null);
    setWslBusy(false);
    if (!res.ok) {
      setLogs((cur) => [...cur, `✗ ${res.error ?? "WSL 启用失败"}`]);
    }
    void refresh();
  }

  async function wslInstallDevTools() {
    if (wslBusy) return;
    setWslBusy(true);
    setLogs([]);
    setBusyTool("wsl-dev-tools");
    const res = await runStreamingCommand("wsl_install_dev_tools", {});
    setBusyTool(null);
    setWslBusy(false);
    if (!res.ok) {
      setLogs((cur) => [...cur, `✗ ${res.error ?? "WSL 内开发环境安装失败"}`]);
    }
    void refresh();
  }

  async function applyNpmMirror(url: string) {
    if (mirrorBusy) return;
    setMirrorBusy(true);
    try {
      const changed = await invoke<boolean>("apply_npm_mirror", {
        registryUrl: url,
      });
      setLogs((cur) => [
        ...cur,
        changed ? `✓ npm registry 已写入：${url}` : `[skip] npm registry 已是该值`,
      ]);
      void refresh();
    } catch (e) {
      setLogs((cur) => [...cur, `✗ npm mirror 失败：${String(e)}`]);
    } finally {
      setMirrorBusy(false);
    }
  }

  async function applyPipMirror(url: string) {
    if (mirrorBusy) return;
    setMirrorBusy(true);
    try {
      const changed = await invoke<boolean>("apply_pip_mirror", {
        indexUrl: url,
      });
      setLogs((cur) => [
        ...cur,
        changed ? `✓ pip index-url 已写入：${url}` : `[skip] pip index-url 已是该值`,
      ]);
      void refresh();
    } catch (e) {
      setLogs((cur) => [...cur, `✗ pip mirror 失败：${String(e)}`]);
    } finally {
      setMirrorBusy(false);
    }
  }

  async function applyDomestic() {
    if (mirrorBusy) return;
    setMirrorBusy(true);
    try {
      const res = await invoke<[string, boolean][]>(
        "apply_domestic_acceleration",
      );
      for (const [kind, changed] of res) {
        setLogs((cur) => [
          ...cur,
          changed
            ? `✓ ${kind} 已切到国内源`
            : `[skip] ${kind} 已是国内源`,
        ]);
      }
      void refresh();
    } catch (e) {
      setLogs((cur) => [...cur, `✗ 国内加速失败：${String(e)}`]);
    } finally {
      setMirrorBusy(false);
    }
  }

  async function installOne(id: string) {
    if (busy) return;
    setBusy(true);
    setLogs([]);
    const res = await runOneInstall(id, params[id] ?? {});
    setBusy(false);
    setBusyTool(null);
    if (res.ok) {
      const ver = res.version ? ` · v${res.version}` : "";
      setLogs((cur) => [...cur, `✓ 安装成功${ver}`]);
    } else {
      setLogs((cur) => [...cur, `✗ ${res.error ?? "安装失败"}`]);
    }
    void refresh();
  }

  async function applyPreset(presetId: string) {
    if (busy) return;
    let full: PresetFull;
    try {
      full = await invoke<PresetFull>("get_preset", { id: presetId });
    } catch (e) {
      setLogs((cur) => [
        ...cur,
        `[error] 无法加载预设 ${presetId}：${String(e)}`,
      ]);
      return;
    }

    setBusy(true);
    setLogs([]);
    const succeeded: string[] = [];
    const failed: string[] = [];
    const skipped: string[] = [];

    setPresetProgress({
      presetName: full.meta.display_name,
      index: 0,
      total: full.tools.ids.length,
      currentTool: full.tools.ids[0] ?? "",
      succeeded,
      failed,
      skipped,
    });

    const freshStatus = new Map(statusById);
    for (let i = 0; i < full.tools.ids.length; i++) {
      const id = full.tools.ids[i];
      setPresetProgress((cur) =>
        cur ? { ...cur, index: i, currentTool: id } : cur,
      );

      if (freshStatus.get(id)?.installed) {
        skipped.push(id);
        setLogs((cur) => [
          ...cur,
          `[skip] ${id} 已安装，跳过`,
        ]);
        continue;
      }

      const toolParams = {
        ...(params[id] ?? {}),
        ...(full.tools.params[id] ?? {}),
      };
      setLogs((cur) => [...cur, `[preset] ▶ ${id}`]);
      const res = await runOneInstall(id, toolParams);
      if (res.ok) {
        succeeded.push(id);
        setLogs((cur) => [
          ...cur,
          `✓ ${id}${res.version ? ` · v${res.version}` : ""}`,
        ]);
        // optimistic local status update
        freshStatus.set(id, {
          id,
          display_name: id,
          category: "runtime",
          installed: true,
          version: res.version,
        });
      } else {
        failed.push(id);
        setLogs((cur) => [
          ...cur,
          `✗ ${id}：${res.error ?? "失败"}`,
        ]);
      }
    }

    setBusy(false);
    setBusyTool(null);
    setLogs((cur) => [
      ...cur,
      ``,
      `[preset] 总结：✓ ${succeeded.length} · ✗ ${failed.length} · 跳过 ${skipped.length}`,
    ]);
    setPresetProgress(null);
    void refresh();
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
        <header className="flex items-end justify-between gap-6 pb-6">
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

        {/* Tab bar */}
        <nav className="mb-5 flex items-center gap-1 border-b border-line">
          <TabButton
            active={view === "dashboard"}
            onClick={() => setView("dashboard")}
            label="仪表盘"
          />
          <TabButton
            active={view === "presets"}
            onClick={() => setView("presets")}
            label="预设"
            badge={presets.length}
          />
          <TabButton
            active={view === "wsl"}
            onClick={() => setView("wsl")}
            label="WSL"
          />
          <TabButton
            active={view === "mirrors"}
            onClick={() => setView("mirrors")}
            label="镜像"
          />
        </nav>

        {/* Slim top progress bar */}
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

        <section className="grid flex-1 gap-6 lg:grid-cols-[1fr_360px]">
          <div className="flex flex-col gap-8">
            {view === "dashboard" ? (
              categoryOrder.map((category) => {
                const sectionMetas = meta.filter(
                  (m) => m.category === category,
                );
                if (sectionMetas.length === 0) return null;
                return (
                  <section key={category}>
                    <div className="mb-3 flex items-baseline justify-between border-b border-line pb-2">
                      <h2 className="text-[11px] font-semibold uppercase tracking-[0.12em] text-ink-faint">
                        {categoryLabel[category]}
                      </h2>
                      <span className="text-[11px] tabular-nums text-ink-faint">
                        {installedForSection(category)} /{" "}
                        {totalForSection(category)} 已就绪
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
                                          setParam(
                                            m.id,
                                            p.key,
                                            e.target.value,
                                          )
                                        }
                                        disabled={busy}
                                      >
                                        {p.options.map((opt) => (
                                          <option
                                            key={opt.value}
                                            value={opt.value}
                                          >
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
                                onClick={() => void installOne(m.id)}
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
              })
            ) : view === "presets" ? (
              <PresetsView
                presets={presets}
                statusById={statusById}
                onApply={(id) => void applyPreset(id)}
                busy={busy}
                presetProgress={presetProgress}
              />
            ) : view === "wsl" ? (
              <WslView
                status={wsl}
                busy={wslBusy}
                onEnable={() => void wslEnable()}
                onInstallDevTools={() => void wslInstallDevTools()}
              />
            ) : (
              <MirrorsView
                status={mirror}
                busy={mirrorBusy}
                onApplyNpm={(url) => void applyNpmMirror(url)}
                onApplyPip={(url) => void applyPipMirror(url)}
                onAccelerate={() => void applyDomestic()}
              />
            )}
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
                {presetProgress
                  ? `${presetProgress.index + 1}/${presetProgress.total} · ${presetProgress.currentTool}`
                  : busy
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
          <span className="font-mono">v0.4.0 · slice 4</span>
        </footer>
      </div>
    </main>
  );
}

/* ---------- Presets view ---------- */

function PresetsView(props: {
  presets: PresetMeta[];
  statusById: Map<string, ToolStatus>;
  onApply: (id: string) => void;
  busy: boolean;
  presetProgress: {
    presetName: string;
    index: number;
    total: number;
    currentTool: string;
    succeeded: string[];
    failed: string[];
    skipped: string[];
  } | null;
}) {
  const { presets, onApply, busy, presetProgress } = props;
  if (presets.length === 0) {
    return (
      <div className="rounded-xl border border-dashed border-line bg-surface-sunken p-8 text-center">
        <p className="text-[13px] text-ink-muted">暂无可用预设。</p>
      </div>
    );
  }
  return (
    <div className="grid gap-3 sm:grid-cols-2">
      {presets.map((p) => {
        const isThis = presetProgress?.presetName === p.display_name;
        return (
          <article
            key={p.id}
            className="card-enter flex flex-col gap-3 rounded-xl border border-line bg-surface p-5 shadow-[0_1px_2px_rgba(31,30,27,0.03)] transition hover:border-line-strong"
          >
            <div className="flex items-start gap-3">
              <span className="text-3xl leading-none">{p.emoji || "📦"}</span>
              <div className="min-w-0 flex-1">
                <h3 className="truncate text-[15px] font-medium text-ink">
                  {p.display_name}
                </h3>
                <p className="mt-1 text-[12.5px] leading-relaxed text-ink-muted">
                  {p.description}
                </p>
              </div>
            </div>
            <div className="mt-auto flex items-center justify-between pt-2">
              <span className="text-[11px] text-ink-faint">
                {isThis
                  ? `${presetProgress!.index + 1}/${presetProgress!.total} · ${presetProgress!.currentTool}`
                  : "一键组合安装"}
              </span>
              <button
                type="button"
                onClick={() => onApply(p.id)}
                disabled={busy}
                className={
                  "inline-flex h-8 items-center gap-1.5 rounded-lg px-3.5 text-[13px] font-medium transition " +
                  (isThis
                    ? "bg-accent text-white shadow-[0_1px_2px_rgba(204,120,92,0.4)]"
                    : busy
                      ? "bg-ink/30 text-white/70 cursor-not-allowed"
                      : "bg-ink text-white hover:bg-ink/90 shadow-[0_1px_2px_rgba(31,30,27,0.18)]")
                }
              >
                {isThis ? (
                  <>
                    <SpinnerIcon className="h-3 w-3 animate-spin" />
                    安装中…
                  </>
                ) : (
                  "应用预设"
                )}
              </button>
            </div>
          </article>
        );
      })}
    </div>
  );
}

/* ---------- WSL wizard view ---------- */

function WslView(props: {
  status: WslStatus | null;
  busy: boolean;
  onEnable: () => void;
  onInstallDevTools: () => void;
}) {
  const { status, busy, onEnable, onInstallDevTools } = props;

  const stateLabel: Record<WslStatus["state"], string> = {
    "not-installed": "未启用",
    enabled: "已启用（无发行版）",
    ready: "就绪",
    unknown: "未知",
  };
  const stateDot: Record<WslStatus["state"], string> = {
    "not-installed": "bg-danger",
    enabled: "bg-warn",
    ready: "bg-success",
    unknown: "bg-ink-faint/40",
  };

  const step1Done = status?.state === "enabled" || status?.state === "ready";
  const step2Done = status?.state === "ready";

  return (
    <div className="flex flex-col gap-4">
      <article className="rounded-xl border border-line bg-surface p-6 shadow-[0_1px_2px_rgba(31,30,27,0.03)]">
        <div className="flex items-start justify-between gap-6">
          <div className="min-w-0 flex-1">
            <div className="flex items-center gap-2">
              <h2 className="text-[16px] font-medium text-ink">
                Windows Subsystem for Linux
              </h2>
              {status && (
                <span
                  className={`h-2 w-2 shrink-0 rounded-full ${stateDot[status.state]}`}
                />
              )}
            </div>
            <p className="mt-1 text-[12.5px] text-ink-muted">
              {status
                ? `当前状态：${stateLabel[status.state]}${status.default_distro ? ` · 默认发行版：${status.default_distro}` : ""}`
                : "检测中…"}
            </p>
            {status?.distros && status.distros.length > 0 && (
              <p className="mt-1 font-mono text-[11.5px] text-ink-faint">
                已装发行版：{status.distros.join(", ")}
              </p>
            )}
          </div>
        </div>
      </article>

      <article className="rounded-xl border border-line bg-surface p-5 shadow-[0_1px_2px_rgba(31,30,27,0.03)]">
        <div className="flex items-start gap-4">
          <StepBadge n={1} done={step1Done} />
          <div className="min-w-0 flex-1">
            <h3 className="text-[14px] font-medium text-ink">
              启用 WSL 并安装 Ubuntu
            </h3>
            <p className="mt-1 text-[12.5px] leading-relaxed text-ink-muted">
              首次启用需要一次性管理员权限（Windows 会弹 UAC 对话框）。
              操作完成后新开 PowerShell 运行{" "}
              <code className="rounded bg-surface-sunken px-1 py-0.5 font-mono text-[11.5px] text-ink">
                wsl --status
              </code>{" "}
              验证。
            </p>
            <button
              type="button"
              onClick={onEnable}
              disabled={busy || step1Done}
              className={
                "mt-3 inline-flex h-8 items-center gap-1.5 rounded-lg px-3.5 text-[13px] font-medium transition " +
                (step1Done
                  ? "border border-line bg-surface-sunken text-ink-muted"
                  : busy
                    ? "bg-ink/30 text-white/70 cursor-not-allowed"
                    : "bg-ink text-white hover:bg-ink/90 shadow-[0_1px_2px_rgba(31,30,27,0.18)]")
              }
            >
              {step1Done ? "✓ 已完成" : "启用 WSL"}
            </button>
          </div>
        </div>
      </article>

      <article className="rounded-xl border border-line bg-surface p-5 shadow-[0_1px_2px_rgba(31,30,27,0.03)]">
        <div className="flex items-start gap-4">
          <StepBadge n={2} done={step2Done} />
          <div className="min-w-0 flex-1">
            <h3 className="text-[14px] font-medium text-ink">
              在 Ubuntu 里装开发环境
            </h3>
            <p className="mt-1 text-[12.5px] leading-relaxed text-ink-muted">
              在 WSL 的 Ubuntu 发行版中以 root 身份安装 Git、Node LTS、Bun、Python、uv、Claude Code。
              约需 3-5 分钟。
            </p>
            <button
              type="button"
              onClick={onInstallDevTools}
              disabled={busy || !step1Done || step2Done}
              className={
                "mt-3 inline-flex h-8 items-center gap-1.5 rounded-lg px-3.5 text-[13px] font-medium transition " +
                (step2Done
                  ? "border border-line bg-surface-sunken text-ink-muted"
                  : busy
                    ? "bg-ink/30 text-white/70 cursor-not-allowed"
                    : step1Done
                      ? "bg-ink text-white hover:bg-ink/90 shadow-[0_1px_2px_rgba(31,30,27,0.18)]"
                      : "bg-ink/20 text-white/60 cursor-not-allowed")
              }
            >
              {step2Done ? "✓ 已就绪" : "安装 WSL 开发环境"}
            </button>
          </div>
        </div>
      </article>

      <p className="px-1 text-[11.5px] text-ink-faint">
        安装完成后，在 PowerShell 运行{" "}
        <code className="rounded bg-surface px-1 py-0.5 font-mono text-[11px] text-ink-muted">
          wsl
        </code>{" "}
        进入 Ubuntu；或{" "}
        <code className="rounded bg-surface px-1 py-0.5 font-mono text-[11px] text-ink-muted">
          wsl -d Ubuntu code .
        </code>{" "}
        在 WSL 中直接打开 VS Code（需 Windows 端已装 VS Code）。
      </p>
    </div>
  );
}

/* ---------- Mirrors view ---------- */

function MirrorsView(props: {
  status: MirrorStatus | null;
  busy: boolean;
  onApplyNpm: (url: string) => void;
  onApplyPip: (url: string) => void;
  onAccelerate: () => void;
}) {
  const { status, busy, onApplyNpm, onApplyPip, onAccelerate } = props;
  const npmCurrent = status?.npm ?? "";
  const pipCurrent = status?.pip ?? "";
  const npmInCN = npmCurrent.includes("npmmirror") || npmCurrent.includes("huaweicloud") || npmCurrent.includes("tencent");
  const pipInCN = pipCurrent.includes("tuna") || pipCurrent.includes("aliyun") || pipCurrent.includes("huaweicloud") || pipCurrent.includes("tencent");

  return (
    <div className="flex flex-col gap-4">
      <article className="flex items-center justify-between gap-4 rounded-xl border border-accent-soft bg-accent-soft/30 p-5">
        <div className="min-w-0 flex-1">
          <h2 className="text-[15px] font-medium text-ink">国内加速模式</h2>
          <p className="mt-1 text-[12.5px] text-ink-muted">
            一键切换 npm + pip 到国内镜像。GitHub 加速请见下方的 "gh-proxy" 链接。
          </p>
        </div>
        <button
          type="button"
          onClick={onAccelerate}
          disabled={busy}
          className="inline-flex h-9 items-center gap-2 rounded-lg bg-accent px-4 text-[13px] font-medium text-white shadow-[0_1px_2px_rgba(204,120,92,0.4)] transition hover:bg-accent-deep disabled:opacity-50"
        >
          <RocketIcon className="h-3.5 w-3.5" />
          {busy ? "切换中…" : "一键加速"}
        </button>
      </article>

      <MirrorCard
        title="npm registry"
        description="通过 ~/.npmrc 锁定。影响 npm install / npx 等所有 Node 包下载。"
        current={npmCurrent}
        inCN={npmInCN}
        options={NPM_MIRRORS}
        busy={busy}
        onApply={onApplyNpm}
      />

      <MirrorCard
        title="pip index-url"
        description="通过 pip.ini (Windows) / pip.conf (POSIX) 锁定。影响 pip install / uv 等所有 Python 包下载。"
        current={pipCurrent}
        inCN={pipInCN}
        options={PIP_MIRRORS}
        busy={busy}
        onApply={onApplyPip}
      />

      <article className="rounded-xl border border-line bg-surface p-5">
        <h3 className="text-[14px] font-medium text-ink">GitHub 克隆加速</h3>
        <p className="mt-1 text-[12.5px] leading-relaxed text-ink-muted">
          Flint 不直接修改 git config（避免污染你的提交身份）。手动加速两种方式：
        </p>
        <ol className="mt-2 list-decimal space-y-1 pl-5 text-[12.5px] text-ink-muted">
          <li>
            <code className="rounded bg-surface-sunken px-1 py-0.5 font-mono text-[11.5px] text-ink">
              git clone https://gh-proxy.com/https://github.com/owner/repo
            </code>
          </li>
          <li>
            全局 <code className="font-mono text-[11.5px]">git config --global url."https://gh-proxy.com/https://github.com/".insteadOf "https://github.com/"</code>
          </li>
        </ol>
      </article>
    </div>
  );
}

function MirrorCard(props: {
  title: string;
  description: string;
  current: string;
  inCN: boolean;
  options: { value: string; label: string }[];
  busy: boolean;
  onApply: (url: string) => void;
}) {
  const { title, description, current, inCN, options, busy, onApply } = props;
  return (
    <article className="rounded-xl border border-line bg-surface p-5 shadow-[0_1px_2px_rgba(31,30,27,0.03)]">
      <div className="flex items-start justify-between gap-4">
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <h3 className="text-[14px] font-medium text-ink">{title}</h3>
            {inCN ? (
              <span className="rounded-full bg-success-soft px-2 py-0.5 text-[10.5px] font-medium text-success">
                国内
              </span>
            ) : current ? (
              <span className="rounded-full bg-surface-sunken px-2 py-0.5 text-[10.5px] font-medium text-ink-muted">
                官方
              </span>
            ) : null}
          </div>
          <p className="mt-1 text-[12.5px] text-ink-muted">{description}</p>
          {current && (
            <p className="mt-1.5 truncate font-mono text-[11.5px] text-ink-faint">
              当前：{current}
            </p>
          )}
        </div>
        <div className="w-72 shrink-0">
          <select
            className="h-9 w-full appearance-none rounded-lg border border-line bg-surface pl-3 pr-8 text-[12.5px] text-ink transition hover:border-line-strong focus:border-accent focus:outline-none disabled:cursor-not-allowed disabled:opacity-60"
            defaultValue=""
            onChange={(e) => {
              if (e.target.value) {
                onApply(e.target.value);
                e.target.value = "";
              }
            }}
            disabled={busy}
          >
            <option value="" disabled>
              切换到…
            </option>
            {options.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>
        </div>
      </div>
    </article>
  );
}

function RocketIcon(props: { className?: string }) {
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
      <path d="M9 2c4 0 5 1 5 5l-4 4-3-3 4-4c0-1-1-2-2-2z" />
      <path d="M7 8 3 12l1 1 4-4" />
      <path d="M4 13c-1 1-2 1-2 1s0-1 1-2" />
    </svg>
  );
}

function StepBadge(props: { n: number; done: boolean }) {
  return (
    <span
      className={
        "mt-0.5 inline-flex h-6 w-6 shrink-0 items-center justify-center rounded-full text-[12px] font-semibold " +
        (props.done
          ? "bg-success-soft text-success"
          : "bg-surface-sunken text-ink-muted")
      }
    >
      {props.done ? "✓" : props.n}
    </span>
  );
}

/* ---------- Tab button ---------- */

function TabButton(props: {
  active: boolean;
  onClick: () => void;
  label: string;
  badge?: number;
}) {
  const { active, onClick, label, badge } = props;
  return (
    <button
      type="button"
      onClick={onClick}
      className={
        "relative -mb-px inline-flex h-9 items-center gap-1.5 border-b-2 px-3 text-[13px] font-medium transition " +
        (active
          ? "border-accent text-ink"
          : "border-transparent text-ink-muted hover:text-ink")
      }
    >
      {label}
      {typeof badge === "number" && badge > 0 && (
        <span
          className={
            "ml-0.5 inline-flex h-4 min-w-4 items-center justify-center rounded-full px-1 text-[10px] font-medium tabular-nums " +
            (active
              ? "bg-accent-soft text-accent-deep"
              : "bg-surface-sunken text-ink-faint")
          }
        >
          {badge}
        </span>
      )}
    </button>
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
