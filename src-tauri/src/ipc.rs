use crate::diagnose::{self, DiagnosticReport, Finding};
use crate::preset::{self, Preset, PresetMeta};
use crate::recipe::ParameterOption;
use crate::wsl::{self, WslStatus};
use crate::{
    config,
    detector::{self, ToolCategory, ToolStatus},
    executor::{self, StreamEvent},
    recipe::{self, Recipe},
    snapshot, versions,
};
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use tauri::ipc::Channel;

/// Streamed events to the frontend. The `#[serde(tag = "type")]` makes
/// `event.type` the discriminator on the JS side.
#[derive(Serialize, Clone)]
#[serde(tag = "type")]
pub enum InstallEvent {
    Log {
        line: String,
    },
    Progress {
        pct: u8,
    },
    Done {
        ok: bool,
        version: Option<String>,
        error: Option<String>,
    },
    /// Restore-only: marks the start of a logical section (e.g. "安装缺失工具",
    /// "应用镜像") so the snapshot-import UI can group its log output.
    RestoreSection {
        name: String,
    },
}

#[derive(Serialize, Clone)]
pub struct ToolMeta {
    pub id: String,
    pub display_name: String,
    pub category: ToolCategory,
    pub requires_elevation: bool,
    pub parameters: Vec<ToolParameterMeta>,
}

#[derive(Serialize, Clone)]
pub struct ToolParameterMeta {
    pub key: String,
    pub label: String,
    pub default: Option<String>,
    pub options: Vec<ParameterOption>,
}

#[tauri::command]
pub async fn detect_environment() -> Result<Vec<ToolStatus>, String> {
    detector::detect_environment().map_err(|e| e.to_string())
}

/// Enumerate every preset at `resources/presets/*.toml`.
#[tauri::command]
pub async fn list_presets() -> Result<Vec<PresetMeta>, String> {
    Ok(preset::Preset::list_available())
}

/// Return the full preset (including the tool id list and default params)
/// for a given preset id. The frontend uses this to drive batch install.
#[tauri::command]
pub async fn get_preset(id: String) -> Result<Preset, String> {
    Preset::load(&id)
}

/// Probe `wsl --status` and return a structured snapshot.
#[tauri::command]
pub async fn wsl_status() -> Result<WslStatus, String> {
    wsl::detect_wsl().map_err(|e| e.to_string())
}

/// Read the current npm + pip registry selections.
#[tauri::command]
pub async fn mirror_status() -> Result<MirrorStatus, String> {
    Ok(MirrorStatus {
        npm: config::current_npm_registry().map_err(|e| e.to_string())?,
        pip: config::current_pip_registry().map_err(|e| e.to_string())?,
    })
}

#[derive(Serialize, Clone)]
pub struct MirrorStatus {
    pub npm: Option<String>,
    pub pip: Option<String>,
}

/// Apply an npm registry by writing to `~/.npmrc`.
#[tauri::command]
pub async fn apply_npm_mirror(registry_url: String) -> Result<bool, String> {
    config::apply_npm_registry(&registry_url).map_err(|e| e.to_string())
}

/// Apply a pip index URL.
#[tauri::command]
pub async fn apply_pip_mirror(index_url: String) -> Result<bool, String> {
    config::apply_pip_registry(&index_url).map_err(|e| e.to_string())
}

/// "国内加速模式" master switch — apply the default CN mirrors for both
/// npm and pip in one call. Returns the list of (kind, changed) pairs so
/// the UI can summarize what happened.
#[tauri::command]
pub async fn apply_domestic_acceleration() -> Result<Vec<(String, bool)>, String> {
    let npm =
        config::apply_npm_registry("https://registry.npmmirror.com/").map_err(|e| e.to_string())?;
    let pip = config::apply_pip_registry("https://pypi.tuna.tsinghua.edu.cn/simple")
        .map_err(|e| e.to_string())?;
    Ok(vec![("npm".into(), npm), ("pip".into(), pip)])
}

/// Run the diagnostic checks for a given tool and return a structured
/// report. Never returns an error — internal rule failures are surfaced
/// as findings with `severity: error`.
#[tauri::command]
pub async fn diagnose_tool(tool_id: String) -> Result<DiagnosticReport, String> {
    diagnose::run_diagnostics(&tool_id)
        .await
        .map_err(|e| e.to_string())
}

/// Opt-in: validate the Anthropic API key by hitting the (free)
/// `GET /v1/models` endpoint. Never run automatically — only when the user
/// clicks the "验证 Key" button. Endpoint + headers verified against the
/// Anthropic API reference (x-api-key + anthropic-version: 2023-06-01).
#[tauri::command]
pub async fn verify_anthropic_key() -> Result<Finding, String> {
    Ok(diagnose::verify_anthropic_key().await)
}

/// Return the version options to show in a tool's dropdown. For `node` and
/// `python` we fetch the *real* available versions at request time (fnm /
/// endoflife.date); on any failure — or for any other tool — we fall back to
/// the recipe's static `[parameters.*_version]` options. Best-effort: never
/// errors, so the dropdown always has something. Lazy: the frontend calls
/// this on demand (dropdown focus), not on dashboard load.
#[tauri::command]
pub async fn list_tool_versions(tool_id: String) -> Result<Vec<ParameterOption>, String> {
    let dynamic = match tool_id.as_str() {
        "node" => versions::fetch_node_versions(12).await,
        "python" => versions::fetch_python_versions().await,
        _ => None,
    };
    Ok(dynamic.unwrap_or_else(|| recipe_static_version_options(&tool_id)))
}

/// Pull the static version options out of a recipe's first parameter that has
/// any (the `*_version` param). Empty vec if the recipe has none.
fn recipe_static_version_options(tool_id: &str) -> Vec<ParameterOption> {
    let Some(recipe) = Recipe::load_optional(tool_id) else {
        return Vec::new();
    };
    recipe
        .parameters
        .values()
        .find(|def| !def.options.is_empty())
        .map(|def| def.options.clone())
        .unwrap_or_default()
}

/// Build a snapshot of the current environment (for preview in the UI).
#[tauri::command]
pub async fn current_snapshot() -> Result<snapshot::Snapshot, String> {
    snapshot::build().map_err(|e| e.to_string())
}

/// Capture the current environment and write it to `path` as JSON. The
/// frontend obtains `path` from a native save dialog.
#[tauri::command]
pub async fn export_snapshot(path: String) -> Result<(), String> {
    let snap = snapshot::build().map_err(|e| e.to_string())?;
    snapshot::export_to(Path::new(&path), &snap).map_err(|e| e.to_string())
}

/// Load a snapshot from `path` and "smart restore" it: install every tool
/// the snapshot had installed but this machine lacks, then apply the npm/pip
/// mirrors. Never uninstalls and never touches PATH directly (install steps
/// do their own PATH additions). Streams progress over `on_event`.
#[tauri::command]
pub async fn import_snapshot(path: String, on_event: Channel<InstallEvent>) -> Result<(), String> {
    let snap = snapshot::load(Path::new(&path)).map_err(|e| e.to_string())?;

    // What's already here — so we skip tools that don't need reinstalling.
    let present: std::collections::HashSet<String> = detector::detect_environment()
        .map_err(|e| e.to_string())?
        .into_iter()
        .filter(|t| t.installed)
        .map(|t| t.id)
        .collect();

    // ---- Section 1: install missing tools ----
    let _ = on_event.send(InstallEvent::RestoreSection {
        name: "安装缺失工具".into(),
    });
    let wanted: Vec<&ToolStatus> = snap
        .tools
        .iter()
        .filter(|t| t.installed && !present.contains(&t.id))
        .collect();
    if wanted.is_empty() {
        let _ = on_event.send(InstallEvent::Log {
            line: "[ok] 快照里的工具本机都已安装，无需安装".into(),
        });
    }
    let total = wanted.len().max(1);
    let mut failures: Vec<String> = Vec::new();
    for (i, tool) in wanted.iter().enumerate() {
        let _ = on_event.send(InstallEvent::Log {
            line: format!("[restore] 安装 {} ({}/{})", tool.id, i + 1, wanted.len()),
        });
        // Use recipe defaults for params (snapshot doesn't record the exact
        // version params the user picked — restoring "latest"/default is the
        // pragmatic choice and matches what the dashboard one-click does).
        let params: HashMap<String, String> = HashMap::new();
        if let Err(e) = install_recipe(&tool.id, &params, &on_event).await {
            let _ = on_event.send(InstallEvent::Log {
                line: format!("[warn] {} 安装失败：{e}（继续其余）", tool.id),
            });
            failures.push(tool.id.clone());
        }
        let _ = on_event.send(InstallEvent::Progress {
            pct: ((i + 1) * 100 / total) as u8,
        });
    }

    // ---- Section 2: apply mirrors ----
    let _ = on_event.send(InstallEvent::RestoreSection {
        name: "应用镜像".into(),
    });
    if let Some(url) = &snap.npm_registry {
        match config::apply_npm_registry(url) {
            Ok(true) => {
                let _ = on_event.send(InstallEvent::Log {
                    line: format!("[ok] npm 源 → {url}"),
                });
            }
            Ok(false) => {
                let _ = on_event.send(InstallEvent::Log {
                    line: format!("[skip] npm 源已是 {url}"),
                });
            }
            Err(e) => {
                let _ = on_event.send(InstallEvent::Log {
                    line: format!("[warn] 设置 npm 源失败：{e}"),
                });
            }
        }
    }
    if let Some(url) = &snap.pip_registry {
        match config::apply_pip_registry(url) {
            Ok(true) => {
                let _ = on_event.send(InstallEvent::Log {
                    line: format!("[ok] pip 源 → {url}"),
                });
            }
            Ok(false) => {
                let _ = on_event.send(InstallEvent::Log {
                    line: format!("[skip] pip 源已是 {url}"),
                });
            }
            Err(e) => {
                let _ = on_event.send(InstallEvent::Log {
                    line: format!("[warn] 设置 pip 源失败：{e}"),
                });
            }
        }
    }

    let _ = on_event.send(InstallEvent::Done {
        ok: failures.is_empty(),
        version: None,
        error: if failures.is_empty() {
            None
        } else {
            Some(format!("以下工具未能安装：{}", failures.join(", ")))
        },
    });
    Ok(())
}

/// Trigger the Windows "enable WSL" flow. This is the one operation that
/// genuinely requires admin: it spawns `wsl --install` via PowerShell's
/// `Start-Process -Verb RunAs`, which prompts the user with a UAC dialog.
/// Returns immediately after kicking off the elevation; the caller should
/// poll `wsl_status` to confirm the feature is enabled.
#[tauri::command]
pub async fn wsl_enable(on_event: Channel<InstallEvent>) -> Result<(), String> {
    let script = r#"Start-Process wsl -Verb RunAs -ArgumentList "--install","--no-distribution","--no-launch""#;
    let argv = vec![
        "powershell".to_string(),
        "-NoProfile".to_string(),
        "-Command".to_string(),
        script.to_string(),
    ];
    let mut rx = executor::run(&argv, None)
        .await
        .map_err(|e| e.to_string())?;
    let mut exit_code: i32 = 0;
    while let Some(ev) = rx.recv().await {
        match ev {
            StreamEvent::Line(l) => {
                let _ = on_event.send(InstallEvent::Log { line: l });
            }
            StreamEvent::Exit(c) => exit_code = c,
        }
    }
    // Start-Process returns immediately; we don't strictly care about its
    // exit code. Surface a hint log instead.
    let _ = on_event.send(InstallEvent::Log {
        line: "[info] 若弹出 UAC 对话框请点「是」；操作完成后新开 PowerShell 运行 `wsl --status` 验证。".into(),
    });
    let _ = on_event.send(InstallEvent::Done {
        ok: exit_code == 0,
        version: None,
        error: None,
    });
    Ok(())
}

/// Install a baseline dev environment inside the default WSL Ubuntu
/// distro. Runs as root to skip the first-launch user setup wizard. Streams
/// progress via `on_event`.
#[tauri::command]
pub async fn wsl_install_dev_tools(on_event: Channel<InstallEvent>) -> Result<(), String> {
    // apt update + install baseline + install fnm + Node LTS + Bun + Python
    // + uv + Claude Code native installer.
    let script = r#"
set -e
echo "[wsl] apt update..."
sudo apt-get update -y
echo "[wsl] install baseline..."
sudo apt-get install -y git curl ca-certificates build-essential
echo "[wsl] install fnm + Node LTS..."
curl -fsSL https://fnm.vercel.app/install | bash -s -- --skip-shell
export PATH="$HOME/.local/share/fnm:$PATH"
eval "$(fnm env --use-on-cd --shell bash)"
fnm install --lts
fnm default lts-latest
echo "[wsl] install Bun..."
curl -fsSL https://bun.com/install | bash
echo "[wsl] install Python + pip..."
sudo apt-get install -y python3 python3-pip python3-venv
echo "[wsl] install uv..."
curl -LsSf https://astral.sh/uv/install.sh | sh
echo "[wsl] install Claude Code..."
curl -fsSL https://claude.ai/install.sh | sh
echo "[wsl] done"
"#;
    let argv = vec![
        "wsl".to_string(),
        "-u".to_string(),
        "root".to_string(),
        "-d".to_string(),
        "Ubuntu".to_string(),
        "--".to_string(),
        "bash".to_string(),
        "-c".to_string(),
        script.to_string(),
    ];
    let total = 1;
    let mut rx = executor::run(&argv, None)
        .await
        .map_err(|e| e.to_string())?;
    let mut exit_code: i32 = 0;
    while let Some(ev) = rx.recv().await {
        match ev {
            StreamEvent::Line(l) => {
                let _ = on_event.send(InstallEvent::Log { line: l });
            }
            StreamEvent::Exit(c) => exit_code = c,
        }
    }
    let _ = on_event.send(InstallEvent::Progress { pct: 100 });
    let _ = on_event.send(InstallEvent::Done {
        ok: exit_code == 0,
        version: None,
        error: if exit_code != 0 {
            Some(format!("wsl bash exited with {exit_code}"))
        } else {
            None
        },
    });
    Ok(())
}

/// Enumerate every recipe on disk as a `ToolMeta` for the frontend.
/// The current platform's `[install.<platform>]` is consulted for
/// `requires_elevation`; tools without an install entry for the current
/// platform are still listed (frontend may choose to grey them out).
#[tauri::command]
pub async fn list_installable_tools() -> Result<Vec<ToolMeta>, String> {
    let metas = Recipe::list_available();
    let platform = current_platform();
    let mut out = Vec::with_capacity(metas.len());
    for m in metas {
        let recipe = match Recipe::load_optional(&m.id) {
            Some(r) => r,
            None => continue,
        };
        let requires_elevation = recipe
            .install
            .get(platform)
            .map(|p| p.requires_elevation)
            .unwrap_or(false);
        let parameters = recipe
            .parameters
            .iter()
            .map(|(key, def)| ToolParameterMeta {
                key: key.clone(),
                label: def.label.clone(),
                default: def.default.clone(),
                options: def.options.clone(),
            })
            .collect();
        out.push(ToolMeta {
            id: m.id,
            display_name: m.display_name,
            category: parse_category(&m.category),
            requires_elevation,
            parameters,
        });
    }
    Ok(out)
}

/// Run the install steps for `id`, with `params` substituted into the
/// recipe's `{key}` placeholders. Streams logs / progress to `on_event`.
///
/// Per-tool post-install hooks (e.g. node → fnm PowerShell profile) are
/// handled inside this function for now; if the catalog grows past a
/// handful of tools, switch to a `post_install` field on the recipe.
#[tauri::command]
pub async fn install_tool(
    id: String,
    params: HashMap<String, String>,
    on_event: Channel<InstallEvent>,
) -> Result<(), String> {
    install_recipe(&id, &params, &on_event).await
}

/// Execution core for installing a single tool. Extracted from the
/// `install_tool` command so `import_snapshot` can replay installs through
/// the exact same machinery (steps → post-install hooks → PATH additions →
/// version check). The `on_event` channel is borrowed, not owned, so the
/// caller can drive many installs over one channel.
pub async fn install_recipe(
    id: &str,
    params: &HashMap<String, String>,
    on_event: &Channel<InstallEvent>,
) -> Result<(), String> {
    let recipe = Recipe::load_optional(id).ok_or_else(|| {
        format!("unknown tool id: '{id}' (no recipe at resources/recipes/{id}.toml)")
    })?;
    let platform = current_platform();
    if !recipe.install.contains_key(platform) {
        return Err(format!(
            "recipe '{id}' has no install steps for platform '{platform}'"
        ));
    }

    let substituted = recipe.substitute(params).map_err(|e| e.to_string())?;
    let substituted_install = substituted
        .install
        .get(platform)
        .expect("substitute preserves platform keys");

    let total = substituted_install.steps.len().max(1);
    for (i, step) in substituted_install.steps.iter().enumerate() {
        let argv: Vec<String> = std::iter::once(step.cmd.clone())
            .chain(step.args.iter().cloned())
            .collect();
        let mut rx = executor::run(&argv, None)
            .await
            .map_err(|e| e.to_string())?;
        let mut exit_code: i32 = 0;
        while let Some(ev) = rx.recv().await {
            match ev {
                StreamEvent::Line(l) => {
                    let _ = on_event.send(InstallEvent::Log { line: l });
                }
                StreamEvent::Exit(c) => {
                    exit_code = c;
                }
            }
        }
        if exit_code != 0 {
            let msg = format!("step `{}` exited with code {exit_code}", step.cmd);
            let _ = on_event.send(InstallEvent::Done {
                ok: false,
                version: None,
                error: Some(msg.clone()),
            });
            return Err(msg);
        }
        let _ = on_event.send(InstallEvent::Progress {
            pct: ((i + 1) * 100 / total) as u8,
        });
    }

    // Per-tool post-install hooks.
    match id {
        "node" => {
            // fnm needs the PowerShell profile integration so a new terminal
            // can resolve `node` (fnm install only places the binary in fnm's
            // own dir, not on PATH).
            match config::ensure_fnm_in_powershell_profiles() {
                Ok(changed) if !changed.is_empty() => {
                    let _ = on_event.send(InstallEvent::Log {
                        line: format!(
                            "[ok] 已写入 PowerShell 集成（{}）；新开终端即可用 node",
                            changed.join(", ")
                        ),
                    });
                }
                Ok(_) => {
                    let _ = on_event.send(InstallEvent::Log {
                        line: "[ok] PowerShell 集成已存在，无需重复写入".into(),
                    });
                }
                Err(e) => {
                    let _ = on_event.send(InstallEvent::Log {
                        line: format!(
                            "[warn] 写入 PowerShell 集成失败：{e}；你可能需手动在 PS profile 加入 fnm env 行"
                        ),
                    });
                }
            }
        }
        "python" => {
            // Switching Python versions: Flint installs each version in its
            // own `python-{ver}` dir and only ever *appends* to PATH. Without
            // pruning the old version's dirs, `python` keeps resolving to the
            // first match (the old version). Remove every Flint python-* dir
            // except the target's, *before* the add-to-PATH step below re-adds
            // the target — so the target becomes the only Flint Python on PATH.
            if let Some(ver) = params.get("python_version") {
                match config::prune_user_python_paths(ver) {
                    Ok(()) => {
                        let _ = on_event.send(InstallEvent::Log {
                            line: format!("[ok] 已清理旧版 Python 的 PATH，目标版本 {ver} 生效"),
                        });
                    }
                    Err(e) => {
                        let _ = on_event.send(InstallEvent::Log {
                            line: format!("[warn] 清理旧版 Python PATH 失败：{e}"),
                        });
                    }
                }
            }
        }
        _ => {}
    }

    // Recipe-declared PATH additions (`add_to_user_path` entries in the
    // `[install.<platform>]` block). Each entry may have `{name}`
    // placeholders (already substituted) and `%VAR%` env vars (expanded
    // here against the current process env).
    let env_snapshot: HashMap<String, String> = std::env::vars().collect();
    for raw in &substituted_install.add_to_user_path {
        let expanded = recipe::expand_env_vars(raw, &env_snapshot);
        match config::add_to_user_path(Path::new(&expanded)) {
            Ok(true) => {
                let _ = on_event.send(InstallEvent::Log {
                    line: format!("[ok] 已加入用户 PATH：{expanded}"),
                });
            }
            Ok(false) => {
                let _ = on_event.send(InstallEvent::Log {
                    line: format!("[skip] 已在用户 PATH 中：{expanded}"),
                });
            }
            Err(e) => {
                let _ = on_event.send(InstallEvent::Log {
                    line: format!("[warn] 加入用户 PATH 失败（{expanded}）：{e}"),
                });
            }
        }
    }

    // Best-effort in-process version check. Note: after a PATH-modifying
    // install, this process's env is stale (registry is the source of truth
    // for the *next* shell). We still try, and surface a hint if the
    // freshly-installed tool isn't on our PATH yet.
    let version =
        detector::detect_version(&recipe_detect_cmd(&recipe), &recipe_detect_args(&recipe))
            .ok()
            .flatten();
    if version.is_none() {
        let _ = on_event.send(InstallEvent::Log {
            line: "[!] 安装成功，但当前进程 PATH 未刷新；请重开终端后再点『重新检测』".into(),
        });
    }
    let _ = on_event.send(InstallEvent::Done {
        ok: version.is_some(),
        version: version.clone(),
        error: None,
    });
    Ok(())
}

fn current_platform() -> &'static str {
    #[cfg(windows)]
    {
        "windows"
    }
    #[cfg(target_os = "macos")]
    {
        "macos"
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        "linux"
    }
}

fn parse_category(s: &str) -> ToolCategory {
    match s {
        "ai-tool" | "ai" => ToolCategory::AiTool,
        _ => ToolCategory::Runtime,
    }
}

fn recipe_detect_cmd(recipe: &Recipe) -> String {
    recipe
        .detect
        .get(current_platform())
        .map(|c| c.cmd.clone())
        .unwrap_or_else(|| {
            // Fallback: the recipe's `meta.id` is usually the binary name.
            recipe.meta.id.clone()
        })
}

fn recipe_detect_args(recipe: &Recipe) -> Vec<String> {
    recipe
        .detect
        .get(current_platform())
        .map(|c| c.args.clone())
        .unwrap_or_else(|| vec!["--version".to_string()])
}
