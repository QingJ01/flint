import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Snapshot } from "./types";
import { useT } from "./i18n";

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
  const t = useT();
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
        <h2 className="text-[15px] font-semibold text-ink">{t("snapshot.title")}</h2>
        <p className="mt-1.5 text-[13px] leading-relaxed text-ink-muted">
          {t("snapshot.intro")}
        </p>
        <div className="mt-4 flex gap-2.5">
          <button
            type="button"
            onClick={onExport}
            disabled={busy}
            className="inline-flex h-9 items-center gap-1.5 rounded-lg bg-accent px-4 text-[13px] font-medium text-white transition hover:bg-accent-deep disabled:cursor-not-allowed disabled:opacity-50"
          >
            {t("snapshot.export")}
          </button>
          <button
            type="button"
            onClick={onImport}
            disabled={busy}
            className="inline-flex h-9 items-center gap-1.5 rounded-lg border border-line bg-surface px-4 text-[13px] font-medium text-ink transition hover:border-line-strong hover:bg-cream-deep disabled:cursor-not-allowed disabled:opacity-50"
          >
            {t("snapshot.import")}
          </button>
        </div>
      </div>

      <div className="rounded-xl border border-line bg-surface p-5">
        <h3 className="text-[13px] font-semibold text-ink">{t("snapshot.preview")}</h3>
        {!snap ? (
          <p className="mt-2 text-[13px] text-ink-faint">{t("snapshot.reading")}</p>
        ) : (
          <div className="mt-3 space-y-3 text-[13px]">
            <div>
              <span className="text-ink-muted">{t("snapshot.installedTools", { count: installed.length })}</span>
              <div className="mt-1.5 flex flex-wrap gap-1.5">
                {installed.length === 0 ? (
                  <span className="text-ink-faint">{t("snapshot.none")}</span>
                ) : (
                  installed.map((tool) => (
                    <span
                      key={tool.id}
                      className="inline-flex items-center gap-1 rounded-md bg-surface-sunken px-2 py-0.5 text-[12px] text-ink"
                    >
                      {tool.display_name}
                      {tool.version && (
                        <span className="text-ink-faint">v{tool.version}</span>
                      )}
                    </span>
                  ))
                )}
              </div>
            </div>
            <div className="grid grid-cols-2 gap-x-4 gap-y-1.5 border-t border-line pt-3">
              <SnapRow label={t("snapshot.npm")} value={snap.npm_registry ?? t("snapshot.default")} />
              <SnapRow label={t("snapshot.pip")} value={snap.pip_registry ?? t("snapshot.default")} />
              <SnapRow
                label={t("snapshot.wsl")}
                value={snap.wsl ? wslLabel(snap.wsl.state, t) : t("snapshot.notDetected")}
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

function wslLabel(state: string, t: ReturnType<typeof useT>): string {
  switch (state) {
    case "ready":
      return t("snapshot.wslReady");
    case "enabled":
      return t("snapshot.wslEnabled");
    case "not-installed":
      return t("snapshot.wslNotInstalled");
    default:
      return t("snapshot.wslUnknown");
  }
}
