import { SpinnerIcon } from "./icons";
import type { PresetMeta } from "./types";

type PresetProgress = {
  presetName: string;
  index: number;
  total: number;
  currentTool: string;
} | null;

type Props = {
  presets: PresetMeta[];
  onApply: (id: string) => void;
  busy: boolean;
  presetProgress: PresetProgress;
};

export function PresetsView(props: Props) {
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
