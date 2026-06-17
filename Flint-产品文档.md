# Flint — 一击点燃你的开发环境

> 从零到能用 AI 写代码，只需一步。

## 1. 产品概述

Flint 是一款面向 Vibecoder 和开发者的桌面端开发环境配置工具。通过可视化界面，用户可以一键完成运行时安装、AI 编程工具部署、开发环境初始化等操作，将原本需要数小时的环境搭建压缩到几分钟。

Flint 这个名字取自英文"燧石"——原始人类点燃第一把火的工具。正如燧石是文明的起点，Flint 是每一个开发者旅程的起点。

### 1.1 核心问题

对于传统程序员，环境配置不难但烦——每次换电脑、重装系统都要重复一遍。对于正在涌入的 Vibecoder 群体，环境配置是劝退第一关：他们可能连终端都没打开过，却被要求先装 Node.js、配 PATH、解决权限问题，然后才能开始用 Claude Code 写第一行代码。

现有工具要么只管单一维度（nvm 只管 Node 版本、Homebrew 只管 macOS 包），要么对新手不友好（asdf/mise 是纯 CLI），要么不覆盖 AI 编程工具链。

### 1.2 产品定位

**一句话定位：** AI 时代的开发环境启动器。

**核心差异：**

- 面向 AI 编程时代——内置 Claude Code、OpenCode、Codex CLI、Cursor 等 AI 工具的原生一键安装
- 现代运行时全覆盖——Node.js、Bun、Python 多版本管理 + GitHub CLI 开箱即用
- 中国开发者友好——npm/pip/GitHub 镜像一键切换，代理配置一步搞定
- 不只装工具，还配好环境——安装完就能用，不留"装了但跑不起来"的坑

### 1.3 目标用户

| 用户类型 | 画像 | 核心痛点 | 使用场景 |
|---------|------|---------|---------|
| Vibecoder / AI 编程新手 | 非科班、设计师转型、产品经理学编程、学生 | 不会配环境，教程看不懂，一个报错卡半天 | 第一次用 Claude Code / Cursor，需要从零搭环境 |
| 独立开发者 | 有经验但怕麻烦，经常换设备 | 每次重装系统要花半天配环境 | 新电脑到手、系统重装后快速恢复 |
| 团队 Leader | 带新人、管团队环境一致性 | 新人入职环境搭建耗时，各人环境不一致导致 bug | 共享团队标准环境配置，新人一键同步 |

**优先级：Vibecoder > 独立开发者 > 团队 Leader**

---

## 2. 功能设计

### 2.1 P0 · 核心功能（MVP）

#### 2.1.1 环境检测仪表盘

打开 Flint 的第一屏。自动扫描当前系统环境，以直观的仪表盘形式展示：

- 已安装的运行时及版本（Node.js、Bun、Python、Git、GitHub CLI 等）
- 已安装的 AI 编程工具（Claude Code、OpenCode、Codex、Cursor、Windsurf 等）
- 系统状态（操作系统版本、WSL 状态、Shell 类型、包管理器）
- 缺失项高亮提示，附带一键安装按钮

设计原则：用户打开 Flint 就能看到"我的电脑现在能干什么、还缺什么"，不需要自己去终端查。

#### 2.1.2 运行时一键安装

| 运行时 / 工具 | 说明 |
|-------------|------|
| Node.js | 可选版本（LTS / Latest / 指定版本），底层调用 fnm |
| Bun | JavaScript / TypeScript 全能运行时，内置包管理器、打包器和测试框架，安装后即可替代 Node + npm + npx 全家桶 |
| Python | 可选版本，底层调用 pyenv 或系统包管理器 |
| Git | 安装并完成基础配置（用户名、邮箱、默认分支名） |
| GitHub CLI (gh) | GitHub 官方命令行工具，支持 PR、Issue、仓库管理、Actions 等全套操作 |
| 包管理器 | pnpm、yarn、uv、pip 等按需安装 |

版本选择提供推荐标注（如"Claude Code 推荐 Node 18+"），降低用户选择成本。Bun 作为新一代运行时单独推荐，适合追求速度和简洁的用户。

#### 2.1.3 AI 编程工具一键安装

| 工具 | 安装方式 | 备注 |
|-----|---------|------|
| Claude Code | 原生安装器：macOS/Linux `curl -fsSL https://claude.ai/install.sh \| sh`，Windows `irm https://claude.ai/install.ps1 \| iex`，备选 Homebrew / winget / npm | 官方推荐原生安装，零依赖、自动更新，无需预装 Node.js |
| OpenCode | 原生安装器：`curl -fsSL https://opencode.ai/install \| bash`，备选 npm / Homebrew / Go install | 开源 AI 编程 Agent，支持 75+ 模型提供商（含 Ollama 本地模型），165k+ GitHub Stars |
| OpenAI Codex CLI | `npm install -g @openai/codex` | 需要 Node.js 环境 |
| Cursor | 下载安装包并引导安装 | 检测是否已安装 |
| Windsurf | 下载安装包并引导安装 | 检测是否已安装 |
| Trae | 下载安装包并引导安装 | 检测是否已安装 |

安装优先级策略：**原生安装器 > 系统包管理器（Homebrew / winget） > npm 全局安装**。原生安装器零依赖、自带自动更新，是最省心的方式。

安装完成后自动验证：运行 `claude --version`、`opencode --version` 等命令确认工具可用，不可用则给出诊断建议。

#### 2.1.4 WSL 一键配置（Windows 专属）

针对 Windows 用户提供完整的 WSL 开发环境搭建：

1. 检测 WSL 是否已启用
2. 启用 WSL 功能（需管理员权限，引导用户授权）
3. 安装 Ubuntu 发行版
4. 在 WSL 内自动配置基础开发环境（Git、Node、Bun、Python、Claude Code）
5. 配置 Windows Terminal 集成

### 2.2 P1 · 增强体验

#### 2.2.1 Preset 配置档

预设场景化套装，一键应用：

| Preset 名称 | 包含内容 |
|-------------|---------|
| 🤖 Vibecoder 全家桶 | Node.js LTS + Bun + Git + GitHub CLI + Claude Code + OpenCode + Cursor + pnpm |
| 🐍 Python AI 开发者 | Python 3.12+ + uv + Git + GitHub CLI + Cursor |
| 🌐 全栈新手包 | Node.js + Bun + Python + Git + GitHub CLI + VSCode + pnpm + uv |
| ⚡ Claude Code 极速包 | Claude Code 原生安装（最小安装，零依赖） |
| 🔓 开源极客包 | Bun + OpenCode + Git + GitHub CLI + Ollama（全部免费/开源） |

用户也可以自定义 Preset 并导出为 JSON 分享。

#### 2.2.2 镜像与代理配置

这是国内用户的刚需：

- npm 镜像切换（官方源 / 淘宝镜像 / 自定义）
- pip 镜像切换（PyPI / 清华源 / 阿里源）
- GitHub 加速（ghproxy 等方案）
- 终端代理设置（HTTP/SOCKS5，支持从系统代理读取）
- 一个总开关：「国内加速模式」，一键切换全部镜像

#### 2.2.3 VSCode / Cursor 插件包

按场景打包常用插件，一键安装：

| 插件包 | 包含插件 |
|-------|---------|
| AI 编程必备 | Cline、Continue、GitHub Copilot |
| 前端开发 | ESLint、Prettier、Tailwind CSS IntelliSense、Auto Rename Tag |
| Python 开发 | Python、Pylance、Jupyter |
| 通用效率 | GitLens、Error Lens、Todo Tree、Material Icon Theme |

#### 2.2.4 Shell 美化

一键安装并配置终端美化方案：

- Oh My Zsh / Oh My Posh（Windows）一键安装
- Starship prompt 主题
- 终端配色方案选择（Catppuccin、Dracula、Nord 等）
- 常用 alias 预配置（`ll`、`gs`、`gp` 等）

### 2.3 P2 · 差异化功能

#### 2.3.1 环境快照与迁移

- 导出当前完整环境配置为 `flint-snapshot.json`（运行时版本、全局包列表、镜像配置、Shell 配置等）
- 在新设备上导入快照，一键还原整套环境
- 支持云端同步（可选，绑定 GitHub Gist）

#### 2.3.2 团队共享配置

- 团队 Leader 配好标准环境后，导出为可分享的 Flint 配置链接
- 新成员打开链接 → Flint 自动应用配置
- 可指定必装项和可选项，保留个人自由度

#### 2.3.3 故障诊断

针对常见的"装了但跑不起来"问题提供自动诊断：

- Claude Code 问题排查：`claude doctor` 自动诊断 → PATH 检测 → 安装方式检测（原生 vs npm）→ 网络连通性 → API 认证状态
- OpenCode 问题排查：安装方式检测 → 模型 Provider 配置 → API Key 有效性
- Bun 问题排查：PATH 检测 → 与 Node 共存冲突排查
- Python 环境混乱：多版本冲突检测、虚拟环境状态检查
- Git / GitHub CLI 配置问题：SSH key 检测、gh auth 状态、credential helper
- 给出具体修复建议，而非泛泛的"请检查环境变量"

---

## 3. 技术方案

### 3.1 技术栈

| 层级 | 选型 | 理由 |
|-----|------|------|
| 框架 | Tauri v2 | 体积小（~5MB vs Electron ~150MB）、性能好、原生系统调用方便 |
| 前端 | React + TypeScript | 生态成熟，组件丰富 |
| 样式 | Tailwind CSS | 快速开发，一致性好 |
| 后端逻辑 | Rust（Tauri 核心） | 系统调用、进程管理、文件操作 |
| 安装编排 | Shell 脚本 + Rust 调度 | 各平台安装脚本独立维护，Rust 层负责调度和状态管理 |

### 3.2 架构概览

```
┌──────────────────────────────────────────────────┐
│                   Flint UI (React)                │
│  ┌──────────┐ ┌──────────┐ ┌──────────────────┐  │
│  │ 仪表盘    │ │ 安装中心  │ │ 配置 & Preset    │  │
│  └──────────┘ └──────────┘ └──────────────────┘  │
├──────────────────────────────────────────────────┤
│              Tauri IPC Bridge                     │
├──────────────────────────────────────────────────┤
│              Flint Core (Rust)                    │
│  ┌──────────┐ ┌──────────┐ ┌──────────────────┐  │
│  │ 环境检测  │ │ 安装引擎  │ │ 配置管理         │  │
│  │ Detector  │ │ Installer │ │ ConfigManager    │  │
│  └──────────┘ └──────────┘ └──────────────────┘  │
│  ┌──────────┐ ┌──────────┐ ┌──────────────────┐  │
│  │ 脚本执行  │ │ 日志系统  │ │ 快照 / 迁移      │  │
│  │ Executor  │ │ Logger   │ │ Snapshot         │  │
│  └──────────┘ └──────────┘ └──────────────────┘  │
├──────────────────────────────────────────────────┤
│           操作系统 (Windows / macOS / Linux)       │
└──────────────────────────────────────────────────┘
```

### 3.3 核心模块说明

**Detector（环境检测器）**
- 通过 `which` / `where` 定位已安装工具
- 执行 `--version` 获取版本信息
- 检查 PATH、环境变量、权限等
- Windows 上额外检查 WSL 状态、Windows Terminal 配置

**Installer（安装引擎）**
- 维护每个工具的安装脚本（按平台区分）
- 支持安装队列：用户勾选多个工具后批量安装
- 实时输出安装日志，前端展示进度
- 安装后自动验证（smoke test）

**ConfigManager（配置管理器）**
- 管理 Preset 的增删改查
- 读写镜像配置（.npmrc、pip.conf 等）
- 管理 Shell 配置文件（.zshrc、.bashrc、PowerShell profile）

**Snapshot（快照系统）**
- 收集当前环境元数据
- 序列化为 JSON
- 支持导入还原和差异对比

### 3.4 安装脚本策略

不重复造轮子，底层调用成熟工具：

| 任务 | Windows | macOS | Linux |
|-----|---------|-------|-------|
| Node.js 版本管理 | fnm (winget) | fnm (brew) | fnm (curl) |
| Bun | `irm bun.sh/install.ps1 \| iex` | `curl -fsSL https://bun.com/install \| bash` | 同 macOS |
| Python 版本管理 | Python 官方安装器 | pyenv (brew) | pyenv (curl) |
| Git | winget / 官方安装器 | Xcode CLT / brew | apt / dnf |
| GitHub CLI | winget / 官方安装器 | brew | apt (官方源) |
| Claude Code | 原生安装器 (PowerShell) | 原生安装器 (curl) | 原生安装器 (curl) |
| OpenCode | npm / Go install | brew / curl | curl / npm |
| 包管理器 (pnpm/uv) | npm 全局安装 | npm 全局安装 | npm 全局安装 |
| Shell 美化 | Oh My Posh | Oh My Zsh | Oh My Zsh |

Flint 的价值不在于写一套新的安装逻辑，而在于把这些散落的工具编排成一个连贯、可视化、可一键执行的流程。

---

## 4. 用户体验流程

### 4.1 首次使用（Vibecoder 典型路径）

```
下载并打开 Flint
    ↓
环境检测仪表盘（看到"你的电脑还缺 Node.js、Bun、Git、Claude Code"）
    ↓
选择 Preset「Vibecoder 全家桶」
    ↓
一键安装（进度条 + 实时日志）
    ↓
安装完成 → 自动验证 → 显示"✅ Claude Code 已就绪"
    ↓
可选：开启「国内加速模式」
    ↓
打开终端，开始 vibe coding
```

### 4.2 老用户换电脑

```
旧电脑：Flint → 导出快照
    ↓
新电脑：下载 Flint → 导入快照 → 一键还原
```

### 4.3 团队新人入职

```
Leader 分享 Flint 配置链接
    ↓
新人打开链接 → Flint 自动安装团队标准环境
    ↓
新人可在标准环境基础上添加个人偏好
```

---

## 5. 跨平台策略

### 5.1 优先级

| 优先级 | 平台 | 理由 |
|-------|------|------|
| P0 | Windows（含 WSL） | Vibecoder 主力平台，WSL 配置是最大痛点 |
| P1 | macOS | 开发者第二大平台，配置相对简单但仍有价值 |
| P2 | Linux | 用户基数小，且 Linux 用户通常自己能搞定 |

### 5.2 平台差异处理

- Windows：重点做 WSL 全流程、winget 集成、PowerShell 配置
- macOS：Homebrew 集成、Xcode CLI Tools 检测与安装
- Linux：apt/dnf 适配，但 MVP 阶段可暂缓

---

## 6. 竞品分析

| 产品 | 定位 | 优势 | 劣势 |
|-----|------|------|------|
| Homebrew | macOS/Linux 包管理 | 生态丰富，社区大 | 仅限 macOS/Linux，CLI only，不覆盖 AI 工具 |
| Volta | Node.js 版本管理 | 快速、简单 | 只管 Node，不覆盖其他 |
| mise / asdf | 多语言版本管理 | 覆盖面广 | 纯 CLI，新手不友好，不管 AI 工具 |
| Ninite | Windows 批量装软件 | 方便 | 不覆盖开发工具，不可配置 |
| webinstall.dev (webi) | 一行命令装开发工具 | 覆盖面不错 | 无 GUI，无 AI 工具，无中国镜像 |
| **Flint** | AI 时代开发环境启动器 | GUI 可视化 + AI 工具链 + 中国镜像 + 环境诊断 | 需要建立用户认知 |

---

## 7. 路线图

### Phase 1 · MVP（4-6 周）

- 环境检测仪表盘
- Node.js / Bun / Python / Git / GitHub CLI 一键安装（Windows + macOS）
- Claude Code（原生安装器）/ OpenCode / Codex CLI 一键安装
- WSL 一键配置（Windows）
- 3-4 个内置 Preset

### Phase 2 · 体验完善（4 周）

- 镜像 / 代理配置模块
- VSCode / Cursor 插件包管理
- Shell 美化一键配置
- 自定义 Preset 导入导出

### Phase 3 · 生态拓展（持续）

- 环境快照与迁移
- 团队共享配置
- 故障诊断引擎
- 社区 Preset 市场（用户贡献配置方案）
- 国际化（英文版）

---

## 8. 关键指标

| 指标 | 说明 | 目标（上线 3 个月） |
|-----|------|-------------------|
| 下载量 | Flint 安装包下载次数 | 5,000+ |
| 安装成功率 | 用户选择安装的工具最终成功安装的比例 | > 95% |
| Preset 使用率 | 使用内置 Preset 完成安装的用户占比 | > 60% |
| 快照导出量 | 导出环境快照的用户数 | 500+ |
| GitHub Stars | 开源仓库星标数 | 1,000+ |

---

## 9. 商业模式（远期）

MVP 阶段完全免费开源，积累用户和口碑。远期可探索：

- **团队版订阅**：团队配置共享、集中管理、审计日志（面向企业 / 培训机构）
- **Preset 市场**：社区贡献的高质量配置方案，优质方案可付费
- **品牌合作**：AI 编程工具官方推荐安装渠道（如 Claude Code 官方推荐用 Flint 安装）

---

## 10. 风险与应对

| 风险 | 影响 | 应对策略 |
|-----|------|---------|
| 系统权限问题 | Windows 安装需要管理员权限，用户可能不理解 | 提供清晰的权限说明弹窗，解释为什么需要权限 |
| AI 工具安装方式变更 | Claude Code 等工具的安装命令可能随版本更新 | 安装脚本独立于核心代码，支持热更新 |
| 跨平台兼容性 | 不同系统版本表现不一 | MVP 聚焦 Windows 10/11 + macOS 12+，明确声明支持范围 |
| 镜像源不稳定 | 国内镜像偶尔不可用 | 内置多个备选镜像，自动切换 |
| 用户信任 | 一键脚本涉及系统修改，用户可能不信任 | 完全开源，安装前预览将执行的操作，提供详细日志 |

---

*Flint — 让每个人都能 30 秒内开始 AI 编程。*
