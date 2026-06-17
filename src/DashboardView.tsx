import { ToolCard } from "./ToolCard";
import {
  categoryLabel,
  categoryOrder,
} from "./constants";
import type {
  ToolMeta,
  ToolStatus,
  ParamMap,
} from "./types";

type Props = {
  meta: ToolMeta[];
  tools: ToolStatus[];
  busy: boolean;
  busyTool: string | null;
  params: ParamMap;
  onParamChange: (toolId: string, key: string, value: string) => void;
  onInstall: (id: string) => void;
  onDiagnose: (id: string) => void;
};

export function DashboardView(props: Props) {
  const {
    meta,
    tools,
    busy,
    busyTool,
    params,
    onParamChange,
    onInstall,
    onDiagnose,
  } = props;

  const statusById = new Map(tools.map((t) => [t.id, t]));

  return (
    <div className="flex flex-col gap-8">
      {categoryOrder.map((category) => {
        const sectionMetas = meta.filter((m) => m.category === category);
        if (sectionMetas.length === 0) return null;
        const installedCount = sectionMetas.filter(
          (m) => statusById.get(m.id)?.installed,
        ).length;
        return (
          <section key={category}>
            <div className="mb-3 flex items-baseline justify-between border-b border-line pb-2">
              <h2 className="text-[11px] font-semibold uppercase tracking-[0.12em] text-ink-faint">
                {categoryLabel[category]}
              </h2>
              <span className="text-[11px] tabular-nums text-ink-faint">
                {installedCount} / {sectionMetas.length} 已就绪
              </span>
            </div>
            <div className="grid gap-3 sm:grid-cols-2">
              {sectionMetas.map((m) => (
                <ToolCard
                  key={m.id}
                  meta={m}
                  status={statusById.get(m.id)}
                  busy={busy}
                  busyTool={busyTool}
                  paramValue={params[m.id]?.[m.parameters[0]?.key ?? ""]}
                  onParamChange={(key, value) => onParamChange(m.id, key, value)}
                  onInstall={onInstall}
                  onDiagnose={onDiagnose}
                />
              ))}
            </div>
          </section>
        );
      })}
    </div>
  );
}
