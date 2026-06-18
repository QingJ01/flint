// Color a log line based on its leading tag. Centralized so the
// dashboard, the WSL wizard, the preset runner, and the mirror picker
// all paint logs the same way.

export function logClass(line: string): string {
  if (line.startsWith("[err]") || line.startsWith("✗") || line.startsWith("[error]"))
    return "text-log-err";
  if (line.startsWith("[ok]") || line.startsWith("✓") || line.startsWith("[skip]"))
    return "text-log-ok";
  if (line.startsWith("[warn]") || line.startsWith("[!]")) return "text-log-warn";
  if (line.startsWith("[out]")) return "text-log-faint";
  return "text-log-text";
}

export function statusTextForTool(
  installed: boolean,
  version: string | null,
  t: (key: "tool.notInstalled" | "tool.installed") => string,
): string {
  if (!installed) return t("tool.notInstalled");
  return version ? `v${version}` : t("tool.installed");
}
