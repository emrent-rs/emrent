mod commands;
mod peer;
mod torrent;
mod tracker;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::parse_torrent_file,
            commands::announce_to_tracker,
            commands::connect_to_peer_command,
            commands::start_download,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
