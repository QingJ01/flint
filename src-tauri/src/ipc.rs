use crate::recipe::ParameterOption;
use crate::{
    config,
    detector::{self, ToolCategory, ToolStatus},
    executor::{self, StreamEvent},
    recipe::{self, Recipe},
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
    Log { line: String },
    Progress { pct: u8 },
    Done { ok: bool, version: Option<String>, error: Option<String> },
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
    let recipe = Recipe::load_optional(&id)
        .ok_or_else(|| format!("unknown tool id: '{id}' (no recipe at resources/recipes/{id}.toml)"))?;
    let platform = current_platform();
    let platform_install = recipe.install.get(platform).ok_or_else(|| {
        format!("recipe '{id}' has no install steps for platform '{platform}'")
    })?;

    let substituted = recipe.substitute(&params).map_err(|e| e.to_string())?;
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
    match id.as_str() {
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
    let version = detector::detect_version(&recipe_detect_cmd(&recipe), &recipe_detect_args(&recipe))
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
