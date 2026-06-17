# Flint Slice 1 端到端验证清单

> 配套实现：[`2026-06-17-flint-slice-1-design.md`](./2026-06-17-flint-slice-1-design.md)
> 状态：所有自动化测试通过（34/34）。本清单用于在 UI 上手动验证完整流程。

## 准备

1. 关掉现有 Tauri dev 会话（如果还在跑）：
   ```powershell
   Get-Process flint | Stop-Process -Force
   ```
2. 重新拉起：
   ```powershell
   pnpm tauri dev
   ```
3. 窗口打开后，仪表盘应展示 8 个工具卡片，分两组：
   - 运行时与基础工具：Node.js / Bun / Python / Git / GitHub CLI
   - AI 编程工具：Claude Code / OpenCode / Codex CLI
4. 已装工具显示绿点 + 版本号；未装的显示红点 + "未安装"

## 逐个安装验证

按下面顺序点。每装一个都看：
- 日志面板实时滚动（不是装完才一次性出）
- 进度条 0% → 100% 平滑推进
- 装完"重新检测"后卡片翻绿、显示版本

| # | 工具 | 验证点 | 预期 |
|---|---|---|---|
| 1 | **Bun** | 点安装 | 看到 `irm bun.sh/install.ps1 \| iex` 跑；约 5-15s；"重新检测"看到 `bun 1.x.x` |
| 2 | **Python** | 切下拉到 `3.13.0` | 看到 5 步日志：建目录 → 下载 zip → 解压 → 启 site → 装 pip；"重新检测"看到 `Python 3.13.0` |
| 3 | **Git** | 点安装 | 3 步日志（建目录 → 下载 mingit.zip → 解压）；装完开新终端 `git --version` 验证 |
| 4 | **GitHub CLI** | 点安装 | 3 步日志；装完开新终端 `gh --version` 验证 |
| 5 | **Claude Code** | 点安装 | `irm https://claude.ai/install.ps1 \| iex`；装完开新终端 `claude --version` |
| 6 | **OpenCode** | 点安装 | `irm https://opencode.ai/install.ps1 \| iex`；装完开新终端 `opencode --version` |
| 7 | **Codex CLI** | **先装 Node** 再点 Codex | `npm install -g @openai/codex`；装完开新终端 `codex --version` |
| 8 | **Node** | 切下拉到 `Latest` | `fnm install latest` + `fnm default latest`；装完开新终端 `node -v` 看 Latest 而非 LTS |

## 边界条件

- [ ] 装 Python 时切到 `3.13.0`，装完"重新检测"显示 `Python 3.13.0`，**不是**默认的 3.12.7
- [ ] 装任一工具中途，切到其他工具卡片——所有安装按钮**应被禁用**，无任何按钮可点
- [ ] 同一工具**装两次**（再次点安装按钮），日志会显示"已在用户 PATH 中"（幂等）
- [ ] 装完一个后，**关掉** Flint 窗口，**开新终端**，`where python`（或对应命令）能解析到
- [ ] 装 GitHub CLI 完成后日志有 `[ok] 已加入用户 PATH：...` 或 `[skip] 已在用户 PATH 中：...`

## 异常路径（可选）

- [ ] 断网时点 Bun 安装：日志显示 PowerShell 报错，`Done` 事件 `ok: false`，仪表盘卡片不变
- [ ] 装 Python 时把 `3.10.15` 改成一个不存在的版本号 `9.9.9` 改 TOML 后重装：占位符替换正常，下载 404，安装失败
- [ ] `resources/recipes/_test.toml` 里写个故意坏掉的 TOML，重启应用：日志不 panic，"重新检测" 仍工作

## 跨平台验证（P1 — 不阻塞 Slice 1 验收）

- [ ] macOS：`pnpm tauri dev` 起来，仪表盘同 8 工具；macOS 配方走 `brew install ...`（注意 brew 必须先装）
- [ ] Linux：同理

## 已知限制（不在 Slice 1 范围）

- 当前进程 PATH 不会自动刷新；装完需新开终端才能用新工具——日志里会显式提示
- Codex CLI 依赖 Node；前端**不**自动检测依赖（如果没装 Node 就点 Codex，npm 会报 command not found）
- macOS / Linux 配方只写了 P1 步骤，未在非 Windows 平台实跑过

## 通过标准

- [ ] 8 个工具中至少 5 个能成功安装（Node / Bun / Claude Code / OpenCode / Git 是最稳的；Python / GitHub CLI / Codex 取决于网络）
- [ ] 装完开新终端能跑对应命令
- [ ] 进度条 / 日志 / 按钮禁用行为符合预期
- [ ] 没有 panic / 崩溃 / 状态错乱

## 验证完成后

```bash
git log --oneline -20  # 确认 Slice 1 全部 commit 入仓
pnpm exec tsc --noEmit  # TS 0 错
cd src-tauri && cargo test  # 34/34
```
