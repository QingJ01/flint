# Flint 技术设计文档

> 日期：2026-06-17  
> 状态：已通过头脑风暴评审，进入实施  
> 配套产品文档：[`Flint-产品文档.md`](../../Flint-产品文档.md)

本文档把产品文档的"做什么"翻译成"怎么建"。只覆盖技术决策与架构；具体到任务的执行步骤见 [Slice 0 实施计划](./2026-06-17-flint-slice-0-plan.md)。

---

## 1. 范围

Flint 是一款桌面端开发环境启动器，面向 Vibecoder 与开发者，把"装运行时 + 装 AI 编程工具 + 配镜像/PATH + WSL"压缩成可视化一键流程。

- **首发平台**：Windows 10 1809+ / Windows 11（含 WSL）。macOS 为 Phase 1 后期，Linux 暂缓。
- **设计目标**：打开即看到环境状态、一键安装、安装完就能用（不留"装了跑不起来"的坑）、全程不需要管理员（仅 WSL 启用例外）。

---

## 2. 技术栈

| 层级 | 选型 | 理由 |
|-----|------|------|
| 桌面框架 | Tauri v2 | 体积小（~5MB）、原生调用方便、安全模型清晰 |
| 前端 | React + TypeScript + Vite | 生态成熟 |
| 样式 | Tailwind CSS | 快速、一致 |
| 核心逻辑 | Rust（Tauri 后端） | 进程调度、PATH/注册表、提权、文件操作 |
| 安装编排 | TOML 配方 + Rust 调度 | 配方可热更新，安装命令与二进制解耦 |
| 包管理 | pnpm（前端）+ Cargo（Rust） | |

---

## 3. 架构总览

```
┌───────────────────────────────────────────────────┐
│                 Flint UI (React)                  │
│   仪表盘   │   安装中心   │   配置 & Preset         │
├───────────────────────────────────────────────────┤
│              Tauri IPC / Channel                  │
├───────────────────────────────────────────────────┤
│                 Flint Core (Rust)                 │
│  Detector │ Installer │ Executor │ ConfigManager  │
│           │ (Recipe)  │          │ Snapshot       │
├───────────────────────────────────────────────────┤
│            OS (Windows / macOS / Linux)           │
└───────────────────────────────────────────────────┘
```

数据与配置层：`src-tauri/resources/recipes/*.toml`（安装配方）+ 用户配置存 `%APPDATA%\flint\`。

---

## 4. 核心模块

### 4.1 Detector（环境检测器）
- 通过 `where`/`which` 定位可执行文件，执行 `--version` 取版本。
- 检查用户级 PATH（`HKCU\Environment`）、Shell 类型、包管理器。
- Windows 额外检测 WSL 状态（`wsl --status`）、VirtualMachinePlatform 是否已启用。
- 产出结构化 `DetectionResult`，前端仪表盘直接消费。

### 4.2 Executor（命令执行器）
- spawn 子进程，**实时**读 stdout/stderr 并按行推送（经 Tauri Channel）。
- 支持超时、取消、退出码捕获、环境变量注入。
- **默认以当前用户权限运行**；仅 `requires_elevation: true` 的步骤走提权子进程（见 §6）。

### 4.3 Installer + Recipe（安装引擎 + 配方）
- 每个工具一份 TOML 配方：`detect`、各平台 `install` 命令、`verify` 命令、版本选项、`requires_elevation`、`depends_on`、`path_entries`（需加入用户 PATH 的目录）。
- Rust 只解释和调度配方，不含具体安装逻辑。
- 支持安装队列（多工具批量、按依赖排序）、进度、失败诊断。

### 4.4 ConfigManager（配置管理器）
- 读写镜像配置（`.npmrc`、`pip.conf`）。
- 管理用户 PATH（`HKCU\Environment`，广播 `WM_SETTINGCHANGE`）。
- 管理 Shell 配置（PowerShell profile）。

### 4.5 Snapshot（快照系统，Phase 3）
- 收集环境元数据 → 序列化为 `flint-snapshot.json` → 导入还原 / 差异对比。

### 4.6 IPC（前后端桥）
- `#[tauri::command]` 暴露同步操作（detect、queue_install、apply_preset…）。
- **高频日志/进度走 Tauri v2 `Channel<T>`**（比 `emit` 更高效、按 session 隔离）。
- 每次 install 分配 `session_id`，Rust 推 `{ session_id, stream, line, ts }`。

---

## 5. 关键架构决策

1. **安装配方即数据（TOML）**。安装命令抽到 `resources/recipes/*.toml`，随包发布、可从远端（GitHub release）热更新覆盖。直接消解产品文档"AI 工具安装方式变更"风险——改配方不重编二进制。

2. **日志流用 Tauri Channel**。安装是高频流式输出，`Channel<T>` 按会话隔离、避免全局事件广播开销。

3. **Platform trait 从第一天抽象**。`trait Platform { resolve_install_cmd(), elevate(), shell_rc_paths(), ... }`。Windows 实装，macOS/Linux 留 stub。成本几乎为零，避免后续重写。

4. **提权模型：默认零提权，仅 WSL 一步例外**。见下节。

---

## 6. 提权模型（已用官方来源验证，2026-06）

**核心结论：Flint 以普通权限运行，几乎所有安装都不需要管理员。唯一躲不开的是 WSL 的 `VirtualMachinePlatform` 功能启用。**

| 任务 | 默认要管理员？ | Flint 采用的零提权方案 |
|------|:---:|------|
| fnm | ❌ | winget 用户级 / portable zip（装 `%LOCALAPPDATA%`） |
| Node（经 fnm） | ❌ | node 版本落 `%LOCALAPPDATA%`，用户级 |
| Bun | ❌ | `irm bun.sh/install.ps1 \| iex` → `~/.bun`（写 HKCU） |
| Claude Code 原生 | ❌ | `irm https://claude.ai/install.ps1 \| iex` → `~/.local/bin`（官方明确无需管理员） |
| OpenCode | ❌ | `npm i -g opencode-ai`（默认前缀 `%AppData%\npm`）或 scoop |
| Codex CLI | ❌ | `npm i -g @openai/codex` |
| Python | ❌ | 官方安装器 "Install just for me" → `%LocalAppData%\Programs\Python` |
| Git | ⚠️ 默认要 | `winget --scope user` 或 PortableGit（**不用默认 MSI**） |
| GitHub CLI | ⚠️ 默认要 | Scoop / Webi / portable .exe（**不用 MSI**） |
| 改用户 PATH | ❌ | `HKCU\Environment` / `setx`（仅对新进程生效） |
| **WSL 启用** | ✅ **要** | **单步提权**：`VirtualMachinePlatform` 启用须 DISM 提权一次；之后引擎/发行版均为用户级 |

**实现约束：**
- Executor 默认以当前用户跑命令；仅 `requires_elevation: true` 的配方步骤拉一个提权子进程（UAC），做完即止。
- 配方必须编码**零提权变体**：Git/gh 用 scoop/portable 而非 MSI；优先原生安装器（curl/irm）而非 winget，行为更确定。
- `~/.local/bin`（Claude Code）默认不在 Windows PATH 上 → Flint 自动加用户 PATH 并提示重启终端，否则用户以为装失败。
- **避开 nvm-windows**：它创建符号链接需提权，不是零提权方案——这正是选 fnm 的原因。

> 验证细节与官方来源链接见会话校验记录；结论高置信度（多数 HIGH）。

---

## 7. 一次安装的完整数据流

```
用户点"安装 Node"
  → UI 调 ipc: install(recipe_id="node", version="lts")
  → Rust Installer：建 session_id，发 Channel("session_start")
  → Executor 以当前用户 spawn: fnm install --lts
     ├─ stdout 行 → Channel("log", {session_id, stream:"stdout", line})
     └─ 进度估算 → Channel("progress", {session_id, pct})
  → 退出码 0 → Executor 跑 verify: node -v
  → ConfigManager 把 fnm shim 目录写入用户 PATH（HKCU）
  → 发 Channel("session_done", {session_id, ok:true, version:"v20.x"})
  → UI 仪表盘 Node 状态翻绿
```

失败路径：退出码非 0 → 收集诊断上下文（最后 N 行日志 + 已知错误模式表）→ `session_done.ok=false` + 诊断建议。

---

## 8. 跨平台策略

| 平台 | 优先级 | 重点 |
|------|:---:|------|
| Windows（含 WSL） | P0 | winget 用户级、PowerShell profile、WSL 全流程、HKCU PATH |
| macOS | P1 | Homebrew、Xcode CLT 检测（Phase 1 后期，靠 Platform trait 接入） |
| Linux | P2 | apt/dnf（MVP 暂缓） |

---

## 9. 实施切片

每个切片都是一个**可运行、可演示**的 app。

- **Slice 0 · 行走骨架（~1 周）**：Tauri 工程跑起来；Detector 检 fnm/Node；Executor 实时日志；一条完整链路——检测缺 Node → fnm 装 Node LTS → 日志 → `node -v` 验证 → 仪表盘翻绿。**打通即证明 IPC/进程/日志/验证四件事。**
- **Slice 1 · 检测仪表盘完整化（~1 周）**：Detector 覆盖全部 P0 目标；仪表盘 UI（已装/版本/缺失高亮/一键安装）。
- **Slice 2 · 安装中心横向铺开（~2 周）**：把单工具模式复制到全部运行时与 AI 工具；配方表化；安装队列 + 诊断。
- **Slice 3 · WSL 全流程 + Preset（~2 周）**：WSL 检测→启用（唯一提权步）→装 Ubuntu→WSL 内配环境→Windows Terminal；3-4 个内置 Preset。

---

## 10. 待执行期验证的开放项

实施时需用官方来源二次确认（不靠记忆）：
- Tauri v2 当前脚手架命令与 `Channel<T>` API 形态。
- 各 AI 工具安装命令的 2026 最新版（配方化后可热更新，但首发要准）。
- fnm 在 PowerShell 的 shell 集成片段（`fnm env --use-on-cd`）当前写法。
- 代码签名方案（公开分发前需要，避免 SmartScreen 拦截）——非 Slice 0 阻塞项。

---

## 11. 风险与对策

| 风险 | 对策 |
|------|------|
| 安装命令随版本变更 | 配方即数据 + 远端热更新 |
| 用户信任（一键改系统） | 完全开源；安装前预览将执行的操作；详细日志 |
| 国内镜像不稳 | 内置多备选镜像，自动切换（Phase 2） |
| 提权弹窗惊吓用户 | 仅 WSL 一步需提权，弹窗前给清晰说明 |
| 跨系统版本差异 | MVP 聚焦 Win10 1809+/Win11，明确声明支持范围 |
