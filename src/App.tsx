import { useEffect, useRef, useState } from "react";
import { invoke, Channel } from "@tauri-apps/api/core";
import { save, open } from "@tauri-apps/plugin-dialog";
import { TabBar } from "./TabBar";
import { DashboardView } from "./DashboardView";
import { PresetsView } from "./PresetsView";
import { WslView } from "./WslView";
import { MirrorsView } from "./MirrorsView";
import { SnapshotView } from "./SnapshotView";
import { DiagnosticModal } from "./DiagnosticModal";
import { RefreshIcon } from "./icons";
import { logClass } from "./format";
import type {
  DiagnosticReport,
  InstallEvent,
  MirrorStatus,
  ParameterOption,
  ParamMap,
  PresetFull,
  PresetMeta,
  ToolMeta,
  ToolStatus,
  View,
  WslStatus,
} from "./types";

type PresetProgress = {
  presetName: string;
  index: number;
  total: number;
  currentTool: string;
} | null;

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
  const [presetProgress, setPresetProgress] = useState<PresetProgress>(null);
  const [wsl, setWsl] = useState<WslStatus | null>(null);
  const [wslBusy, setWslBusy] = useState(false);
  const [mirror, setMirror] = useState<MirrorStatus | null>(null);
  const [mirrorBusy, setMirrorBusy] = useState(false);
  const [diagTarget, setDiagTarget] = useState<string | null>(null);
  const [diagReport, setDiagReport] = useState<DiagnosticReport | null>(null);
  const [diagLoading, setDiagLoading] = useState(false);
  const [dynamicVersions, setDynamicVersions] = useState<
    Record<string, ParameterOption[]>
  >({});
  const [versionsLoading, setVersionsLoading] = useState<string | null>(null);
  const settled = useRef(false);
  const logEndRef = useRef<HTMLDivElement | null>(null);

  const installedCount = tools.filter((t) => t.installed).length;
  const totalCount = tools.length;

  // ----- refresh all state from backend -----
  async function refresh() {
    if (busy) return;
    setRefreshing(true);
    // Each invoke is wrapped in `.catch` with a sensible default — the
    // dashboard must not surface "[error] io error" for an expected
    // absence (e.g. wsl.exe on a Win10 box without WSL).
    const [status, m, p, ws, mi] = await Promise.all([
      invoke<ToolStatus[]>("detect_environment").catch((e) => {
        setLogs((cur) => [...cur, `[error] detect_environment: ${String(e)}`]);
        return [] as ToolStatus[];
      }),
      invoke<ToolMeta[]>("list_installable_tools").catch((e) => {
        setLogs((cur) => [...cur, `[error] list_installable_tools: ${String(e)}`]);
        return [] as ToolMeta[];
      }),
      invoke<PresetMeta[]>("list_presets").catch(() => [] as PresetMeta[]),
      invoke<WslStatus>("wsl_status").catch(() => null),
      invoke<MirrorStatus>("mirror_status").catch(() => ({
        npm: null,
        pip: null,
      })),
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
    setRefreshing(false);
  }

  useEffect(() => {
    void refresh();
  }, []);

  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: "smooth", block: "end" });
  }, [logs]);

  // ----- shared streaming-runner for any IPC that takes a Channel -----
  function runStreaming(
    cmd:
      | "install_tool"
      | "wsl_enable"
      | "wsl_install_dev_tools"
      | "import_snapshot",
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
        if (event.type === "RestoreSection") {
          setLogs((cur) => [...cur, `── ${event.name} ──`]);
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

  // ----- dashboard: lazily fetch the real version list for a tool -----
  // Triggered on dropdown focus. Cached per tool — only the first focus hits
  // the backend (which may run fnm / a network call). Failures are silent;
  // the card keeps showing the recipe's static options.
  async function loadVersions(id: string) {
    if (dynamicVersions[id] || versionsLoading === id) return;
    setVersionsLoading(id);
    try {
      const opts = await invoke<ParameterOption[]>("list_tool_versions", {
        toolId: id,
      });
      if (opts.length > 0) {
        setDynamicVersions((cur) => ({ ...cur, [id]: opts }));
      }
    } catch {
      // keep static options
    } finally {
      setVersionsLoading(null);
    }
  }

  // ----- dashboard: single-tool install / version switch -----
  async function installOne(id: string) {
    if (busy) return;
    const tool = tools.find((t) => t.id === id);
    const switching = tool?.installed ?? false;
    const targetVer = params[id]
      ? Object.values(params[id])[0]
      : undefined;
    setBusy(true);
    setLogs(
      switching
        ? [`正在切换 ${tool?.display_name ?? id}${targetVer ? ` 到 ${targetVer}` : ""}（将覆盖当前安装）`]
        : [],
    );
    setBusyTool(id);
    const res = await runStreaming("install_tool", {
      id,
      params: params[id] ?? {},
    });
    setBusy(false);
    setBusyTool(null);
    if (res.ok) {
      const ver = res.version ? ` · v${res.version}` : "";
      setLogs((cur) => [...cur, `✓ ${switching ? "切换成功" : "安装成功"}${ver}`]);
    } else {
      setLogs((cur) => [...cur, `✗ ${res.error ?? (switching ? "切换失败" : "安装失败")}`]);
    }
    void refresh();
  }

  // ----- snapshot: export -----
  async function exportSnapshot() {
    if (busy) return;
    const path = await save({
      title: "导出环境快照",
      defaultPath: "flint-snapshot.json",
      filters: [{ name: "Flint 快照", extensions: ["json"] }],
    });
    if (!path) return; // user cancelled
    try {
      await invoke("export_snapshot", { path });
      setLogs((cur) => [...cur, `✓ 已导出快照：${path}`]);
    } catch (e) {
      setLogs((cur) => [...cur, `✗ 导出失败：${String(e)}`]);
    }
  }

  // ----- snapshot: import (smart restore) -----
  async function importSnapshot() {
    if (busy) return;
    const selected = await open({
      title: "选择要还原的快照",
      multiple: false,
      filters: [{ name: "Flint 快照", extensions: ["json"] }],
    });
    if (!selected || typeof selected !== "string") return; // cancelled
    setBusy(true);
    setLogs([`从快照还原：${selected}`]);
    const res = await runStreaming("import_snapshot", { path: selected });
    setBusy(false);
    if (res.ok) {
      setLogs((cur) => [...cur, "✓ 还原完成"]);
    } else {
      setLogs((cur) => [...cur, `✗ ${res.error ?? "还原过程中有工具失败"}`]);
    }
    void refresh();
  }

  // ----- presets: batch install -----
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
    });

    for (let i = 0; i < full.tools.ids.length; i++) {
      const id = full.tools.ids[i];
      setPresetProgress({ presetName: full.meta.display_name, index: i, total: full.tools.ids.length, currentTool: id });
      setLogs((cur) => [...cur, `[preset] ▶ ${id}`]);
      const toolParams = {
        ...(params[id] ?? {}),
        ...(full.tools.params[id] ?? {}),
      };
      const res = await runStreaming("install_tool", { id, params: toolParams });
      if (res.ok) {
        succeeded.push(id);
        setLogs((cur) => [
          ...cur,
          `✓ ${id}${res.version ? ` · v${res.version}` : ""}`,
        ]);
      } else {
        failed.push(id);
        setLogs((cur) => [...cur, `✗ ${id}：${res.error ?? "失败"}`]);
      }
    }
    void skipped; // (skipped detection happens on next refresh)

    setBusy(false);
    setBusyTool(null);
    setLogs((cur) => [
      ...cur,
      ``,
      `[preset] 总结：✓ ${succeeded.length} · ✗ ${failed.length}`,
    ]);
    setPresetProgress(null);
    void refresh();
  }

  // ----- WSL -----
  async function wslEnable() {
    if (wslBusy) return;
    setWslBusy(true);
    setLogs([]);
    setBusyTool("wsl-enable");
    const res = await runStreaming("wsl_enable", {});
    setBusyTool(null);
    setWslBusy(false);
    if (!res.ok) setLogs((cur) => [...cur, `✗ ${res.error ?? "WSL 启用失败"}`]);
    void refresh();
  }

  async function wslInstallDevTools() {
    if (wslBusy) return;
    setWslBusy(true);
    setLogs([]);
    setBusyTool("wsl-dev-tools");
    const res = await runStreaming("wsl_install_dev_tools", {});
    setBusyTool(null);
    setWslBusy(false);
    if (!res.ok)
      setLogs((cur) => [...cur, `✗ ${res.error ?? "WSL 内开发环境安装失败"}`]);
    void refresh();
  }

  // ----- mirrors -----
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
          changed ? `✓ ${kind} 已切到国内源` : `[skip] ${kind} 已是国内源`,
        ]);
      }
      void refresh();
    } catch (e) {
      setLogs((cur) => [...cur, `✗ 国内加速失败：${String(e)}`]);
    } finally {
      setMirrorBusy(false);
    }
  }

  // ----- diagnose -----
  async function openDiagnostic(toolId: string) {
    setDiagTarget(toolId);
    setDiagReport(null);
    setDiagLoading(true);
    try {
      const r = await invoke<DiagnosticReport>("diagnose_tool", { toolId });
      setDiagReport(r);
    } catch (e) {
      setDiagReport({
        tool_id: toolId,
        findings: [
          {
            severity: "error",
            message: `诊断失败：${String(e)}`,
            suggestion: null,
          },
        ],
      });
    } finally {
      setDiagLoading(false);
    }
  }

  function closeDiagnostic() {
    setDiagTarget(null);
    setDiagReport(null);
  }

  // ----- param helpers -----
  function setParam(toolId: string, key: string, value: string) {
    setParams((cur) => ({
      ...cur,
      [toolId]: { ...(cur[toolId] ?? {}), [key]: value },
    }));
  }

  // ----- render -----
  return (
    <main className="min-h-screen bg-cream text-ink">
      <div className="mx-auto flex min-h-screen w-full max-w-6xl flex-col px-8 py-8">
        <header className="flex items-end justify-between gap-6 pb-6">
          <div>
            <h1 className="font-sans text-[26px] font-semibold leading-none tracking-[-0.01em] text-ink">
              Flint<span className="text-accent">.</span>
            </h1>
            <p className="mt-2 text-[13px] text-ink-muted">
              一击点燃你的开发环境
            </p>
          </div>
          <div className="flex items-center gap-5">
            <div className="text-[13px] tabular-nums text-ink-muted">
              <span className="font-medium text-success">{installedCount}</span>
              <span className="mx-1.5 text-ink-faint">/</span>
              <span className="font-medium text-ink">{totalCount}</span>
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

        <TabBar view={view} onChange={setView} presetCount={presets.length} />

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
          <div>
            {view === "dashboard" && (
              <DashboardView
                meta={meta}
                tools={tools}
                busy={busy}
                busyTool={busyTool}
                params={params}
                dynamicVersions={dynamicVersions}
                versionsLoading={versionsLoading}
                onParamChange={setParam}
                onLoadVersions={(id) => void loadVersions(id)}
                onInstall={(id) => void installOne(id)}
                onDiagnose={(id) => void openDiagnostic(id)}
              />
            )}
            {view === "presets" && (
              <PresetsView
                presets={presets}
                onApply={(id) => void applyPreset(id)}
                busy={busy}
                presetProgress={presetProgress}
              />
            )}
            {view === "wsl" && (
              <WslView
                status={wsl}
                busy={wslBusy}
                onEnable={() => void wslEnable()}
                onInstallDevTools={() => void wslInstallDevTools()}
              />
            )}
            {view === "mirrors" && (
              <MirrorsView
                status={mirror}
                busy={mirrorBusy}
                onApplyNpm={(url) => void applyNpmMirror(url)}
                onApplyPip={(url) => void applyPipMirror(url)}
                onAccelerate={() => void applyDomestic()}
              />
            )}
            {view === "snapshot" && (
              <SnapshotView
                busy={busy}
                onExport={() => void exportSnapshot()}
                onImport={() => void importSnapshot()}
              />
            )}
          </div>

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
          <span className="font-mono">v0.5.0</span>
        </footer>
      </div>

      <DiagnosticModal
        toolId={diagTarget}
        report={diagReport}
        loading={diagLoading}
        onClose={closeDiagnostic}
      />
    </main>
  );
}
