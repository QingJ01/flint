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
  /** Show the first-run intro banner (set for brand-new / empty environments). */
  showOnboarding?: boolean;
};

export function PresetsView(props: Props) {
  const { presets, onApply, busy, presetProgress, showOnboarding } = props;

  if (presets.length === 0) {
    return (
      <div className="rounded-xl border border-dashed border-line bg-surface-sunken p-8 text-center">
        <p className="text-[13px] text-ink-muted">暂无可用预设。</p>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-4">
      {showOnboarding && (
        <div className="rounded-xl border border-accent-soft/50 bg-accent-soft/15 p-4">
          <h2 className="text-[15px] font-semibold text-ink">
            👋 欢迎！从零到能用 AI 写代码，选一个组合就够了
          </h2>
          <p className="mt-1.5 text-[13px] leading-relaxed text-ink-muted">
            不确定装什么？点下面任意一个「<span className="font-medium text-ink">应用预设</span>」，
            Flint 会自动把整套工具一次装好——无需逐个挑选。
            拿不准就选 <span className="font-medium text-ink">Vibecoder 全家桶</span>。
            想自己挑？切到上方「仪表盘」逐个安装。
          </p>
        </div>
      )}
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
    </div>
  );
}
