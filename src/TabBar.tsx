import type { View } from "./types";

type Props = {
  view: View;
  onChange: (v: View) => void;
  presetCount: number;
};

const tabs: { id: View; label: string }[] = [
  { id: "dashboard", label: "仪表盘" },
  { id: "presets", label: "预设" },
  { id: "wsl", label: "WSL" },
  { id: "mirrors", label: "镜像" },
  { id: "snapshot", label: "快照" },
];

export function TabBar(props: Props) {
  const { view, onChange, presetCount } = props;
  return (
    <nav className="mb-5 flex items-center gap-1 border-b border-line">
      {tabs.map((t) => (
        <TabButton
          key={t.id}
          active={view === t.id}
          onClick={() => onChange(t.id)}
          label={t.label}
          badge={t.id === "presets" ? presetCount : undefined}
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
