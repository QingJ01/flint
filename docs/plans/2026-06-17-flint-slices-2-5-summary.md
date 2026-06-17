# Flint Slices 2-5 总结

> 配套实现 commit：a225d30 · 7ec0125 · cfe3bf3 · ee134b2 · f719034 · 6dabe6c
> 状态：MVP Phase 1 全部完成

## Slice 2 · Preset 套件

**目标**：5 个内置组合一键装，1 个总按钮搞定 Vibecoder 入门。

**新增文件：**
- `src-tauri/src/preset.rs` — TOML 预设解析（与 recipe 同结构，独立模块）
- `src-tauri/resources/presets/{vibecoder-stack,python-ai,fullstack-bundle,claude-minimal,open-source-geek}.toml` — 5 个预设
- `src-tauri/resources/recipes/{cursor,pnpm,uv}.toml` — 预设引用但之前没有的 3 个工具

**新增 IPC：**
- `list_presets() -> Vec<PresetMeta>` — 列表
- `get_preset(id) -> PresetFull` — 返回完整预设（含工具 id 列表 + 默认参数覆盖）

**前端：** PresetsView 卡片网格 + 「应用预设」按钮，触发顺序批量 install（带 progress + 跳过已装 + 失败汇总）。

**测试：** 5 个 preset 模块测试（解析 / 列出 / 跳过坏 TOML / 目录缺失 / shipped-recipes 回归）。

---

## Slice 3 · WSL 一键配置

**目标**：把「WSL 启用 → Ubuntu 安装 → 装开发环境」三步串成向导。

**新增文件：**
- `src-tauri/src/wsl.rs` — `wsl --status` 输出解析为 4 态：`not-installed` / `enabled` / `ready` / `unknown`

**新增 IPC：**
- `wsl_status() -> WslStatus` — 仪表盘轮询
- `wsl_enable(on_event)` — `Start-Process wsl -Verb RunAs --args "--install --no-distribution --no-launch"`（触发 UAC）
- `wsl_install_dev_tools(on_event)` — `wsl -u root -d Ubuntu -- bash -c "<apt update; install fnm/Node/Bun/Python/uv/Claude Code>"`（以 root 跳过首次启动的用户配置向导）

**前端：** WslView 三步式向导（状态卡 + 步骤 1「启用 WSL」+ 步骤 2「装开发环境」），每步带完成徽章 + 禁用态管理。

**测试：** 8 个 wsl 模块测试（4 态解析 + 多种语言 / 状态消息）。

---

## Slice 4 · 镜像 / 代理

**目标**：npm / pip 镜像一键切换 + 「国内加速模式」总开关。

**新增模块：**
- `config::build_npmrc(url, existing)` — 纯函数构造新的 `~/.npmrc`，保留无关 key、跳过注释行
- `config::build_pip_conf(url, existing)` — 纯函数构造 `pip.ini` / `pip.conf`，只改 [global] 节，保留 [install] 等其它节
- `config::apply_npm_registry` / `apply_pip_registry` — 写盘（幂等，值未变不写）
- `config::current_npm_registry` / `current_pip_registry` — 读盘

**新增 IPC：**
- `mirror_status() -> MirrorStatus` — 读当前 npm + pip
- `apply_npm_mirror(url)` / `apply_pip_mirror(url)` — 写单个
- `apply_domestic_acceleration() -> Vec<(kind, changed)>` — 写 npm + pip 到国内源

**前端：** MirrorsView — 暖珊瑚色「国内加速模式」卡片（rocket 图标） + 4 源 npm 卡片 + 5 源 pip 卡片 + GitHub 加速提示。徽章动态显示「国内 / 官方」状态。

**测试：** 7 个 config::mirror_tests（npmrc 替换/追加/保留注释 + pip.conf 多个边界）。

---

## Slice 5 · 故障诊断

**目标**：每个工具点「诊断」弹模态，按严重度染色 + 修复建议。

**新增文件：**
- `src-tauri/src/diagnose.rs` — 11 工具 × 多规则 诊断引擎

**诊断规则清单：**
- **通用**：on_path（`which::which`）
- **claude-code**：on_path / `ANTHROPIC_API_KEY` 格式校验 / `claude doctor` 退出码
- **opencode**：on_path / `opencode --version`
- **codex-cli**：on_path / Node.js 依赖
- **node**：on_path / fnm PowerShell profile 集成（写入了 `fnm env` 行？）
- **python**：on_path / `python -m pip --version`
- **git**：on_path / `git config --global user.{name,email}` 都设了？
- **github-cli**：on_path / `gh auth status`
- **bun / pnpm / uv / cursor**：on_path

**新增 IPC：**
- `diagnose_tool(tool_id) -> DiagnosticReport` — 永不返回 Err，内部失败转 severity: error 报告

**前端：** ToolCard 已装工具下方的「诊断」按钮 → DiagnosticModal（modal 居中 + 模糊背景 + 按 ok/warn/error 染色边框 + 💡 建议）

**测试：** 5 个 diagnose 模块测试（severity 比较 / 未知工具返空 / on_path 标记缺失 / API key 警告分支）。

---

## 通用基础设施改进

- **`executor::run` 改造** (Slice 0 已有) 异步 spawn + 双流（stdout/stderr）独立读取 + Channel 推前端染色
- **recipe 参数化**：`{python_version}` / `{node_version}` 占位符 + 默认值 + 失败时报具体 key
- **PATH 持久化**：`add_to_user_path(dir)` 走 HKCU\Environment\Path 注册表，幂等大小写不敏感
- **PS profile 集成**：`ensure_fnm_in_powershell_profiles` 自动写入 `fnm env` 行

## 测试覆盖

```
slice 0-1 既有：         33 tests
slice 2 (preset)：       +5 = 38
slice 3 (wsl)：          +8 = 46
slice 4 (mirror)：       +7 = 53
slice 5 (diagnose)：     +5 = 58
bug 修复后 (wsl)         +2 = 60
───
总计：                   60 tests
```

## 累计 commit 链（自 Slice 0 末）

```
f719034 fix(wsl): wsl.exe present-but-disabled no longer surfaces as io error
6dabe6c refactor(ui): split App.tsx into 12 files
ee134b2 feat(diagnose): per-tool diagnostic engine + modal
cfe3bf3 feat(mirrors): npm/pip registry switch + domestic acceleration
7ec0125 feat(wsl): status detect + UAC enable + WSL dev tools install
a225d30 feat(preset): 5 built-in presets + cursor/pnpm/uv recipes
50cd521 feat(ui): Claude-style redesign
... (Slice 0-1 commits)
```

## 后续候选（按用户价值排序）

1. **环境快照 / 迁移**（P2）— 导出 `flint-snapshot.json` 一键还原整套环境
2. **故障诊断深化** — 实际跑 `claude doctor`、检查 API key 真有效性、网络可达性
3. **Shell 美化** — Oh My Posh / Starship 一键装
4. **VSCode 插件包** — 前端/AI/Python 三套
5. **Ollama 集成** — 开源预设里目前是「需手动」状态
6. **macOS / Linux 端到端验证**
7. **国际化**（英文版）

## 已知遗留

- 安装后必须重开终端（Windows 注册表 vs 进程 env 限制）— 已在日志中提示
- WSL 启用必须 admin — UI 已说明
- macOS / Linux 配方未实测
- Codex 依赖 Node，preset 中未做依赖图（用户应先装 Node 再用 Codex 预设）
