import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Snapshot } from "./types";

type Props = {
  busy: boolean;
  onExport: () => void;
  onImport: () => void;
};

/**
 * 环境快照：把当前环境导出成可移植的 JSON，或从快照「智能还原」
 * （只装缺失工具 + 应用镜像，不卸载、不动 PATH）。导出/导入的实际动作
 * （含原生文件选择 + 流式日志）由 App 提供，本视图负责预览 + 触发。
 */
export function SnapshotView(props: Props) {
  const { busy, onExport, onImport } = props;
  const [snap, setSnap] = useState<Snapshot | null>(null);

  useEffect(() => {
    void invoke<Snapshot>("current_snapshot")
      .then(setSnap)
      .catch(() => setSnap(null));
  }, []);

  const installed = snap?.tools.filter((t) => t.installed) ?? [];

  return (
    <div className="space-y-5">
      <div className="rounded-xl border border-line bg-surface p-5">
        <h2 className="text-[15px] font-semibold text-ink">环境快照与迁移</h2>
        <p className="mt-1.5 text-[13px] leading-relaxed text-ink-muted">
          把当前这套环境导出成一个 <code className="rounded bg-surface-sunken px-1 py-0.5 text-[12px]">flint-snapshot.json</code>，
          换机或重装后一键还原：自动安装缺失的工具、应用 npm/pip 镜像。
          还原只补缺口——不会卸载已有工具，也不改你的 PATH。
        </p>
        <div className="mt-4 flex gap-2.5">
          <button
            type="button"
            onClick={onExport}
            disabled={busy}
            className="inline-flex h-9 items-center gap-1.5 rounded-lg bg-accent px-4 text-[13px] font-medium text-white transition hover:bg-accent-deep disabled:cursor-not-allowed disabled:opacity-50"
          >
            导出快照
          </button>
          <button
            type="button"
            onClick={onImport}
            disabled={busy}
            className="inline-flex h-9 items-center gap-1.5 rounded-lg border border-line bg-surface px-4 text-[13px] font-medium text-ink transition hover:border-line-strong hover:bg-cream-deep disabled:cursor-not-allowed disabled:opacity-50"
          >
            从快照还原…
          </button>
        </div>
      </div>

      <div className="rounded-xl border border-line bg-surface p-5">
        <h3 className="text-[13px] font-semibold text-ink">当前快照预览</h3>
        {!snap ? (
          <p className="mt-2 text-[13px] text-ink-faint">读取中…</p>
        ) : (
          <div className="mt-3 space-y-3 text-[13px]">
            <div>
              <span className="text-ink-muted">已安装工具（{installed.length}）：</span>
              <div className="mt-1.5 flex flex-wrap gap-1.5">
                {installed.length === 0 ? (
                  <span className="text-ink-faint">无</span>
                ) : (
                  installed.map((t) => (
                    <span
                      key={t.id}
                      className="inline-flex items-center gap-1 rounded-md bg-surface-sunken px-2 py-0.5 text-[12px] text-ink"
                    >
                      {t.display_name}
                      {t.version && (
                        <span className="text-ink-faint">v{t.version}</span>
                      )}
                    </span>
                  ))
                )}
              </div>
            </div>
            <div className="grid grid-cols-2 gap-x-4 gap-y-1.5 border-t border-line pt-3">
              <SnapRow label="npm 源" value={snap.npm_registry ?? "默认（官方）"} />
              <SnapRow label="pip 源" value={snap.pip_registry ?? "默认（官方）"} />
              <SnapRow
                label="WSL"
                value={snap.wsl ? wslLabel(snap.wsl.state) : "未检测"}
              />
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

function SnapRow(props: { label: string; value: string }) {
  return (
    <div className="flex items-baseline gap-2">
      <span className="text-ink-muted">{props.label}：</span>
      <span className="truncate text-ink" title={props.value}>
        {props.value}
      </span>
    </div>
  );
}

function wslLabel(state: string): string {
  switch (state) {
    case "ready":
      return "已就绪";
    case "enabled":
      return "已启用（无发行版）";
    case "not-installed":
      return "未安装";
    default:
      return "未知";
  }
}
