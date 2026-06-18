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

  // common
  "common.diagnose": "Diagnose",
  "common.installing": "Installing…",
  "common.switching": "Switching…",

  // tool card
  "tool.detecting": "Detecting…",
  "tool.notInstalled": "Not installed",
  "tool.installed": "Installed",
  "tool.install": "Install",
  "tool.switchVersion": "Switch version",
  "tool.reinstall": "Reinstall",
  "tool.reinstalling": "Reinstalling…",
  "tool.sameVersion": "Current version",
  "tool.sameVersionTip": "Already the current version",
  "tool.upgradeLatest": "Upgrade to latest",
  "tool.upgrading": "Upgrading…",

  // dashboard
  "dash.sectionReady": "{installed} / {total} ready",

  // presets
  "presets.empty": "No presets available.",
  "presets.apply": "Apply preset",
  "presets.oneClick": "One-click bundle install",
  "presets.welcomeTitle": "👋 Welcome! From zero to AI-coding-ready — just pick one bundle",
  "presets.welcomeBody1": "Not sure what to install? Click any “",
  "presets.welcomeApply": "Apply preset",
  "presets.welcomeBody2": "” below and Flint installs the whole set at once — no picking tool by tool. When in doubt, choose ",
  "presets.welcomeBundle": "Vibecoder Bundle",
  "presets.welcomeBody3": ". Want to pick yourself? Switch to “Dashboard” above and install one by one.",

  // diagnostic modal
  "diag.title": "Diagnostics · {tool}",
  "diag.summaryPass": "{n} passed",
  "diag.summaryWarn": "{n} warnings",
  "diag.summaryError": "{n} errors",
  "diag.close": "Close",
  "diag.keyPrompt": "Verify API key validity (online · free)",
  "diag.verifyKey": "Verify key",
  "diag.verifying": "Verifying…",
  "diag.checking": "Checking…",
  "diag.noRules": "No diagnostic rules for this tool.",
  "diag.keyFailed": "Verification failed: {err}",

  // dashboard categories
  "cat.runtime": "Runtimes & Core Tools",
  "cat.ai": "AI Coding Tools",

  // snapshot
  "snapshot.title": "Snapshot & Migrate",
  "snapshot.intro":
    "Export this environment to a flint-snapshot.json and restore it on another machine in one click: missing tools get installed, npm/pip mirrors get applied. Restore only fills gaps — it never uninstalls anything or touches your PATH.",
  "snapshot.export": "Export snapshot",
  "snapshot.import": "Restore from snapshot…",
  "snapshot.preview": "Current snapshot preview",
  "snapshot.reading": "Reading…",
  "snapshot.installedTools": "Installed tools ({count}):",
  "snapshot.none": "none",
  "snapshot.npm": "npm registry",
  "snapshot.pip": "pip index",
  "snapshot.wsl": "WSL",
  "snapshot.default": "Default (official)",
  "snapshot.notDetected": "not detected",
  "snapshot.wslReady": "Ready",
  "snapshot.wslEnabled": "Enabled (no distro)",
  "snapshot.wslNotInstalled": "Not installed",
  "snapshot.wslUnknown": "Unknown",

  // mirrors
  "mirrors.cnTitle": "China acceleration",
  "mirrors.cnDesc": "Switch npm + pip to China mirrors in one click. For GitHub, see the “gh-proxy” section below.",
  "mirrors.accelerate": "Accelerate",
  "mirrors.accelerating": "Switching…",
  "mirrors.npmTitle": "npm registry",
  "mirrors.npmDesc": "Pinned via ~/.npmrc. Affects all Node downloads (npm install / npx).",
  "mirrors.pipTitle": "pip index-url",
  "mirrors.pipDesc":
    "Pinned via pip.ini (Windows) / pip.conf (POSIX). Affects all Python downloads (pip install / uv).",
  "mirrors.badgeCN": "China",
  "mirrors.badgeOfficial": "Official",
  "mirrors.current": "Current: ",
  "mirrors.switchTo": "Switch to…",
  "mirrors.ghTitle": "GitHub clone acceleration",
  "mirrors.ghDesc": "Flint doesn't touch your git config (to avoid polluting your commit identity). Two manual options:",
  "mirrors.ghGlobal": "Globally",

  // wsl
  "wsl.stateNotInstalled": "Not enabled",
  "wsl.stateEnabled": "Enabled (no distro)",
  "wsl.stateReady": "Ready",
  "wsl.stateUnknown": "Unknown",
  "wsl.detecting": "Detecting…",
  "wsl.currentState": "Status: {state}",
  "wsl.defaultDistro": " · Default distro: {distro}",
  "wsl.installedDistros": "Installed distros: {distros}",
  "wsl.step1Title": "Enable WSL and install Ubuntu",
  "wsl.step1Desc":
    "First-time enable needs one-off admin rights (Windows shows a UAC dialog). When done, open a new PowerShell and run wsl --status to verify.",
  "wsl.enable": "Enable WSL",
  "wsl.done": "✓ Done",
  "wsl.step2Title": "Install the dev environment in Ubuntu",
  "wsl.step2Desc":
    "Installs Git, Node LTS, Bun, Python, uv, Claude Code as root inside the WSL Ubuntu distro. Takes about 3-5 minutes.",
  "wsl.installDev": "Install WSL dev environment",
  "wsl.ready": "✓ Ready",
  "wsl.footer1": "After installing, run wsl in PowerShell to enter Ubuntu; or wsl -d Ubuntu code . to open VS Code in WSL (needs VS Code installed on Windows).",

  // App — logs / dialogs / footer
  "log.switching": "Switching {name} to {ver} (overwrites current install)",
  "log.switchingNoVer": "Switching {name} (overwrites current install)",
  "log.switchOk": "✓ Switched{ver}",
  "log.installOk": "✓ Installed{ver}",
  "log.switchFail": "✗ Switch failed",
  "log.installFail": "✗ Install failed",
  "dialog.exportTitle": "Export environment snapshot",
  "dialog.snapshotFilter": "Flint snapshot",
  "dialog.importTitle": "Choose a snapshot to restore",
  "log.exportOk": "✓ Snapshot exported: {path}",
  "log.exportFail": "✗ Export failed: {err}",
  "log.restoreFrom": "Restoring from snapshot: {path}",
  "log.restoreOk": "✓ Restore complete",
  "log.restoreFail": "✗ Some tools failed during restore",
  "log.presetLoadFail": "[error] Failed to load preset {id}: {err}",
  "log.presetItemFail": "✗ {id}: {err}",
  "log.fail": "failed",
  "log.presetSummary": "[preset] Summary: ✓ {ok} · ✗ {fail}",
  "log.wslEnableFail": "✗ WSL enable failed",
  "log.wslDevFail": "✗ WSL dev environment install failed",
  "log.npmOk": "✓ npm registry set: {url}",
  "log.npmSkip": "[skip] npm registry already set to this value",
  "log.npmFail": "✗ npm mirror failed: {err}",
  "log.pipOk": "✓ pip index-url set: {url}",
  "log.pipSkip": "[skip] pip index-url already set to this value",
  "log.pipFail": "✗ pip mirror failed: {err}",
  "log.cnOk": "✓ {kind} switched to China mirror",
  "log.cnSkip": "[skip] {kind} already on China mirror",
  "log.cnFail": "✗ China acceleration failed: {err}",
  "log.diagFail": "Diagnostics failed: {err}",
  "log.title": "Install log",
  "log.lines": "{n} lines",
  "log.idle": "Idle",
  "log.empty": "No logs yet. Click “Install” on any tool card to start.",
  "footer.brand": "Flint · Ignite your dev environment",
};
