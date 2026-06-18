import { RocketIcon } from "./icons";
import {
  CN_NPM_HOSTS,
  CN_PIP_HOSTS,
  NPM_MIRRORS,
  PIP_MIRRORS,
  isCN,
} from "./constants";
import { useT } from "./i18n";
import type { MirrorStatus } from "./types";

type Props = {
  status: MirrorStatus | null;
  busy: boolean;
  onApplyNpm: (url: string) => void;
  onApplyPip: (url: string) => void;
  onAccelerate: () => void;
};

export function MirrorsView(props: Props) {
  const t = useT();
  const { status, busy, onApplyNpm, onApplyPip, onAccelerate } = props;
  const npmCurrent = status?.npm ?? "";
  const pipCurrent = status?.pip ?? "";
  const npmInCN = isCN(npmCurrent, CN_NPM_HOSTS);
  const pipInCN = isCN(pipCurrent, CN_PIP_HOSTS);

  return (
    <div className="flex flex-col gap-4">
      <article className="flex items-center justify-between gap-4 rounded-xl border border-accent-soft bg-accent-soft/30 p-5">
        <div className="min-w-0 flex-1">
          <h2 className="text-[15px] font-medium text-ink">{t("mirrors.cnTitle")}</h2>
          <p className="mt-1 text-[12.5px] text-ink-muted">
            {t("mirrors.cnDesc")}
          </p>
        </div>
        <button
          type="button"
          onClick={onAccelerate}
          disabled={busy}
          className="inline-flex h-9 items-center gap-2 rounded-lg bg-accent px-4 text-[13px] font-medium text-white shadow-[0_1px_2px_rgba(204,120,92,0.4)] transition hover:bg-accent-deep disabled:opacity-50"
        >
          <RocketIcon className="h-3.5 w-3.5" />
          {busy ? t("mirrors.accelerating") : t("mirrors.accelerate")}
        </button>
      </article>

      <MirrorCard
        title={t("mirrors.npmTitle")}
        description={t("mirrors.npmDesc")}
        current={npmCurrent}
        inCN={npmInCN}
        options={NPM_MIRRORS}
        busy={busy}
        onApply={onApplyNpm}
      />

      <MirrorCard
        title={t("mirrors.pipTitle")}
        description={t("mirrors.pipDesc")}
        current={pipCurrent}
        inCN={pipInCN}
        options={PIP_MIRRORS}
        busy={busy}
        onApply={onApplyPip}
      />

      <article className="rounded-xl border border-line bg-surface p-5">
        <h3 className="text-[14px] font-medium text-ink">{t("mirrors.ghTitle")}</h3>
        <p className="mt-1 text-[12.5px] leading-relaxed text-ink-muted">
          {t("mirrors.ghDesc")}
        </p>
        <ol className="mt-2 list-decimal space-y-1 pl-5 text-[12.5px] text-ink-muted">
          <li>
            <code className="rounded bg-surface-sunken px-1 py-0.5 font-mono text-[11.5px] text-ink">
              git clone https://gh-proxy.com/https://github.com/owner/repo
            </code>
          </li>
          <li>
            {t("mirrors.ghGlobal")}{" "}
            <code className="rounded bg-surface-sunken px-1 py-0.5 font-mono text-[11px] text-ink">
              git config --global url."https://gh-proxy.com/https://github.com/".insteadOf "https://github.com/"
            </code>
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
  const t = useT();
  const { title, description, current, inCN, options, busy, onApply } = props;
  return (
    <article className="rounded-xl border border-line bg-surface p-5 shadow-[0_1px_2px_rgba(31,30,27,0.03)]">
      <div className="flex items-start justify-between gap-4">
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <h3 className="text-[14px] font-medium text-ink">{title}</h3>
            {inCN ? (
              <span className="rounded-full bg-success-soft px-2 py-0.5 text-[10.5px] font-medium text-success">
                {t("mirrors.badgeCN")}
              </span>
            ) : current ? (
              <span className="rounded-full bg-surface-sunken px-2 py-0.5 text-[10.5px] font-medium text-ink-muted">
                {t("mirrors.badgeOfficial")}
              </span>
            ) : null}
          </div>
          <p className="mt-1 text-[12.5px] text-ink-muted">{description}</p>
          {current && (
            <p className="mt-1.5 truncate font-mono text-[11.5px] text-ink-faint">
              {t("mirrors.current")}{current}
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
              {t("mirrors.switchTo")}
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
