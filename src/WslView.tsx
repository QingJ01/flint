import type { WslStatus } from "./types";

const stateLabel: Record<WslStatus["state"], string> = {
  "not-installed": "未启用",
  enabled: "已启用（无发行版）",
  ready: "就绪",
  unknown: "状态未知",
};

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
  const { status, busy, onEnable, onInstallDevTools } = props;
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
