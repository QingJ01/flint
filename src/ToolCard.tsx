import { ChevronIcon, SpinnerIcon, StethoscopeIcon } from "./icons";
import type { ToolMeta, ToolStatus } from "./types";
import { statusTextForTool } from "./format";

type Props = {
  meta: ToolMeta;
  status: ToolStatus | undefined;
  busy: boolean;
  busyTool: string | null;
  paramValue: string | undefined;
  onParamChange: (key: string, value: string) => void;
  onInstall: (id: string) => void;
  onDiagnose: (id: string) => void;
};

export function ToolCard(props: Props) {
  const {
    meta,
    status,
    busy,
    busyTool,
    paramValue,
    onParamChange,
    onInstall,
    onDiagnose,
  } = props;
  const installed = status?.installed ?? false;
  const isThisBusy = busyTool === meta.id;
  const isOtherBusy = busy && !isThisBusy;

  return (
    <article className="card-enter group relative flex flex-col gap-3 rounded-xl border border-line bg-surface p-4 shadow-[0_1px_2px_rgba(31,30,27,0.03)] transition hover:border-line-strong hover:shadow-[0_2px_6px_rgba(31,30,27,0.05)]">
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0 flex-1">
          <h3 className="truncate text-[15px] font-medium text-ink">
            {meta.display_name}
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
            {status ? statusTextForTool(installed, status.version) : "检测中…"}
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

      {!installed && meta.parameters.length > 0 && (
        <div className="flex flex-col gap-2">
          {meta.parameters.map((p) => (
            <label key={p.key} className="block">
              <span className="block text-[10.5px] font-medium uppercase tracking-[0.08em] text-ink-faint">
                {p.label}
              </span>
              <div className="relative mt-1">
                <select
                  className="h-8 w-full appearance-none rounded-md border border-line bg-surface pl-2.5 pr-7 text-[12.5px] text-ink transition hover:border-line-strong focus:border-accent focus:outline-none disabled:cursor-not-allowed disabled:opacity-60"
                  value={paramValue ?? p.default ?? p.options[0]?.value ?? ""}
                  onChange={(e) => onParamChange(p.key, e.target.value)}
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
          onClick={() => onInstall(meta.id)}
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

      {installed && (
        <button
          type="button"
          onClick={() => onDiagnose(meta.id)}
          className="mt-auto inline-flex h-7 w-full items-center justify-center gap-1.5 rounded-md border border-line bg-surface px-2.5 text-[11.5px] font-medium text-ink-muted transition hover:border-line-strong hover:text-ink"
        >
          <StethoscopeIcon className="h-3 w-3" />
          诊断
        </button>
      )}
    </article>
  );
}
