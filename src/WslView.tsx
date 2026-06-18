import type { WslStatus } from "./types";
import { useT } from "./i18n";

const stateDot: Record<WslStatus["state"], string> = {
  "not-installed": "bg-danger",
  enabled: "bg-warn",
  ready: "bg-success",
  unknown: "bg-ink-faint/40",
};

type Props = {
  status: WslStatus | null;
  busy: boolean;
  onEnable: () => void;
  onInstallDevTools: () => void;
};

export function WslView(props: Props) {
  const t = useT();
  const { status, busy, onEnable, onInstallDevTools } = props;
  const step1Done = status?.state === "enabled" || status?.state === "ready";
  const step2Done = status?.state === "ready";

  const stateLabel = (state: WslStatus["state"]): string => {
    switch (state) {
      case "not-installed":
        return t("wsl.stateNotInstalled");
      case "enabled":
        return t("wsl.stateEnabled");
      case "ready":
        return t("wsl.stateReady");
      default:
        return t("wsl.stateUnknown");
    }
  };

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
                ? t("wsl.currentState", { state: stateLabel(status.state) }) +
                  (status.default_distro
                    ? t("wsl.defaultDistro", { distro: status.default_distro })
                    : "")
                : t("wsl.detecting")}
            </p>
            {status?.distros && status.distros.length > 0 && (
              <p className="mt-1 font-mono text-[11.5px] text-ink-faint">
                {t("wsl.installedDistros", { distros: status.distros.join(", ") })}
              </p>
            )}
            {status?.kernel_version && (
              <p className="mt-1 font-mono text-[11.5px] text-ink-faint">
                {status.kernel_version}
              </p>
            )}
            {status?.state === "unknown" && status.raw && (
              <p className="mt-1 font-mono text-[11px] text-warn">
                {status.raw}
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
              {t("wsl.step1Title")}
            </h3>
            <p className="mt-1 text-[12.5px] leading-relaxed text-ink-muted">
              {t("wsl.step1Desc")}
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
              {step1Done ? t("wsl.done") : t("wsl.enable")}
            </button>
          </div>
        </div>
      </article>

      <article className="rounded-xl border border-line bg-surface p-5 shadow-[0_1px_2px_rgba(31,30,27,0.03)]">
        <div className="flex items-start gap-4">
          <StepBadge n={2} done={step2Done} />
          <div className="min-w-0 flex-1">
            <h3 className="text-[14px] font-medium text-ink">
              {t("wsl.step2Title")}
            </h3>
            <p className="mt-1 text-[12.5px] leading-relaxed text-ink-muted">
              {t("wsl.step2Desc")}
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
              {step2Done ? t("wsl.ready") : t("wsl.installDev")}
            </button>
          </div>
        </div>
      </article>

      <p className="px-1 text-[11.5px] text-ink-faint">
        {t("wsl.footer1")}
      </p>
    </div>
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
