use serde::Serialize;
use tauri::ipc::Channel;
use crate::{config, detector::{self, ToolStatus}, executor::{self, StreamEvent}, recipe::Recipe};

/// 流式推送给前端的事件。`#[serde(tag = "type")]` 让前端可以按 `event.type` 分支。
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
pub async fn detect_environment() -> Result<Vec<ToolStatus>, String> {
    detector::detect_environment().map_err(|e| e.to_string())
}

/// 执行 node 安装配方，按行把 stdout/stderr 推给前端，并在每一步推进进度。
///
/// 注意：fnm install + fnm default 只是把 node 装到 fnm 自己管理的目录里，
/// 要让新开的终端能直接 `node -v`，还需要 PowerShell profile 里写入
/// `fnm env --use-on-cd --shell powershell | Out-String | Invoke-Expression`。
/// 那个持久化 PATH/profile 的活儿留给后续的 ConfigManager；这里只跑配方里的步骤，
/// 并在结束时如实报告（如果当前会话 PATH 未刷新，会发一条 Log 提示用户重开终端）。
#[tauri::command]
pub async fn install_node(on_event: Channel<InstallEvent>) -> Result<(), String> {
    // Slice 0: embed the recipe at compile time to avoid runtime CWD/path fragility.
    // (Recipe::load remains in recipe.rs for the future file/hot-update path.)
    let recipe: Recipe = toml::from_str(include_str!("../resources/recipes/node.toml"))
        .map_err(|e| e.to_string())?;
    let win = recipe.install.get("windows").ok_or_else(|| "no windows install steps in recipe".to_string())?;

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
            let msg = format!("step `{}` exited with code {exit_code}", step.cmd);
            let _ = on_event.send(InstallEvent::Done { ok: false, version: None, error: Some(msg.clone()) });
            return Err(msg);
        }
        let _ = on_event.send(InstallEvent::Progress { pct: ((i + 1) * 100 / total) as u8 });
    }

    // 让新终端能用上 node：把 fnm 的 shell 集成片段写入 PowerShell profile（用户级，免管理员）
    match config::ensure_fnm_in_powershell_profiles() {
        Ok(changed) if !changed.is_empty() => {
            let _ = on_event.send(InstallEvent::Log {
                line: format!("[ok] 已写入 PowerShell 集成（{}）；新开终端即可用 node", changed.join(", ")),
            });
        }
        Ok(_) => {
            let _ = on_event.send(InstallEvent::Log { line: "[ok] PowerShell 集成已存在，无需重复写入".into() });
        }
        Err(e) => {
            let _ = on_event.send(InstallEvent::Log {
                line: format!("[warn] 写入 PowerShell 集成失败：{e}；你可能需手动在 PS profile 加入 fnm env 行"),
            });
        }
    }

    // 验证：node 是否可用（注意：当前进程的 PATH 不会自动刷新，所以这里可能仍检测不到）
    let version = detector::detect_node().ok().flatten();
    if version.is_none() {
        let _ = on_event.send(InstallEvent::Log { line: "[!] node 已装但当前会话 PATH 未刷新；请新开终端运行 `node -v` 验证".into() });
    }
    let _ = on_event.send(InstallEvent::Done { ok: version.is_some(), version: version.clone(), error: None });
    Ok(())
}
