// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

mod commands;


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::services::enumerate_services,
            commands::services::stop_service,
            commands::services::start_service,
            commands::services::pause_service,
            commands::services::resume_service
            ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
