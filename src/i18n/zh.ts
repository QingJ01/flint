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

  // common
  "common.diagnose": "诊断",
  "common.installing": "安装中…",
  "common.switching": "切换中…",

  // tool card
  "tool.detecting": "检测中…",
  "tool.notInstalled": "未安装",
  "tool.installed": "已安装",
  "tool.install": "安装",
  "tool.switchVersion": "切换版本",
  "tool.reinstall": "重新安装",
  "tool.reinstalling": "重装中…",
  "tool.sameVersion": "已是此版本",
  "tool.sameVersionTip": "已是当前版本",
  "tool.upgradeLatest": "升级到最新",
  "tool.upgrading": "升级中…",

  // dashboard
  "dash.sectionReady": "{installed} / {total} 已就绪",

  // presets
  "presets.empty": "暂无可用预设。",
  "presets.apply": "应用预设",
  "presets.oneClick": "一键组合安装",
  "presets.welcomeTitle": "👋 欢迎！从零到能用 AI 写代码，选一个组合就够了",
  "presets.welcomeBody1": "不确定装什么？点下面任意一个「",
  "presets.welcomeApply": "应用预设",
  "presets.welcomeBody2": "」，Flint 会自动把整套工具一次装好——无需逐个挑选。拿不准就选 ",
  "presets.welcomeBundle": "Vibecoder 全家桶",
  "presets.welcomeBody3": "。想自己挑？切到上方「仪表盘」逐个安装。",

  // diagnostic modal
  "diag.title": "诊断报告 · {tool}",
  "diag.summaryPass": "{n} 项通过",
  "diag.summaryWarn": "{n} 警告",
  "diag.summaryError": "{n} 错误",
  "diag.close": "关闭",
  "diag.keyPrompt": "真实校验 API key 有效性（联网 · 免费）",
  "diag.verifyKey": "验证 Key",
  "diag.verifying": "校验中…",
  "diag.checking": "正在检查…",
  "diag.noRules": "该工具没有可用的诊断规则。",
  "diag.keyFailed": "校验失败：{err}",

  // dashboard categories
  "cat.runtime": "运行时与基础工具",
  "cat.ai": "AI 编程工具",

  // snapshot
  "snapshot.title": "环境快照与迁移",
  "snapshot.intro":
    "把当前这套环境导出成一个 flint-snapshot.json，换机或重装后一键还原：自动安装缺失的工具、应用 npm/pip 镜像。还原只补缺口——不会卸载已有工具，也不改你的 PATH。",
  "snapshot.export": "导出快照",
  "snapshot.import": "从快照还原…",
  "snapshot.preview": "当前快照预览",
  "snapshot.reading": "读取中…",
  "snapshot.installedTools": "已安装工具（{count}）：",
  "snapshot.none": "无",
  "snapshot.npm": "npm 源",
  "snapshot.pip": "pip 源",
  "snapshot.wsl": "WSL",
  "snapshot.default": "默认（官方）",
  "snapshot.notDetected": "未检测",
  "snapshot.wslReady": "已就绪",
  "snapshot.wslEnabled": "已启用（无发行版）",
  "snapshot.wslNotInstalled": "未安装",
  "snapshot.wslUnknown": "未知",

  // mirrors
  "mirrors.cnTitle": "国内加速模式",
  "mirrors.cnDesc": "一键切换 npm + pip 到国内镜像。GitHub 加速请见下方的 “gh-proxy” 链接。",
  "mirrors.accelerate": "一键加速",
  "mirrors.accelerating": "切换中…",
  "mirrors.npmTitle": "npm registry",
  "mirrors.npmDesc": "通过 ~/.npmrc 锁定。影响 npm install / npx 等所有 Node 包下载。",
  "mirrors.pipTitle": "pip index-url",
  "mirrors.pipDesc":
    "通过 pip.ini (Windows) / pip.conf (POSIX) 锁定。影响 pip install / uv 等所有 Python 包下载。",
  "mirrors.badgeCN": "国内",
  "mirrors.badgeOfficial": "官方",
  "mirrors.current": "当前：",
  "mirrors.switchTo": "切换到…",
  "mirrors.ghTitle": "GitHub 克隆加速",
  "mirrors.ghDesc": "Flint 不直接修改 git config（避免污染你的提交身份）。手动加速两种方式：",
  "mirrors.ghGlobal": "全局",

  // wsl
  "wsl.stateNotInstalled": "未启用",
  "wsl.stateEnabled": "已启用（无发行版）",
  "wsl.stateReady": "就绪",
  "wsl.stateUnknown": "状态未知",
  "wsl.detecting": "检测中…",
  "wsl.currentState": "当前状态：{state}",
  "wsl.defaultDistro": " · 默认发行版：{distro}",
  "wsl.installedDistros": "已装发行版：{distros}",
  "wsl.step1Title": "启用 WSL 并安装 Ubuntu",
  "wsl.step1Desc":
    "首次启用需要一次性管理员权限（Windows 会弹 UAC 对话框）。操作完成后新开 PowerShell 运行 wsl --status 验证。",
  "wsl.enable": "启用 WSL",
  "wsl.done": "✓ 已完成",
  "wsl.step2Title": "在 Ubuntu 里装开发环境",
  "wsl.step2Desc":
    "在 WSL 的 Ubuntu 发行版中以 root 身份安装 Git、Node LTS、Bun、Python、uv、Claude Code。约需 3-5 分钟。",
  "wsl.installDev": "安装 WSL 开发环境",
  "wsl.ready": "✓ 已就绪",
  "wsl.footer1": "安装完成后，在 PowerShell 运行 wsl 进入 Ubuntu；或 wsl -d Ubuntu code . 在 WSL 中直接打开 VS Code（需 Windows 端已装 VS Code）。",

  // App — logs / dialogs / footer
  "log.switching": "正在切换 {name} 到 {ver}（将覆盖当前安装）",
  "log.switchingNoVer": "正在切换 {name}（将覆盖当前安装）",
  "log.switchOk": "✓ 切换成功{ver}",
  "log.installOk": "✓ 安装成功{ver}",
  "log.switchFail": "✗ 切换失败",
  "log.installFail": "✗ 安装失败",
  "dialog.exportTitle": "导出环境快照",
  "dialog.snapshotFilter": "Flint 快照",
  "dialog.importTitle": "选择要还原的快照",
  "log.exportOk": "✓ 已导出快照：{path}",
  "log.exportFail": "✗ 导出失败：{err}",
  "log.restoreFrom": "从快照还原：{path}",
  "log.restoreOk": "✓ 还原完成",
  "log.restoreFail": "✗ 还原过程中有工具失败",
  "log.presetLoadFail": "[error] 无法加载预设 {id}：{err}",
  "log.presetItemFail": "✗ {id}：{err}",
  "log.fail": "失败",
  "log.presetSummary": "[preset] 总结：✓ {ok} · ✗ {fail}",
  "log.wslEnableFail": "✗ WSL 启用失败",
  "log.wslDevFail": "✗ WSL 内开发环境安装失败",
  "log.npmOk": "✓ npm registry 已写入：{url}",
  "log.npmSkip": "[skip] npm registry 已是该值",
  "log.npmFail": "✗ npm mirror 失败：{err}",
  "log.pipOk": "✓ pip index-url 已写入：{url}",
  "log.pipSkip": "[skip] pip index-url 已是该值",
  "log.pipFail": "✗ pip mirror 失败：{err}",
  "log.cnOk": "✓ {kind} 已切到国内源",
  "log.cnSkip": "[skip] {kind} 已是国内源",
  "log.cnFail": "✗ 国内加速失败：{err}",
  "log.diagFail": "诊断失败：{err}",
  "log.title": "安装日志",
  "log.lines": "{n} 行",
  "log.idle": "空闲",
  "log.empty": "暂无日志。点击任意工具卡的「安装」开始。",
  "footer.brand": "Flint · 一击点燃你的开发环境",
} as const;

export type MessageKey = keyof typeof zh;
