import type { MessageKey } from "./zh";

// English strings. Typed as `Record<MessageKey, string>` so tsc fails the
// build if any key is missing or extra relative to zh.ts.
export const en: Record<MessageKey, string> = {
  // app shell
  "app.tagline": "Ignite your dev environment in one click",
  "app.ready": "ready",
  "app.refresh": "Re-scan",
  "app.refreshing": "Scanning",

  // tabs
  "tab.dashboard": "Dashboard",
  "tab.presets": "Presets",
  "tab.wsl": "WSL",
  "tab.mirrors": "Mirrors",
  "tab.snapshot": "Snapshot",

  // language switch
  "lang.zh": "中文",
  "lang.en": "EN",
};
