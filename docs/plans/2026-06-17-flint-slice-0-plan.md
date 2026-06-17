# Flint Slice 0 — 行走骨架（Walking Skeleton）实施计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 用一个工具（Node.js via fnm）在 Windows 上跑通"检测 → 安装 → 实时日志 → 验证 → 仪表盘翻转"完整链路，证明 Tauri IPC / 进程调度 / 日志流 / 验证四件事可行。

**Architecture:** Tauri v2 桌面壳；Rust 后端含 Detector（检 fnm/Node）、Executor（async spawn + 实时日志经 Tauri Channel）、Recipe 加载器（TOML）；React 前端最小仪表盘 + 日志面板。所有安装零提权（用户级）。

**Tech Stack:** Tauri v2, React + TypeScript + Vite, Tailwind CSS, Rust（tokio, serde, toml, which）, pnpm, Cargo.

**配套设计：** [`2026-06-17-flint-design.md`](./2026-06-17-flint-design.md)

---

## 前置说明

- **平台**：Windows 10 1809+ / 11。本计划命令为 PowerShell/bash 通用（git bash 环境）。
- **TDD 边界**：纯逻辑（版本解析、配方解析、检测结果聚合）严格 TDD；进程 spawn / 前端渲染用集成测试 + 手动验证。
- **标记 `[VERIFY]`** 的步骤表示该命令/API 在 2026 可能有变动，执行时需用官方来源确认后再跑（不靠记忆）。
- **每个 Task 末尾都要 commit**（frequent commits）。

---

## Task 1: 工程脚手架

**Files:**
- Create: 整个 Tauri 工程根目录结构（`src/`, `src-tauri/`, `package.json`, …）
- Create: `src-tauri/tauri.conf.json`（模板生成后微调）

**Step 1: 确认前置工具**

Run: `node -v ; pnpm -v ; rustc --version ; cargo --version`
Expected: 四个都有版本号。若 pnpm 缺失：`npm i -g pnpm`。若 Rust 缺失：装 rustup（`https://rustup.rs`，用户级，无需管理员）。

**Step 2: `[VERIFY]` 脚手架**

[VERIFY] 用官方脚手架创建 Tauri v2 + React + TS 工程。先查当前推荐命令：
Run: `pnpm create tauri-app --help`
预期可用形态（确认后执行）：
```bash
pnpm create tauri-app . \
  --template react-ts \
  --manager pnpm \
  --identifier com.flint.app
```
> 若当前目录非空（已有 `docs/`、产品文档），脚手架可能拒绝。可先在临时目录生成再合并，或用 `--force`（确认不覆盖 `docs/`）。

**Step 3: 安装依赖 + 加 Tailwind**

```bash
pnpm install
pnpm add -D tailwindcss postcss autoprefixer
pnpm dlx tailwindcss init -p
```

配置 `tailwind.config.js` 的 `content` 指向 `./index.html` 和 `./src/**/*.{ts,tsx}`；在 `src/index.css` 顶部加：
```css
@tailwind base;
@tailwind components;
@tailwind utilities;
```

**Step 4: 验证 dev 能起来（手动）**

Run: `pnpm tauri dev`
Expected: 弹出桌面窗口，显示模板默认页。Ctrl-C 退出。

**Step 5: Commit**

```bash
git add -A
git commit -m "chore: scaffold Tauri v2 + React + TS + Tailwind"
```

---

## Task 2: Rust 依赖与错误类型

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/error.rs`

**Step 1: 加依赖**

`src-tauri/Cargo.toml` 的 `[dependencies]`：
```toml
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
tokio = { version = "1", features = ["process", "io-util", "rt-multi-thread", "macros", "time"] }
which = "6"
regex = "1"
thiserror = "1"
log = "0.4"
```

**Step 2: 写错误类型**

`src-tauri/src/error.rs`：
```rust
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FlintError {
    #[error("command not found: {0}")]
    NotFound(String),
    #[error("command failed ({code}): {stderr}")]
    CommandFailed { code: i32, stderr: String },
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("{0}")]
    Other(String),
}

// 序列化给前端，避免泄露非 Send 类型
#[derive(Debug, Serialize, Clone)]
pub struct ErrorPayload {
    pub kind: &'static str,
    pub message: String,
}

impl From<&FlintError> for ErrorPayload {
    fn from(e: &FlintError) -> Self {
        let kind = match e {
            FlintError::NotFound(_) => "not_found",
            FlintError::CommandFailed { .. } => "command_failed",
            FlintError::Io(_) => "io",
            FlintError::Parse(_) => "parse",
            FlintError::Other(_) => "other",
        };
        ErrorPayload { kind, message: e.to_string() }
    }
}

pub type Result<T> = std::result::Result<T, FlintError>;
```

**Step 3: 验证编译**

Run: `cargo build --manifest-path src-tauri/Cargo.toml`
Expected: 编译通过（可能有 unused warning，无妨）。

**Step 4: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/error.rs
git commit -m "feat(core): add rust deps and error type"
```

> Cargo.lock 应提交（Flint 是应用，非库）。

---

## Task 3: 版本解析（TDD · 纯逻辑）

**Files:**
- Create: `src-tauri/src/version.rs`

**Step 1: 写失败测试**

在 `src-tauri/src/version.rs` 底部：
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_node_version_output() {
        assert_eq!(parse_version("v20.11.0\n").unwrap().to_string(), "20.11.0");
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse_version("not a version").is_err());
    }

    #[test]
    fn compares_versions() {
        let a = parse_version("v18.0.0").unwrap();
        let b = parse_version("v20.11.0").unwrap();
        assert!(a < b);
    }
}
```

**Step 2: 跑测试确认失败**

Run: `cargo test --manifest-path src-tauri/Cargo.toml version`
Expected: 编译失败 / 测试失败（`parse_version` 未定义）。

**Step 3: 最小实现**

```rust
use std::cmp::Ordering;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Version(pub u32, pub u32, pub u32);

impl Version {
    pub fn major(&self) -> u32 { self.0 }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.0, self.1, self.2)
    }
}

impl Ord for Version {
    fn cmp(&self, o: &Self) -> Ordering {
        self.0.cmp(&o.0)
            .then(self.1.cmp(&o.1))
            .then(self.2.cmp(&o.2))
    }
}
impl PartialOrd for Version { fn partial_cmp(&self, o: &Self) -> Option<Ordering> { Some(self.cmp(o)) } }

/// 从 `node -v` / `v20.11.0` 这类输出解析版本。
pub fn parse_version(s: &str) -> Result<Version, String> {
    let s = s.trim().trim_start_matches('v');
    let mut parts = s.split('.');
    let maj = parts.next().and_then(|p| p.parse().ok());
    let min = parts.next().and_then(|p| p.parse().ok());
    let pat = parts.next().and_then(|p| p.parse().ok());
    match (maj, min, pat) {
        (Some(a), Some(b), Some(c)) => Ok(Version(a, b, c)),
        _ => Err(format!("invalid version: {s}")),
    }
}
```

**Step 4: 跑测试确认通过**

Run: `cargo test --manifest-path src-tauri/Cargo.toml version`
Expected: 3 passed.

**Step 5: Commit**

```bash
git add src-tauri/src/version.rs
git commit -m "feat(core): version parsing with tests"
```

---

## Task 4: Recipe 数据结构与加载（TDD）

**Files:**
- Create: `src-tauri/src/recipe.rs`
- Create: `src-tauri/resources/recipes/node.toml`

**Step 1: 写配方文件**

`src-tauri/resources/recipes/node.toml`：
```toml
[meta]
id = "node"
display_name = "Node.js"
category = "runtime"

# 通过 fnm 安装（用户级，零提权）
[install.windows]
requires_elevation = false
steps = [
  { cmd = "fnm", args = ["install", "--lts"] },
  { cmd = "fnm", args = ["default", "lts-latest"] },   # 占位，Task 7 校准 fnm PATH 写法
]

[detect.windows]
cmd = "node"
args = ["--version"]
version_regex = "v(?P<v>\\d+\\.\\d+\\.\\d+)"

[verify.windows]
cmd = "node"
args = ["--version"]
```

> `[VERIFY]` Task 7 会校准 fnm 的 PATH 持久化写法（`fnm default` / `fnm env`）。此处先放占位。

**Step 2: 写失败测试**

`src-tauri/src/recipe.rs` 底部：
```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_node_recipe() {
        let toml = r#"
[meta]
id = "node"
display_name = "Node.js"
category = "runtime"
[install.windows]
requires_elevation = false
steps = [ { cmd = "fnm", args = ["install", "--lts"] } ]
"#;
        let r: Recipe = toml::from_str(toml).unwrap();
        assert_eq!(r.meta.id, "node");
        let win = r.install.get("windows").unwrap();
        assert_eq!(win.steps.len(), 1);
        assert_eq!(win.steps[0].cmd, "fnm");
        assert!(!win.requires_elevation);
    }
}
```

**Step 3: 跑确认失败**

Run: `cargo test --manifest-path src-tauri/Cargo.toml recipe`
Expected: 失败（`Recipe` 未定义）。

**Step 4: 实现 struct**

```rust
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct Recipe {
    pub meta: Meta,
    #[serde(default)]
    pub install: HashMap<String, PlatformInstall>,
    #[serde(default)]
    pub detect: HashMap<String, PlatformCommand>,
    #[serde(default)]
    pub verify: HashMap<String, PlatformCommand>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Meta {
    pub id: String,
    pub display_name: String,
    #[serde(default)]
    pub category: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PlatformInstall {
    #[serde(default)]
    pub requires_elevation: bool,
    pub steps: Vec<Step>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Step {
    pub cmd: String,
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PlatformCommand {
    pub cmd: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub version_regex: Option<String>,
}

impl Recipe {
    pub fn load(id: &str) -> Result<Self, String> {
        let path = std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(format!("resources/recipes/{id}.toml"));
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("read {}: {e}", path.display()))?;
        toml::from_str(&text).map_err(|e| format!("parse {}: {e}", path.display()))
    }
}
```

**Step 5: 跑确认通过**

Run: `cargo test --manifest-path src-tauri/Cargo.toml recipe`
Expected: 1 passed.

**Step 6: Commit**

```bash
git add src-tauri/src/recipe.rs src-tauri/resources/recipes/node.toml
git commit -m "feat(core): recipe struct + node.toml with tests"
```

---

## Task 5: Executor —— async spawn + 实时日志（集成测试）

**Files:**
- Create: `src-tauri/src/executor.rs`

**Step 1: 写集成测试（跑真实命令）**

`src-tauri/src/executor.rs` 底部：
```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn streams_echo_output() {
        let mut rx = run(&["cmd".to_string(), "/C".to_string(), "echo hello".to_string()], None).await.unwrap();
        let mut got = String::new();
        while let Ok(ev) = rx.recv().await {
            if let StreamEvent::Line(l) = ev { got.push_str(&l); }
        }
        assert!(got.contains("hello"));
    }
}
```

**Step 2: 跑确认失败**

Run: `cargo test --manifest-path src-tauri/Cargo.toml executor`
Expected: 失败（`run`/`StreamEvent` 未定义）。

**Step 3: 实现**

```rust
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;
use crate::error::Result;

#[derive(Debug, Clone)]
pub enum StreamEvent {
    Line(String),          // stdout 或 stderr 的一行
    Exit(i32),             // 退出码
}

/// 以当前用户权限运行命令，按行推送 stdout/stderr。
/// `on_cancel` 为 None 表示不可取消（Slice 0 足够）。
pub async fn run(
    argv: &[String],
    _on_cancel: Option<()>,
) -> Result<mpsc::Receiver<StreamEvent>> {
    // 注意：argv[0] 经 Executor 内部 resolve（which）；这里直接用 Command
    let (tx, rx) = mpsc::channel::<StreamEvent>(64);
    let mut cmd = Command::new(&argv[0]);
    cmd.args(&argv[1..]);
    cmd.stdout(std::process::Stdio::piped())
       .stderr(std::process::Stdio::piped())
       .kill_on_drop(true);

    let mut child = cmd.spawn()?;
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    tokio::spawn(async move {
        let mut out = BufReader::new(stdout).lines();
        let mut err = BufReader::new(stderr).lines();
        loop {
            tokio::select! {
                Ok(Some(line)) = out.next_line() => {
                    if tx.send(StreamEvent::Line(format!("[out] {line}"))).await.is_err() { break; }
                }
                Ok(Some(line)) = err.next_line() => {
                    if tx.send(StreamEvent::Line(format!("[err] {line}"))).await.is_err() { break; }
                }
                else => break,
            }
        }
        let status = child.wait().await;
        let code = status.ok().and_then(|s| s.code()).unwrap_or(-1);
        let _ = tx.send(StreamEvent::Exit(code)).await;
    });

    Ok(rx)
}
```

**Step 4: 跑确认通过**

Run: `cargo test --manifest-path src-tauri/Cargo.toml executor`
Expected: 1 passed。

**Step 5: Commit**

```bash
git add src-tauri/src/executor.rs
git commit -m "feat(core): async executor with streamed stdout/stderr"
```

---

## Task 6: Detector —— 检测 fnm / Node（集成测试）

**Files:**
- Create: `src-tauri/src/detector.rs`

**Step 1: 写测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detects_node_when_present() {
        // 开发机通常有 node；有则解析出版本
        if let Ok(Some(v)) = detect_version("node", &["--version"]) {
            assert!(v.starts_with(|c: char| c.is_ascii_digit()));
        }
    }
}
```

**Step 2: 跑确认失败**

Run: `cargo test --manifest-path src-tauri/Cargo.toml detector`
Expected: 失败。

**Step 3: 实现**

```rust
use crate::error::Result;
use std::process::Command;

/// 返回某工具是否存在。
pub fn is_installed(cmd: &str) -> bool {
    which::which(cmd).is_ok()
}

/// 运行 `cmd args`，取 stdout，正则抽版本。None 表示未装/解析不到。
pub fn detect_version(cmd: &str, args: &[String]) -> Result<Option<String>> {
    if !is_installed(cmd) { return Ok(None); }
    let out = Command::new(cmd).args(args).output()?;
    let text = String::from_utf8_lossy(&out.stdout);
    Ok(crate::version::parse_version(&text).ok().map(|v| v.to_string()))
}

/// Node 检测的便捷聚合。
pub fn detect_node() -> Result<Option<String>> {
    detect_version("node", &["--version".into()])
}
```

**Step 4: 跑确认通过**

Run: `cargo test --manifest-path src-tauri/Cargo.toml detector`
Expected: passed。

**Step 5: Commit**

```bash
git add src-tauri/src/detector.rs
git commit -m "feat(core): detector for fnm/node"
```

---

## Task 7: IPC 命令 + Tauri Channel 接线

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Create: `src-tauri/src/ipc.rs`

**Step 1: `[VERIFY]` Channel API 形态**

[VERIFY] 确认 Tauri v2 当前 `Channel<T>` 用法（官方 `tauri::ipc::Channel`）。下方为标准形态。

`src-tauri/src/ipc.rs`：
```rust
use serde::Serialize;
use tauri::ipc::Channel;
use crate::{detector, error::FlintError, executor::{self, StreamEvent}, recipe::Recipe};

#[derive(Serialize, Clone)]
#[serde(tag = "type")]
pub enum InstallEvent {
    Log { line: String },
    Progress { pct: u8 },
    Done { ok: bool, version: Option<String>, error: Option<String> },
}

#[tauri::command]
pub async fn detect_node() -> Result<Option<String>, String> {
    detector::detect_node().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn install_node(on_event: Channel<InstallEvent>) -> Result<(), String> {
    let recipe = Recipe::load("node").map_err(|e| e.to_string())?;
    let win = recipe.install.get("windows").ok_or("no windows install")?;

    let total = win.steps.len().max(1);
    for (i, step) in win.steps.iter().enumerate() {
        let argv: Vec<String> = std::iter::once(step.cmd.clone())
            .chain(step.args.iter().cloned()).collect();
        let mut rx = executor::run(&argv, None).await.map_err(|e| e.to_string())?;
        let mut exit_code: i32 = 0;
        while let Some(ev) = rx.recv().await {
            match ev {
                StreamEvent::Line(l) => { let _ = on_event.send(InstallEvent::Log { line: l }); }
                StreamEvent::Exit(c) => { exit_code = c; }
            }
        }
        if exit_code != 0 {
            let _ = on_event.send(InstallEvent::Done { ok: false, version: None,
                error: Some(format!("step {} exited {exit_code}", step.cmd)) });
            return Err(format!("step {} failed", step.cmd));
        }
        let _ = on_event.send(InstallEvent::Progress { pct: ((i + 1) * 100 / total) as u8 });
    }

    // 验证
    let version = detector::detect_node().ok().flatten();
    let _ = on_event.send(InstallEvent::Done { ok: version.is_some(), version: version.clone(), error: None });
    // 处理"装了但 PATH 没刷新"的坑：提示前端
    if version.is_none() {
        let _ = on_event.send(InstallEvent::Log { line: "[!] node 仍未在 PATH 上，可能需重启终端".into() });
    }
    Ok(())
}

// 抑制未使用警告（FlintError 后续扩展用）
#[allow(unused)]
fn _unused(e: FlintError) -> String { e.to_string() }
```

**Step 2: 在 lib.rs 注册**

`src-tauri/src/lib.rs`（模板已有 `run()`，在其中注册命令）：
```rust
mod error; mod version; mod recipe; mod executor; mod detector; mod ipc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![ipc::detect_node, ipc::install_node])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Step 3: `[VERIFY]` 校准 fnm PATH 持久化**

[VERIFY] 在 Windows 上确认 fnm 使 `node` 持久可用的官方写法（2026）：
- 跑 `fnm install --lts`、`fnm default lts-latest`
- 持久化：把 `fnm env --shell powershell` 输出的 PATH 条目写入用户 PATH（HKCU\Environment），或按 fnm README 推荐的 profile 片段。
- 据此修正 `node.toml` 的 `install.windows.steps`（可能多一步调 ConfigManager 写 PATH）。

**Step 4: 验证编译**

Run: `cargo build --manifest-path src-tauri/Cargo.toml`
Expected: 通过。

**Step 5: Commit**

```bash
git add src-tauri/src/ipc.rs src-tauri/src/lib.rs src-tauri/resources/recipes/node.toml
git commit -m "feat(ipc): detect_node + install_node with Channel streaming"
```

---

## Task 8: 前端最小仪表盘 + 日志面板

**Files:**
- Create: `src/App.tsx`（替换模板）
- Create: `src/main.tsx`（模板已有，保留）

**Step 1: `[VERIFY]` 前端 Channel API**

[VERIFY] 确认 `@tauri-apps/api` 当前导出的 `Channel` 用法。标准形态：
```ts
import { invoke, Channel } from "@tauri-apps/api/core";
```

**Step 2: 写组件**

`src/App.tsx`：
```tsx
import { useState } from "react";
import { invoke, Channel } from "@tauri-apps/api/core";

type InstallEvent =
  | { type: "Log"; line: string }
  | { type: "Progress"; pct: number }
  | { type: "Done"; ok: boolean; version: string | null; error: string | null };

export default function App() {
  const [node, setNode] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [logs, setLogs] = useState<string[]>([]);

  async function detect() {
    const v = await invoke<string | null>("detect_node");
    setNode(v);
  }

  async function install() {
    setBusy(true); setLogs([]);
    const ch = new Channel<InstallEvent>();
    ch.onmessage = (e) => {
      if (e.type === "Log") setLogs((l) => [...l, e.line]);
      if (e.type === "Done") {
        setBusy(false);
        if (e.ok) setNode(e.version);
        else setLogs((l) => [...l, `✗ ${e.error ?? "failed"}`]);
      }
    };
    try {
      await invoke("install_node", { onEvent: ch });
    } catch (err) {
      setBusy(false);
      setLogs((l) => [...l, `✗ ${String(err)}`]);
    }
  }

  return (
    <div className="min-h-screen bg-slate-950 text-slate-100 p-8 font-sans">
      <h1 className="text-2xl font-bold mb-6">Flint</h1>
      <div className="flex items-center gap-4 mb-6">
        <span className={node ? "text-emerald-400" : "text-rose-400"}>
          Node.js: {node ? `✓ ${node}` : "✗ 未安装"}
        </span>
        <button onClick={detect} className="px-3 py-1 rounded bg-slate-700">检测</button>
        <button onClick={install} disabled={busy || !!node}
          className="px-3 py-1 rounded bg-indigo-600 disabled:opacity-40">
          {busy ? "安装中…" : "安装 Node LTS"}
        </button>
      </div>
      <pre className="bg-black/40 p-4 rounded text-xs h-80 overflow-auto whitespace-pre-wrap">
        {logs.join("\n")}
      </pre>
    </div>
  );
}
```

**Step 3: 手动验证（核心验收点）**

Run: `pnpm tauri dev`
1. 点"检测" → 显示 Node 状态（有则版本，无则未安装）。
2. 若已装 Node，先 `fnm uninstall` 或换机器造一个"未安装"态。
3. 点"安装 Node LTS" → 日志区实时滚动 `[out]/[err]` 行；完成后 Node 状态翻绿、显示版本。
4. **打开一个新 PowerShell**，跑 `node -v` → 应输出版本（证明 PATH 持久化生效）。

**Step 4: Commit**

```bash
git add src/App.tsx
git commit -m "feat(ui): minimal dashboard + live install log panel"
git tag slice-0
```

---

## 验收标准（Definition of Done · Slice 0）

- [ ] `pnpm tauri dev` 起得来，UI 可交互。
- [ ] 检测能正确报 Node 已装/未装。
- [ ] 安装链路全程实时日志可见，无阻塞 UI。
- [ ] 安装完成后**新终端**里 `node -v` 可用（PATH 持久化）。
- [ ] 全程零管理员提示（用户级安装）。
- [ ] `cargo test` 全绿。
- [ ] `slice-0` tag 已打。

## Slice 0 不做（YAGNI）

- 其他运行时/AI 工具（Slice 1/2）
- Preset、镜像配置、Shell 美化（Slice 2/3）
- WSL（Slice 3）
- macOS/Linux 实装（仅留 stub）
- 取消安装、超时（Executor 预留接口即可）
- 代码签名（分发前再做）
