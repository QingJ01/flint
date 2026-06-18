pub mod config;
pub mod detector;
pub mod diagnose;
pub mod error;
pub mod executor;
pub mod ipc;
pub mod preset;
pub mod recipe;
pub mod shell;
pub mod snapshot;
pub mod version;
pub mod versions;
pub mod wsl;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            ipc::detect_environment,
            ipc::list_installable_tools,
            ipc::install_tool,
            ipc::list_presets,
            ipc::get_preset,
            ipc::wsl_status,
            ipc::wsl_enable,
            ipc::wsl_install_dev_tools,
            ipc::mirror_status,
            ipc::apply_npm_mirror,
            ipc::apply_pip_mirror,
            ipc::apply_domestic_acceleration,
            ipc::diagnose_tool,
            ipc::verify_anthropic_key,
            ipc::list_tool_versions,
            ipc::current_snapshot,
            ipc::export_snapshot,
            ipc::import_snapshot
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
