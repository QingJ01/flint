# Flint Slice 1 — 8 个工具一键安装 + 参数化配方

> 配套实施计划：[`2026-06-17-flint-slice-1-plan.md`](./2026-06-17-flint-slice-1-plan.md)
> 前置：Slice 0（walking skeleton：detect_node + install_node + 流式日志）

---

## 目标

把 Slice 0 的"只装 Node"扩成"装全部 8 个工具"：

- 仪表盘上 8 张工具卡片都有一键安装按钮
- Python / Node 走下拉选版本（其他工具无版本概念）
- 全程零提权、零 UAC 弹窗（用便携版 zip / 原生安装器，**不**走 winget）
- 安装后可执行（自动处理 PATH 持久化）

## 范围

| 工具 | 安装方式 | 是否有版本下拉 | 备注 |
|---|---|---|---|
| Node.js | fnm（已在 Slice 0）| ✅ LTS / Latest | 改用 `install_tool` 通用入口 |
| Bun | `irm bun.sh/install.ps1 \| iex` | — | 原生安装器 |
| Python | python.org embeddable zip + get-pip.py | ✅ 3.10 / 3.11 / 3.12 / 3.13 | 4 个常见版本 |
| Git | MinGit portable zip | — | 解压即用 |
| GitHub CLI | cli/cli 官方 windows_amd64.zip | — | 解压即用 |
| Claude Code | `irm https://claude.ai/install.ps1 \| iex` | — | 原生安装器 |
| OpenCode | `irm https://opencode.ai/install.ps1 \| iex` | — | 原生安装器 |
| Codex CLI | `npm i -g @openai/codex` | — | 依赖 Node |

## 关键设计决策

### 1. 零提权优先

- 不用 winget / Homebrew 等系统级包管理器
- 优先用：① 官方便携版 zip ② 官方原生安装器（PowerShell / bash 一行命令）
- 全部 `requires_elevation = false`
- Windows PATH 持久化走用户级（`[Environment]::SetEnvironmentVariable('PATH', ..., 'User')`），写到 PowerShell profile

### 2. 配方（Recipe）参数化

每个 recipe 可声明 `[parameters.<name>]` 节，type 仅 MVP 仅 `select`：

```toml
[parameters.python_version]
type = "select"
label = "Python 版本"
default = "3.12.7"
options = [
  { value = "3.13.0", label = "3.13 (最新)" },
  { value = "3.12.7", label = "3.12 (推荐)" },
  ...
]
```

step 的 `cmd` / `args` 里写 `{python_version}` 占位，运行时被替换。无参数的工具不写 `[parameters.*]`，行为与 Slice 0 一致。

### 3. 后端命令签名

```rust
#[tauri::command]
async fn install_tool(
    id: String,
    params: serde_json::Map<String, serde_json::Value>,
    on_event: Channel<InstallEvent>,
) -> Result<(), String>

#[tauri::command]
async fn list_installable_tools() -> Result<Vec<ToolMeta>, String>
```

`ToolMeta`：
```rust
pub struct ToolMeta {
    pub id: String,
    pub display_name: String,
    pub category: ToolCategory,
    pub requires_elevation: bool,
    pub parameters: Vec<ParameterDef>,
}
```

### 4. Recipe 加载策略

- 保留 `Recipe::load(id)`（panic-on-missing），改用 `Recipe::load_optional(id) -> Option<Self>`
- 新增 `Recipe::list_available() -> Vec<Meta>`：扫 `resources/recipes/*.toml` 返回所有元数据
- 找不到 recipe 的工具在 `list_installable_tools` 里不返回（detector 检得到但没有安装方式 = 跳过）

### 5. PATH 持久化

`config::add_to_user_path(dir: &Path) -> Result<bool>`：

- 读 user PATH
- `dir` 不在里面 → 追加 → 写回
- 幂等
- **仅 Windows**：用 `Environment::set_environment_variable` 的 user scope；macOS / Linux 后续 Slice 再做

## 数据流

```
前端                        后端
─────────────────          ─────────────────
启动 App                   
  │                         
  ├─ invoke list_installable_tools() 
  │                         → 扫 resources/recipes/*.toml
  │                         → 解析 → 返回 Vec<ToolMeta>
  │                         
  ├─ invoke detect_environment() 
  │                         → 静态 catalog 跑 --version
  │                         
  ├─ 用户选 Python 3.12.7，点安装
  │                         
  └─ invoke install_tool(   → 查 recipe → 替换 {python_version}
      "python",             → 跑 step 1 (下载 zip) 
      {python_version:      → 跑 step 2 (解压)
       "3.12.7"},           → 跑 step 3 (PATH)
      onEvent)              → 跑 step 4 (get-pip)
                            → config::add_to_user_path
                            → 推 Done {ok: true, version: ...}
  │
  └─ invoke detect_environment() （重新检测）
                            → 这次 python 报"已装 3.12.7"
```

## 错误处理

- `install_tool("unknown_id", ...)` → `Err("unknown tool id: ...")`
- step 退出码非 0 → 推 `Done {ok: false, error: "step X exited Y"}` 并中断
- 配方 TOML 解析失败 → `list_installable_tools` 跳过并 log（不 panic）
- 下载失败（PowerShell `Invoke-WebRequest` 非 0）→ 同上 step 失败处理
- 端口占用 / 防病毒拦截 → 留给 Step 内的错误信息透传，文档 §6 故障诊断后续 slice

## 切片外（不做）

- Preset 编排（Vibecoder 全家桶等组合一键）
- WSL 一键配置
- 镜像 / 代理 / 国内加速
- 快照 / 迁移
- Shell 美化
- 故障诊断
- VSCode / Cursor 插件包
- 团队共享配置

## 验证清单

- [ ] TDD：recipe 参数化解析、占位符替换、`list_available` / `load_optional`
- [ ] TDD：`add_to_user_path` 幂等
- [ ] `cargo test` 全绿
- [ ] `pnpm tsc --noEmit` 无错
- [ ] 手动跑 `pnpm tauri dev`，8 个工具每个点一遍安装按钮：
  - 日志实时滚动、进度 0→100
  - 完成后"重新检测"看到 installed + version
  - PATH 写入后**新开**终端可执行
- [ ] Node / Python 切换下拉选项后 install 行为正确
- [ ] 正在装 A 时点 B → B 按钮禁用
