use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;

use crate::export;
use crate::history;
use crate::media;
use crate::model::{HistoryEntry, MediaInfo, TranscriptSegment};
use crate::pipeline::{self, ActiveJob};
use crate::runtime::RuntimePaths;

/// Global single-job registry. The UI only exposes one transcription at a time.
#[derive(Default)]
pub struct AppState {
    pub job: Mutex<Option<ActiveJob>>,
}

#[tauri::command]
pub fn select_file(app: AppHandle) -> Result<Option<String>, String> {
    let picked = app
        .dialog()
        .file()
        .add_filter("视频 / 音频", &["mp4", "mov", "mkv", "mp3", "m4a", "wav"])
        .blocking_pick_file();
    Ok(picked.map(|f| f.to_string()))
}

#[tauri::command]
pub fn get_media_info(app: AppHandle, path: String) -> Result<MediaInfo, String> {
    let runtime = resolve_runtime(&app)?;
    let input = checked_input(&path)?;
    media::probe(&runtime.ffprobe, &input)
}

/// Accept one transcription job. Fails fast on unreadable media or while
/// another job is still running; results arrive via events.
#[tauri::command]
pub fn start_transcription(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> Result<String, String> {
    let runtime = resolve_runtime(&app)?;
    let input = checked_input(&path)?;
    media::probe(&runtime.ffprobe, &input)?;

    let mut current = state.job.lock().expect("job registry poisoned");
    if let Some(existing) = current.as_ref() {
        if existing.running.load(Ordering::SeqCst) {
            return Err("已有转写任务正在进行，请等待完成或先取消".to_string());
        }
    }

    let job = ActiveJob {
        id: uuid::Uuid::new_v4().to_string(),
        cancel: Arc::new(AtomicBool::new(false)),
        running: Arc::new(AtomicBool::new(true)),
        current_child: Arc::new(Mutex::new(None)),
    };
    pipeline::spawn_job(app, job.clone(), runtime, input);
    *current = Some(job);
    Ok(current.as_ref().expect("just stored").id.clone())
}

/// Cancel a job by id: signal the loop and kill any in-flight ASR child.
#[tauri::command]
pub fn cancel_transcription(state: State<'_, AppState>, job_id: String) -> Result<(), String> {
    let current = state.job.lock().expect("job registry poisoned");
    match current.as_ref() {
        Some(job) if job.id == job_id => {
            job.cancel.store(true, Ordering::SeqCst);
            if let Some(child) = job.current_child.lock().expect("child slot poisoned").as_mut() {
                // Killing is best-effort; the loop reaps whatever happens next.
                let _ = child.kill();
            }
            Ok(())
        }
        _ => Err("任务不存在或已结束".to_string()),
    }
}

/// Native save dialog; returns the chosen absolute path (None = canceled).
#[tauri::command]
pub fn save_file_dialog(
    app: AppHandle,
    default_name: String,
    ext: String,
) -> Result<Option<String>, String> {
    let filter_name = if ext.eq_ignore_ascii_case("txt") { "文本文件" } else { "SRT 字幕" };
    let file = app
        .dialog()
        .file()
        .add_filter(filter_name, &[ext.as_str()])
        .set_file_name(default_name)
        .blocking_save_file();
    Ok(file.map(|f| f.to_string()))
}

#[tauri::command]
pub fn export_txt(segments_json: String, path: String) -> Result<(), String> {
    let segments = parse_segments(&segments_json)?;
    export::write_utf8(Path::new(&path), &export::build_txt(&segments))
}

#[tauri::command]
pub fn export_srt(segments_json: String, path: String) -> Result<(), String> {
    let segments = parse_segments(&segments_json)?;
    export::write_utf8(Path::new(&path), &export::build_srt(&segments))
}

#[tauri::command]
pub fn history_list(app: AppHandle) -> Vec<HistoryEntry> {
    history::list(&pipeline::history_dir(&app))
}

#[tauri::command]
pub fn history_read(app: AppHandle, file_name: String) -> Result<String, String> {
    history::read_file(&pipeline::history_dir(&app), &file_name)
}

#[tauri::command]
pub fn history_rename(app: AppHandle, old_name: String, new_name: String) -> Result<String, String> {
    history::rename(&pipeline::history_dir(&app), &old_name, &new_name)
}

#[tauri::command]
pub fn history_delete(app: AppHandle, file_name: String) -> Result<(), String> {
    history::delete(&pipeline::history_dir(&app), &file_name)
}

/// Absolute path of the history directory, for display in the UI.
#[tauri::command]
pub fn history_dir_path(app: AppHandle) -> String {
    pipeline::history_dir(&app).to_string_lossy().to_string()
}

fn checked_input(path: &str) -> Result<PathBuf, String> {
    let input = PathBuf::from(path);
    if !input.is_file() {
        return Err(format!("文件不存在：{path}"));
    }
    Ok(input)
}

fn resolve_runtime(app: &AppHandle) -> Result<RuntimePaths, String> {
    let resource_dir = app.path().resource_dir().ok();
    RuntimePaths::resolve(resource_dir.as_deref())
}

fn parse_segments(json: &str) -> Result<Vec<TranscriptSegment>, String> {
    serde_json::from_str(json).map_err(|e| format!("段落解析失败：{e}"))
}
