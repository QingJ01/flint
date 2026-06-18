// Shared types for the Flint frontend. These mirror the Rust IPC
// payloads — keep them in sync with src-tauri/src/ipc.rs.

export type ToolCategory = "runtime" | "ai-tool";

export type ToolStatus = {
  id: string;
  display_name: string;
  category: ToolCategory;
  installed: boolean;
  version: string | null;
};

export type ParameterOption = { value: string; label: string };

export type ToolParameterMeta = {
  key: string;
  label: string;
  default: string | null;
  options: ParameterOption[];
};

export type ToolMeta = {
  id: string;
  display_name: string;
  category: ToolCategory;
  requires_elevation: boolean;
  parameters: ToolParameterMeta[];
};

export type PresetMeta = {
  id: string;
  display_name: string;
  description: string;
  emoji: string;
};

export type PresetFull = {
  meta: PresetMeta;
  tools: { ids: string[]; params: Record<string, Record<string, string>> };
};

export type WslStatus = {
  state: "not-installed" | "enabled" | "ready" | "unknown";
  default_distro: string | null;
  distros: string[];
  kernel_version: string | null;
  raw: string;
};

export type MirrorStatus = {
  npm: string | null;
  pip: string | null;
};

export type Severity = "ok" | "warn" | "error";
export type Finding = {
  severity: Severity;
  message: string;
  suggestion: string | null;
};
export type DiagnosticReport = {
  tool_id: string;
  findings: Finding[];
};

export type InstallEvent =
  | { type: "Log"; line: string }
  | { type: "Progress"; pct: number }
  | { type: "Done"; ok: boolean; version: string | null; error: string | null }
  | { type: "RestoreSection"; name: string };

export type Snapshot = {
  schema: number;
  tools: ToolStatus[];
  npm_registry: string | null;
  pip_registry: string | null;
  wsl: WslStatus | null;
};

export type View = "dashboard" | "presets" | "wsl" | "mirrors" | "snapshot";

export type ParamMap = Record<string, Record<string, string>>;
