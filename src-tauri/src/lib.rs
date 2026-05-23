mod commands;
mod copy;
mod folder;
mod scan;
mod skill;

use commands::{
    add_recent_destination, cancel_scan, copy_skills_command, find_skill_containers_command,
    get_scan_root, get_settings, init_settings, list_skill_files_command, read_skill_file_command,
    set_scan_root, start_scan, AppState,
};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::default())
        .setup(|app| {
            let state = app.state::<AppState>();
            init_settings(app.handle(), &state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_scan_root,
            set_scan_root,
            get_settings,
            add_recent_destination,
            start_scan,
            cancel_scan,
            copy_skills_command,
            find_skill_containers_command,
            list_skill_files_command,
            read_skill_file_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
