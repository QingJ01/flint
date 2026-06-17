# Flint ⚡ — 一击点燃你的开发环境

> AI 时代的开发环境启动器。从零到能用 AI 写代码，只需一步。

Flint 是一款面向 Vibecoder 和开发者的桌面端工具：可视化一键完成运行时安装、AI 编程工具部署、镜像切换、WSL 配置，把数小时的环境搭建压缩到几分钟。

**当前状态：** MVP Phase 1 完成（仪表盘 / 预设 / WSL / 镜像 / 诊断五大模块）。详细见下方 [路线图](#路线图) 与 [`docs/plans/`](./docs/plans/)。

## 快速开始

需要 Node、pnpm、Rust 工具链（首次约 5-10 分钟编译）：

```powershell
# 装依赖
pnpm install

# 启动 dev 模式（开 Tauri 桌面窗口）
pnpm tauri dev

# 打包发布版（生成 .exe / .msi）
pnpm tauri build

# 跑测试
cd src-tauri && cargo test
pnpm exec tsc --noEmit
```

如果 dev 会话已开但 UI 没刷新：Ctrl-R 或者 `Get-Process flint | Stop-Process -Force` 后重启。

## 五大模块

| Tab | 干什么 |
|---|---|
| **仪表盘** | 11 个工具卡片（Node / Bun / Python / Git / GitHub CLI / Claude Code / OpenCode / Codex CLI / Cursor / pnpm / uv），未装的一键装，已装的点「诊断」 |
| **预设** | 5 个组合一键装：Vibecoder 全家桶 / Python AI / 全栈新手包 / Claude Code 极速包 / 开源极客包 |
| **WSL** | 检测 → 一键启用（UAC 一次性）→ 在 Ubuntu 里装 Git/Node/Bun/Python/uv/Claude Code |
| **镜像** | npm/pip 镜像一键切换；总开关「国内加速模式」一秒钟切到淘宝 + 清华源 |
| **诊断**（卡片按钮） | 模态弹窗，按 ok/warn/error 染色显示问题 + 修复建议 |

## 核心约束

- **零提权优先**：99% 的安装走便携版 zip / 原生安装器 / npm i -g，零 UAC 弹窗
- **仅 WSL 启用一步需要管理员**（系统功能限制，不可绕过）；UI 明确说明
- **路径刷新**：装完需新开终端（Windows 注册表 vs 进程 env 限制），日志里有提示

## 技术栈

- **壳**：Tauri v2（~5MB，比 Electron 轻 30 倍）
- **前端**：React 19 + TypeScript + Tailwind CSS v4（暖米白 + Inter 字体 + 暖珊瑚点缀）
- **后端**：Rust（tokio / serde / toml / which / winreg / regex）
- **安装编排**：TOML 配方（`resources/recipes/*.toml`） + Rust 调度
- **进程流式日志**：Tauri Channel 事件，前端按前缀染色

## 仓库结构

```
.
├── docs/
│   ├── plans/                      # 设计 + 实施计划 + 总结
│   └── (产品文档.md 见根)
├── src/                            # React 前端
│   ├── App.tsx                     # 状态机 + shell
│   ├── TabBar.tsx                  # 4 tab 切换
│   ├── DashboardView.tsx           # 仪表盘
│   ├── ToolCard.tsx                # 单张工具卡
│   ├── PresetsView.tsx             # 预设网格
│   ├── WslView.tsx                 # WSL 向导（3 步）
│   ├── MirrorsView.tsx             # 镜像切换
│   ├── DiagnosticModal.tsx         # 诊断弹窗
│   ├── types.ts                    # TS 类型
│   ├── constants.ts                # 镜像源列表 + 分类
│   ├── format.ts                   # 日志染色 + 状态文案
│   └── icons.tsx                   # 内联 SVG
├── src-tauri/                      # Rust 后端
│   ├── src/
│   │   ├── lib.rs                  # Tauri 命令注册
│   │   ├── ipc.rs                  # 命令 + 流式事件
│   │   ├── recipe.rs               # TOML 配方 (含参数化)
│   │   ├── preset.rs               # TOML 预设
│   │   ├── detector.rs             # 11 工具检测
│   │   ├── executor.rs             # async 进程 + 日志流
│   │   ├── config.rs               # PATH/npmrc/pip.conf/PS profile
│   │   ├── wsl.rs                  # WSL 状态检测
│   │   └── diagnose.rs             # 11 工具诊断规则
│   └── resources/
│       ├── recipes/                # 11 个 .toml 配方
│       └── presets/                # 5 个 .toml 预设
└── package.json + Cargo.toml
```

## 文档

- [产品文档](./Flint-产品文档.md) — 定位、用户、竞品、商业模式
- [技术设计](./docs/plans/2026-06-17-flint-design.md) — 架构、核心模块、IPC 协议
- [Slice 0 实施计划](./docs/plans/2026-06-17-flint-slice-0-plan.md) — 行走骨架
- [Slice 1 设计 + 验证清单](./docs/plans/2026-06-17-flint-slice-1-design.md) — 8 工具安装
- [Slice 2-5 总结](./docs/plans/2026-06-17-flint-slices-2-5-summary.md) — 预设/WSL/镜像/诊断

## 测试

57 个 Rust 单元测试 + TS 类型检查 + Vite 构建：

```bash
cd src-tauri && cargo test          # 57 passed
pnpm exec tsc --noEmit              # 0 errors
pnpm exec vite build                # 222KB JS / 26KB CSS
```

## 路线图

- ✅ **Slice 0** — 行走骨架（Node 检测→安装→日志→验证）
- ✅ **Slice 1** — 仪表盘完整化（11 工具 + 参数化配方 + PATH 持久化）
- ✅ **Slice 2** — 预设套件（5 内置 + 一键应用）
- ✅ **Slice 3** — WSL 一键配置（检测/启用/装 Ubuntu + 开发环境）
- ✅ **Slice 4** — 镜像/代理（npm + pip + 国内加速总开关）
- ✅ **Slice 5** — 故障诊断（11 工具规则 + 修复建议）
- ⏭ **下一刀候选**：环境快照与迁移、团队共享配置、Shell 美化、VSCode 插件包

## 已知限制

- 安装后必须重开终端，新工具才能在 PATH 里（Windows 注册表 vs 进程 env 限制）
- WSL 启用一步必须 admin（系统功能限制，UI 已说明）
- macOS / Linux 配方已写但仅在 Windows 上端到端验证
- Ollama / VSCode 不在当前 11 工具内（如需可补 recipe）

## License

待定（将采用开源协议）。
