import { ChevronIcon, SpinnerIcon, StethoscopeIcon } from "./icons";
import type { ParameterOption, ToolMeta, ToolStatus } from "./types";
import { statusTextForTool } from "./format";

type Props = {
  meta: ToolMeta;
  status: ToolStatus | undefined;
  busy: boolean;
  busyTool: string | null;
  paramValue: string | undefined;
  /** Dynamically-fetched version options (overrides recipe's static list). */
  versionOptions: ParameterOption[] | undefined;
  versionsLoading: boolean;
  onParamChange: (key: string, value: string) => void;
  onLoadVersions: (id: string) => void;
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
    versionOptions,
    versionsLoading,
    onParamChange,
    onLoadVersions,
    onInstall,
    onDiagnose,
  } = props;
  const installed = status?.installed ?? false;
  const isThisBusy = busyTool === meta.id;
  const isOtherBusy = busy && !isThisBusy;
  const hasVersions = meta.parameters.length > 0;

  // First (and only) version parameter, if any.
  const versionParam = meta.parameters[0];
  // Dynamic options take precedence over the recipe's static list.
  const options = versionOptions ?? versionParam?.options ?? [];

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

      {/* Version dropdown — shown whether or not the tool is installed, so an
          installed tool can switch versions. Lazily fetches the real list on
          first focus. */}
      {hasVersions && versionParam && (
        <div className="flex flex-col gap-2">
          <label className="block">
            <span className="flex items-center gap-1.5 text-[10.5px] font-medium uppercase tracking-[0.08em] text-ink-faint">
              {versionParam.label}
              {versionsLoading && (
                <SpinnerIcon className="h-2.5 w-2.5 animate-spin" />
              )}
            </span>
            <div className="relative mt-1">
              <select
                className="h-8 w-full appearance-none rounded-md border border-line bg-surface pl-2.5 pr-7 text-[12.5px] text-ink transition hover:border-line-strong focus:border-accent focus:outline-none disabled:cursor-not-allowed disabled:opacity-60"
                value={paramValue ?? versionParam.default ?? options[0]?.value ?? ""}
                onChange={(e) => onParamChange(versionParam.key, e.target.value)}
                onFocus={() => onLoadVersions(meta.id)}
                disabled={busy}
              >
                {options.map((opt) => (
                  <option key={opt.value} value={opt.value}>
                    {opt.label}
                  </option>
                ))}
              </select>
              <ChevronIcon className="pointer-events-none absolute right-2 top-1/2 h-3 w-3 -translate-y-1/2 text-ink-faint" />
            </div>
          </label>
        </div>
      )}

      {/* Actions, three shapes:
          - not installed → one full-width 安装 button.
          - installed + has versions (node/python) → 切换版本 + 诊断, side by
            side. 切换 disables when the picked version already matches the
            installed one (no pointless reinstall).
          - installed, no versions (bun/git/gh/cursor) → 诊断 is the primary
            action; 重新安装 is a quiet corner link, since reinstall is rare. */}
      {!installed ? (
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
      ) : hasVersions ? (
        <div className="mt-auto flex gap-2">
          <SwitchButton
            isThisBusy={isThisBusy}
            isOtherBusy={isOtherBusy}
            busy={busy}
            // Disable when the chosen version equals what's already installed.
            sameVersion={
              !!paramValue && paramValue === (status?.version ?? null)
            }
            onClick={() => onInstall(meta.id)}
          />
          <DiagnoseButton onClick={() => onDiagnose(meta.id)} busy={busy} />
        </div>
      ) : meta.id === "git" ? (
        // Git has no version dropdown (its release asset names are
        // irregular), but its recipe installs the *latest* stable MinGit at
        // runtime — so offer a first-class "升级到最新" alongside 诊断.
        <div className="mt-auto flex gap-2">
          <button
            type="button"
            onClick={() => onInstall(meta.id)}
            disabled={busy}
            className={
              "inline-flex h-8 flex-1 items-center justify-center gap-1.5 rounded-lg border px-3 text-[13px] font-medium transition disabled:cursor-not-allowed " +
              (isThisBusy
                ? "border-accent bg-accent text-white"
                : isOtherBusy
                  ? "border-line bg-surface text-ink-faint"
                  : "border-line bg-surface text-ink hover:border-line-strong hover:bg-cream-deep")
            }
          >
            {isThisBusy ? (
              <>
                <SpinnerIcon className="h-3 w-3 animate-spin" />
                升级中…
              </>
            ) : (
              "升级到最新"
            )}
          </button>
          <DiagnoseButton onClick={() => onDiagnose(meta.id)} busy={busy} />
        </div>
      ) : (
        <div className="mt-auto flex flex-col gap-1.5">
          <DiagnoseButton
            onClick={() => onDiagnose(meta.id)}
            busy={busy}
            fullWidth
          />
          <button
            type="button"
            onClick={() => onInstall(meta.id)}
            disabled={busy}
            className="self-center text-[11px] text-ink-faint underline-offset-2 transition hover:text-ink-muted hover:underline disabled:cursor-not-allowed disabled:opacity-50"
          >
            {isThisBusy ? "重装中…" : "重新安装"}
          </button>
        </div>
      )}
    </article>
  );
}

function SwitchButton(props: {
  isThisBusy: boolean;
  isOtherBusy: boolean;
  busy: boolean;
  sameVersion: boolean;
  onClick: () => void;
}) {
  const { isThisBusy, isOtherBusy, busy, sameVersion, onClick } = props;
  const disabled = busy || sameVersion;
  return (
    <button
      type="button"
      onClick={onClick}
      disabled={disabled}
      title={sameVersion ? "已是当前版本" : undefined}
      className={
        "inline-flex h-8 flex-1 items-center justify-center gap-1.5 rounded-lg px-3 text-[13px] font-medium transition " +
        (isThisBusy
          ? "bg-accent text-white shadow-[0_1px_2px_rgba(204,120,92,0.4)]"
          : isOtherBusy || sameVersion
            ? "border border-line bg-surface text-ink-faint cursor-not-allowed"
            : "border border-line bg-surface text-ink hover:border-line-strong hover:bg-cream-deep")
      }
    >
      {isThisBusy ? (
        <>
          <SpinnerIcon className="h-3 w-3 animate-spin" />
          切换中…
        </>
      ) : sameVersion ? (
        "已是此版本"
      ) : (
        "切换版本"
      )}
    </button>
  );
}

function DiagnoseButton(props: {
  onClick: () => void;
  busy: boolean;
  fullWidth?: boolean;
}) {
  return (
    <button
      type="button"
      onClick={props.onClick}
      disabled={props.busy}
      className={
        "inline-flex h-8 items-center justify-center gap-1.5 rounded-lg border border-line bg-surface px-3 text-[13px] font-medium text-ink-muted transition hover:border-line-strong hover:text-ink disabled:cursor-not-allowed disabled:opacity-50 " +
        (props.fullWidth ? "w-full" : "flex-1")
      }
    >
      <StethoscopeIcon className="h-3.5 w-3.5" />
      诊断
    </button>
  );
}
