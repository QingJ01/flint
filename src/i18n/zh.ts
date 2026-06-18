// Chinese strings — the source of truth for the key set. en.ts is typed
// against `MessageKey`, so a missing or extra key there is a compile error
// (this replaces a runtime dict-parity test — no test runner needed).
// Keys are namespaced `area.meaning`.
export const zh = {
  // app shell
  "app.tagline": "一击点燃你的开发环境",
  "app.ready": "已就绪",
  "app.refresh": "重新检测",
  "app.refreshing": "检测中",

  // tabs
  "tab.dashboard": "仪表盘",
  "tab.presets": "预设",
  "tab.wsl": "WSL",
  "tab.mirrors": "镜像",
  "tab.snapshot": "快照",

  // language switch
  "lang.zh": "中文",
  "lang.en": "EN",
} as const;

export type MessageKey = keyof typeof zh;
