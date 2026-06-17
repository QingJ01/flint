pub mod error;
pub mod version;
pub mod recipe;
pub mod preset;
pub mod executor;
pub mod detector;
pub mod wsl;
pub mod ipc;
pub mod config;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            ipc::detect_environment,
            ipc::list_installable_tools,
            ipc::install_tool,
            ipc::list_presets,
            ipc::get_preset,
            ipc::wsl_status,
            ipc::wsl_enable,
            ipc::wsl_install_dev_tools
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
