pub mod error;
pub mod version;
pub mod recipe;
pub mod executor;
pub mod detector;
pub mod ipc;
pub mod config;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            ipc::detect_node,
            ipc::detect_environment,
            ipc::install_node
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
