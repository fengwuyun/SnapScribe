// Prevents an additional console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ai;
mod ai_service_store;
mod asr;
mod commands;
mod export;
mod ffmpeg;
mod history;
mod legacy;
mod media;
mod model;
mod pipeline;
mod proc;
mod project_store;
mod recording;
mod runtime;
mod secret;
mod settings;

use commands::AppState;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if let Err(error) = legacy::migrate(&app.handle()) {
                eprintln!("旧转录记录迁移失败：{error}");
            }
            Ok(())
        })
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::select_file,
            commands::select_directory,
            commands::get_media_info,
            commands::start_transcription,
            commands::cancel_transcription,
            commands::transcription_job_status,
            commands::set_transcription_paused,
            commands::cancel_project_transcription,
            commands::save_file_dialog,
            commands::export_txt,
            commands::export_srt,
            commands::history_list,
            commands::history_read,
            commands::history_rename,
            commands::history_delete,
            commands::history_dir_path,
            commands::project_create_from_media,
            commands::project_list,
            commands::project_get,
            commands::project_delete,
            commands::project_rename,
            commands::project_save_transcript,
            commands::project_relink_media,
            commands::project_delete_managed_media,
            commands::project_open_media_location,
            commands::project_export,
            commands::recording_start,
            commands::recording_set_paused,
            commands::recording_stop,
            commands::recording_cancel,
            commands::settings_get,
            commands::about_get,
            commands::open_repository,
            commands::settings_save,
            commands::ai_test_connection,
            commands::ai_service_get,
            commands::ai_instruction_save,
            commands::ai_model_create,
            commands::ai_model_update,
            commands::ai_model_delete,
            commands::ai_model_reorder,
            commands::ai_model_set_enabled,
            commands::ai_model_test,
            commands::ai_generate_summary,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
