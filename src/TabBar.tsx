import type { View } from "./types";
import { useT } from "./i18n";
import type { MessageKey } from "./i18n/zh";

type Props = {
  view: View;
  onChange: (v: View) => void;
  presetCount: number;
};

const tabs: { id: View; labelKey: MessageKey }[] = [
  { id: "dashboard", labelKey: "tab.dashboard" },
  { id: "presets", labelKey: "tab.presets" },
  { id: "wsl", labelKey: "tab.wsl" },
  { id: "mirrors", labelKey: "tab.mirrors" },
  { id: "snapshot", labelKey: "tab.snapshot" },
];

export function TabBar(props: Props) {
  const { view, onChange, presetCount } = props;
  const t = useT();
  return (
    <nav className="mb-5 flex items-center gap-1 border-b border-line">
      {tabs.map((tab) => (
        <TabButton
          key={tab.id}
          active={view === tab.id}
          onClick={() => onChange(tab.id)}
          label={t(tab.labelKey)}
          badge={tab.id === "presets" ? presetCount : undefined}
        />
      ))}
    </nav>
  );
}

function TabButton(props: {
  active: boolean;
  onClick: () => void;
  label: string;
  badge?: number;
}) {
  const { active, onClick, label, badge } = props;
  return (
    <button
      type="button"
      onClick={onClick}
      className={
        "relative -mb-px inline-flex h-9 items-center gap-1.5 border-b-2 px-3 text-[13px] font-medium transition " +
        (active
          ? "border-accent text-ink"
          : "border-transparent text-ink-muted hover:text-ink")
      }
    >
      {label}
      {typeof badge === "number" && badge > 0 && (
        <span
          className={
            "ml-0.5 inline-flex h-4 min-w-4 items-center justify-center rounded-full px-1 text-[10px] font-medium tabular-nums " +
            (active
              ? "bg-accent-soft text-accent-deep"
              : "bg-surface-sunken text-ink-faint")
          }
        >
          {badge}
        </span>
      )}
    </button>
  );
}
