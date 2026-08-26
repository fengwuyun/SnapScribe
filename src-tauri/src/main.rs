// Prevents an additional console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod asr;
mod commands;
mod export;
mod ffmpeg;
mod history;
mod media;
mod model;
mod pipeline;
mod runtime;

use commands::AppState;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::select_file,
            commands::get_media_info,
            commands::start_transcription,
            commands::cancel_transcription,
            commands::save_file_dialog,
            commands::export_txt,
            commands::export_srt,
            commands::history_list,
            commands::history_read,
            commands::history_rename,
            commands::history_delete,
            commands::history_dir_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
