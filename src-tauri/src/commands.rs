use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_dialog::DialogExt;

use crate::ai_service_store::AIServiceStore;
use crate::export;
use crate::history;
use crate::media;
use crate::model::{
    AIAuthType, AIModelConfig, AIModelDraft, AIModelStatus, AIServiceConfig, AISummary, AboutInfo,
    AppSettings, ConnectionResult, ProjectDetail, ProjectListItem, ProjectSort, StartProjectResult,
    TranscriptionJobStatus,
};
use crate::model::{
    CanceledEvent, CompletedEvent, FailedEvent, RecordingLevelEvent, RecordingStartResult,
};
use crate::model::{HistoryEntry, MediaInfo, TranscriptSegment};
use crate::pipeline::{self, ActiveJob};
use crate::project_store::ProjectStore;
use crate::recording::RecordingSession;
use crate::runtime::RuntimePaths;
use crate::secret::SecretStore;
use crate::settings::SettingsStore;

/// Global single-job registry. The UI only exposes one transcription at a time.
#[derive(Default)]
pub struct AppState {
    pub job: Mutex<Option<ActiveJob>>,
    pub recording: Mutex<Option<ActiveRecording>>,
}

pub struct ActiveRecording {
    pub project_id: String,
    pub session: RecordingSession,
    pub job: ActiveJob,
    pub worker: JoinHandle<Result<pipeline::LiveOutcome, String>>,
    pub chunk_dir: PathBuf,
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
pub fn select_directory(app: AppHandle) -> Result<Option<String>, String> {
    Ok(app
        .dialog()
        .file()
        .blocking_pick_folder()
        .map(|folder| folder.to_string()))
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
        project_id: String::new(),
        cancel: Arc::new(AtomicBool::new(false)),
        paused: Arc::new(AtomicBool::new(false)),
        running: Arc::new(AtomicBool::new(true)),
        current_child: Arc::new(Mutex::new(None)),
    };
    pipeline::spawn_job(app, job.clone(), runtime, input, None);
    *current = Some(job);
    Ok(current.as_ref().expect("just stored").id.clone())
}

#[tauri::command]
pub fn project_create_from_media(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
    diarization_enabled: bool,
) -> Result<StartProjectResult, String> {
    let runtime = resolve_runtime(&app)?;
    let input = checked_input(&path)?;
    let info = media::probe(&runtime.ffprobe, &input)?;

    let mut current = state.job.lock().expect("job registry poisoned");
    if current
        .as_ref()
        .is_some_and(|job| job.running.load(Ordering::SeqCst))
    {
        return Err("已有转写任务正在进行，请等待完成或先取消".to_string());
    }

    let settings = settings_store(&app)?.load()?;
    let store = ProjectStore::new(PathBuf::from(&settings.data_root));
    let project = store.create_imported_project_with_options(
        &input,
        &info,
        settings.import_strategy,
        diarization_enabled,
    )?;
    let transcription_input = project
        .media
        .path
        .as_deref()
        .map(PathBuf::from)
        .ok_or("项目媒体路径为空")?;
    let job = ActiveJob {
        id: uuid::Uuid::new_v4().to_string(),
        project_id: project.id.clone(),
        cancel: Arc::new(AtomicBool::new(false)),
        paused: Arc::new(AtomicBool::new(false)),
        running: Arc::new(AtomicBool::new(true)),
        current_child: Arc::new(Mutex::new(None)),
    };
    pipeline::spawn_job(app, job.clone(), runtime, transcription_input, Some(store));
    let result = StartProjectResult {
        project_id: project.id,
        job_id: job.id.clone(),
    };
    *current = Some(job);
    Ok(result)
}

#[tauri::command]
pub fn project_list(
    app: AppHandle,
    query: Option<String>,
    sort: ProjectSort,
) -> Result<Vec<ProjectListItem>, String> {
    project_store(&app)?.list_projects(query.as_deref().unwrap_or_default(), sort)
}

#[tauri::command]
pub fn project_get(app: AppHandle, project_id: String) -> Result<ProjectDetail, String> {
    project_store(&app)?.read_detail(&project_id)
}

#[tauri::command]
pub fn project_delete(app: AppHandle, project_id: String) -> Result<(), String> {
    project_store(&app)?.delete_project(&project_id)
}

#[tauri::command]
pub fn project_rename(
    app: AppHandle,
    project_id: String,
    name: String,
) -> Result<crate::model::TranscriptionProject, String> {
    project_store(&app)?.rename_project(&project_id, &name)
}

#[tauri::command]
pub fn project_save_transcript(
    app: AppHandle,
    project_id: String,
    segments: Vec<TranscriptSegment>,
) -> Result<crate::model::TranscriptDocument, String> {
    project_store(&app)?.save_transcript_segments(&project_id, segments)
}

#[tauri::command]
pub fn project_update_speakers(
    app: AppHandle,
    project_id: String,
    speakers: Vec<crate::model::TranscriptSpeaker>,
) -> Result<crate::model::TranscriptDocument, String> {
    project_store(&app)?.update_speakers(&project_id, speakers)
}

#[tauri::command]
pub fn project_relink_media(
    app: AppHandle,
    project_id: String,
    path: String,
) -> Result<crate::model::TranscriptionProject, String> {
    let input = checked_input(&path)?;
    let runtime = resolve_runtime(&app)?;
    let info = media::probe(&runtime.ffprobe, &input)?;
    project_store(&app)?.relink_media(&project_id, &input, &info)
}

#[tauri::command]
pub fn project_delete_managed_media(
    app: AppHandle,
    project_id: String,
) -> Result<crate::model::TranscriptionProject, String> {
    project_store(&app)?.delete_managed_media(&project_id)
}

#[tauri::command]
pub fn project_open_media_location(app: AppHandle, project_id: String) -> Result<(), String> {
    let detail = project_store(&app)?.read_detail(&project_id)?;
    let path = detail.project.media.path.ok_or("原始媒体不可用")?;
    let path = checked_input(&path)?;
    let mut command = std::process::Command::new("explorer.exe");
    crate::proc::hide_console(&mut command);
    command
        .arg(format!("/select,{}", path.display()))
        .spawn()
        .map_err(|e| format!("无法打开原文件位置：{e}"))?;
    Ok(())
}

#[tauri::command]
pub fn project_export(
    app: AppHandle,
    project_id: String,
    format: String,
    path: String,
) -> Result<(), String> {
    let detail = project_store(&app)?.read_detail(&project_id)?;
    let content = match format.as_str() {
        "txt" => export::build_txt(&detail.transcript),
        "srt" => export::build_srt(&detail.transcript),
        "summary" => detail
            .summary
            .as_ref()
            .map(export::build_summary_txt)
            .ok_or("当前项目尚未生成 AI 总结")?,
        _ => return Err("仅支持转录文本、SRT 或 AI 总结导出".to_string()),
    };
    export::write_utf8(Path::new(&path), &content)
}

#[tauri::command]
pub fn recording_start(
    app: AppHandle,
    state: State<'_, AppState>,
    diarization_enabled: bool,
) -> Result<RecordingStartResult, String> {
    if state
        .job
        .lock()
        .expect("job registry poisoned")
        .as_ref()
        .is_some_and(|job| job.running.load(Ordering::SeqCst))
    {
        return Err("已有转写任务正在进行".to_string());
    }
    let mut active = state.recording.lock().expect("recording registry poisoned");
    if active.is_some() {
        return Err("录音已经开始".to_string());
    }
    let runtime = resolve_runtime(&app)?;
    let store = project_store(&app)?;
    let project = store.create_recording_project_with_options(diarization_enabled)?;
    let path = project
        .media
        .path
        .as_deref()
        .map(PathBuf::from)
        .ok_or("录音文件路径为空")?;
    let chunk_dir = store.project_dir(&project.id).join("live-chunks");
    let started = match crate::recording::start(path, chunk_dir.clone()) {
        Ok(started) => started,
        Err(error) => {
            let _ = store.delete_project(&project.id);
            return Err(error);
        }
    };
    let job = ActiveJob {
        id: uuid::Uuid::new_v4().to_string(),
        project_id: project.id.clone(),
        cancel: Arc::new(AtomicBool::new(false)),
        paused: Arc::new(AtomicBool::new(false)),
        running: Arc::new(AtomicBool::new(true)),
        current_child: Arc::new(Mutex::new(None)),
    };
    let level_app = app.clone();
    let level_recording_id = started.session.id.clone();
    let level_project_id = project.id.clone();
    std::thread::spawn(move || {
        let mut last_emit = std::time::Instant::now() - std::time::Duration::from_millis(100);
        for level in started.levels {
            if last_emit.elapsed() < std::time::Duration::from_millis(80) {
                continue;
            }
            last_emit = std::time::Instant::now();
            let _ = level_app.emit(
                "recording://level",
                RecordingLevelEvent {
                    recording_id: level_recording_id.clone(),
                    project_id: level_project_id.clone(),
                    level,
                },
            );
        }
    });
    let worker_app = app.clone();
    let worker_job = job.clone();
    let worker_store = store.clone();
    let chunks = started.chunks;
    let worker = std::thread::spawn(move || {
        pipeline::run_live_chunks(&worker_app, &worker_job, &runtime, chunks, &worker_store)
    });
    let result = RecordingStartResult {
        recording_id: started.session.id.clone(),
        project_id: project.id.clone(),
        job_id: job.id.clone(),
    };
    *active = Some(ActiveRecording {
        project_id: project.id,
        session: started.session,
        job: job.clone(),
        worker,
        chunk_dir,
    });
    *state.job.lock().expect("job registry poisoned") = Some(job);
    Ok(result)
}

#[tauri::command]
pub fn recording_set_paused(
    state: State<'_, AppState>,
    recording_id: String,
    paused: bool,
) -> Result<(), String> {
    let active = state.recording.lock().expect("recording registry poisoned");
    let active = active
        .as_ref()
        .filter(|active| active.session.id == recording_id)
        .ok_or("录音任务不存在")?;
    if paused {
        active.session.pause()
    } else {
        active.session.resume()
    }
}

#[tauri::command]
pub fn recording_stop(
    app: AppHandle,
    state: State<'_, AppState>,
    recording_id: String,
) -> Result<StartProjectResult, String> {
    let active = state
        .recording
        .lock()
        .expect("recording registry poisoned")
        .take()
        .ok_or("没有正在进行的录音")?;
    if active.session.id != recording_id {
        *state.recording.lock().expect("recording registry poisoned") = Some(active);
        return Err("录音任务不匹配".to_string());
    }
    let path = active.session.stop()?;
    let runtime = resolve_runtime(&app)?;
    let info = media::probe(&runtime.ffprobe, &path)?;
    let store = project_store(&app)?;
    let project = store.finalize_recording(&active.project_id, &info)?;
    let job = active.job.clone();
    let worker = active.worker;
    let chunk_dir = active.chunk_dir;
    let finish_app = app.clone();
    let finish_store = store.clone();
    let finish_job = job.clone();
    let finish_path = path.clone();
    let diarization_enabled = project.diarization.enabled;
    std::thread::spawn(move || {
        let outcome = worker
            .join()
            .map_err(|_| "实时转录线程异常退出".to_string())
            .and_then(|value| value);
        let _ = std::fs::remove_dir_all(chunk_dir);
        match outcome {
            Ok(pipeline::LiveOutcome::Completed(mut result)) => {
                let applied = if diarization_enabled {
                    let _ = finish_app.emit(
                        "transcript://progress",
                        crate::model::ProgressEvent {
                            project_id: finish_job.project_id.clone(),
                            job_id: finish_job.id.clone(),
                            stage: "diarizing",
                            percent: 0,
                            processed_seconds: 0.0,
                            total_seconds: result.duration,
                            segment_index: 0,
                            segment_count: 0,
                        },
                    );
                    let temp_dir = finish_store
                        .project_dir(&finish_job.project_id)
                        .join("diarization-temp");
                    let wav = temp_dir.join("recording-full.wav");
                    let outcome = crate::ffmpeg::run_extract_mono_wav(
                        &runtime.ffmpeg,
                        &finish_path,
                        &wav,
                    )
                    .and_then(|_| pipeline::diarize_wav(&finish_job, &runtime, &wav));
                    let _ = std::fs::remove_dir_all(temp_dir);
                    crate::diarization::apply_result(&result.segments, &[], outcome)
                } else {
                    crate::diarization::apply_result(
                        &result.segments,
                        &[],
                        Err("说话人识别未启用".to_string()),
                    )
                };
                result.segments = applied.assignment.segments.clone();
                let document = crate::model::TranscriptDocument {
                    schema_version: 2,
                    project_id: finish_job.project_id.clone(),
                    revision: 0,
                    speakers: applied.assignment.speakers,
                    segments: result.segments.clone(),
                };
                let diarization_state = if diarization_enabled {
                    applied.state
                } else {
                    crate::model::DiarizationState::default()
                };
                if let Err(message) = finish_store.complete_transcription_document(
                    &finish_job.project_id,
                    &document,
                    diarization_state,
                ) {
                    let _ = finish_app.emit(
                        "transcript://failed",
                        FailedEvent {
                            project_id: finish_job.project_id.clone(),
                            job_id: finish_job.id.clone(),
                            message,
                        },
                    );
                } else {
                    let _ = finish_app.emit(
                        "transcript://completed",
                        CompletedEvent {
                            project_id: finish_job.project_id.clone(),
                            job_id: finish_job.id.clone(),
                            result,
                        },
                    );
                }
            }
            Ok(pipeline::LiveOutcome::Canceled) => {
                let _ = finish_store.cancel_transcription(&finish_job.project_id);
                let _ = finish_app.emit(
                    "transcript://canceled",
                    CanceledEvent {
                        project_id: finish_job.project_id.clone(),
                        job_id: finish_job.id.clone(),
                    },
                );
            }
            Err(message) => {
                let _ = finish_app.emit(
                    "transcript://failed",
                    FailedEvent {
                        project_id: finish_job.project_id.clone(),
                        job_id: finish_job.id.clone(),
                        message,
                    },
                );
            }
        }
        finish_job.running.store(false, Ordering::SeqCst);
    });
    Ok(StartProjectResult {
        project_id: project.id,
        job_id: job.id,
    })
}

#[tauri::command]
pub fn recording_cancel(
    app: AppHandle,
    state: State<'_, AppState>,
    recording_id: String,
) -> Result<(), String> {
    let active = state
        .recording
        .lock()
        .expect("recording registry poisoned")
        .take()
        .ok_or("没有正在进行的录音")?;
    if active.session.id != recording_id {
        *state.recording.lock().expect("recording registry poisoned") = Some(active);
        return Err("录音任务不匹配".to_string());
    }
    active.job.cancel.store(true, Ordering::SeqCst);
    if let Some(child) = active
        .job
        .current_child
        .lock()
        .expect("child slot poisoned")
        .as_mut()
    {
        let _ = child.kill();
    }
    let _ = active.session.stop();
    let _ = active.worker.join();
    active.job.running.store(false, Ordering::SeqCst);
    let _ = std::fs::remove_dir_all(active.chunk_dir);
    project_store(&app)?.delete_project(&active.project_id)
}

#[tauri::command]
pub fn settings_get(app: AppHandle) -> Result<AppSettings, String> {
    let mut settings = settings_store(&app)?.load()?;
    settings.ai.has_api_key = secret_store(&app)?.has_key();
    settings.api_key = None;
    settings.clear_api_key = false;
    Ok(settings)
}

#[tauri::command]
pub fn about_get() -> AboutInfo {
    AboutInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        build_time: "2026-08-31".to_string(),
        repository_url: "https://github.com/fengwuyun/SnapScribe".to_string(),
    }
}

#[tauri::command]
pub fn open_repository() -> Result<(), String> {
    let mut command = std::process::Command::new("explorer.exe");
    crate::proc::hide_console(&mut command);
    command
        .arg("https://github.com/fengwuyun/SnapScribe")
        .spawn()
        .map_err(|e| format!("无法打开 GitHub 仓库：{e}"))?;
    Ok(())
}

#[tauri::command]
pub fn settings_open_data_root(app: AppHandle) -> Result<(), String> {
    let settings = settings_store(&app)?.load()?;
    let path = PathBuf::from(settings.data_root);
    std::fs::create_dir_all(&path).map_err(|e| format!("无法创建数据目录：{e}"))?;
    let mut command = std::process::Command::new("explorer.exe");
    crate::proc::hide_console(&mut command);
    command
        .arg(&path)
        .spawn()
        .map_err(|e| format!("无法打开数据目录：{e}"))?;
    Ok(())
}

#[tauri::command]
pub fn settings_save(app: AppHandle, mut settings: AppSettings) -> Result<AppSettings, String> {
    let store = settings_store(&app)?;
    let current = store.load()?;
    let next_root = settings.data_root.trim();
    if next_root.is_empty() {
        return Err("转录数据保存位置不能为空".to_string());
    }
    store.migrate_data_root(&current.data_root, next_root)?;
    let secrets = secret_store(&app)?;
    if settings.clear_api_key {
        secrets.clear()?;
    } else if let Some(key) = settings
        .api_key
        .as_deref()
        .filter(|key| !key.trim().is_empty())
    {
        secrets.save(key)?;
    }
    settings.data_root = next_root.to_string();
    settings.ai.has_api_key = secrets.has_key();
    settings.api_key = None;
    settings.clear_api_key = false;
    store.save(&settings)?;
    Ok(settings)
}

#[tauri::command]
pub fn ai_test_connection(
    app: AppHandle,
    settings: AppSettings,
) -> Result<ConnectionResult, String> {
    let key = resolve_api_key(&app, settings.api_key.as_deref())?;
    crate::ai::test_connection(&settings.ai, &key)
}

#[tauri::command]
pub fn ai_service_get(app: AppHandle) -> Result<AIServiceConfig, String> {
    let store = ensure_ai_service_migrated(&app)?;
    hydrate_ai_config(&store)
}

#[tauri::command]
pub fn ai_instruction_save(app: AppHandle, instruction: String) -> Result<AIServiceConfig, String> {
    let store = ensure_ai_service_migrated(&app)?;
    store.save_instruction(&instruction)?;
    hydrate_ai_config(&store)
}

#[tauri::command]
pub fn ai_model_create(app: AppHandle, draft: AIModelDraft) -> Result<AIModelConfig, String> {
    let store = ensure_ai_service_migrated(&app)?;
    if draft.auth_type != AIAuthType::None
        && draft
            .api_key
            .as_deref()
            .is_none_or(|key| key.trim().is_empty())
    {
        return Err("请填写 API Key".to_string());
    }
    let model = store.create_model(&draft)?;
    save_model_secret(&store, &model.id, &draft)?;
    hydrate_ai_model(&store, model)
}

#[tauri::command]
pub fn ai_model_update(
    app: AppHandle,
    model_id: String,
    draft: AIModelDraft,
) -> Result<AIModelConfig, String> {
    let store = ensure_ai_service_migrated(&app)?;
    let model = store.update_model(&model_id, &draft)?;
    save_model_secret(&store, &model.id, &draft)?;
    hydrate_ai_model(&store, model)
}

#[tauri::command]
pub fn ai_model_delete(app: AppHandle, model_id: String) -> Result<(), String> {
    let store = ensure_ai_service_migrated(&app)?;
    store.delete_model(&model_id)?;
    SecretStore::for_model(store.config_dir(), &model_id).clear()
}

#[tauri::command]
pub fn ai_model_reorder(
    app: AppHandle,
    ordered_ids: Vec<String>,
) -> Result<AIServiceConfig, String> {
    let store = ensure_ai_service_migrated(&app)?;
    store.reorder_models(&ordered_ids)?;
    hydrate_ai_config(&store)
}

#[tauri::command]
pub fn ai_model_set_enabled(
    app: AppHandle,
    model_id: String,
    enabled: bool,
) -> Result<AIModelConfig, String> {
    let store = ensure_ai_service_migrated(&app)?;
    let model = store.set_enabled(&model_id, enabled)?;
    hydrate_ai_model(&store, model)
}

#[tauri::command]
pub fn ai_model_test(app: AppHandle, draft: AIModelDraft) -> Result<AIModelStatus, String> {
    let store = ensure_ai_service_migrated(&app)?;
    let key = if draft.auth_type == AIAuthType::None {
        String::new()
    } else if let Some(key) = draft
        .api_key
        .as_deref()
        .filter(|key| !key.trim().is_empty())
    {
        key.trim().to_string()
    } else if let Some(id) = draft.id.as_deref() {
        SecretStore::for_model(store.config_dir(), id).load()?
    } else {
        return Err("请填写 API Key".to_string());
    };
    let status = crate::ai::test_model(&draft, &key);
    if let Some(id) = draft.id.as_deref() {
        store.update_status(id, status.clone())?;
    }
    Ok(status)
}

#[tauri::command]
pub fn ai_generate_summary(app: AppHandle, project_id: String) -> Result<AISummary, String> {
    let ai_store = ensure_ai_service_migrated(&app)?;
    let projects = project_store(&app)?;
    let detail = projects.read_detail(&project_id)?;
    let transcript = export::build_txt(&detail.transcript);
    let summary = crate::ai::generate_summary_with_failover(
        &ai_store,
        ai_store.config_dir(),
        &project_id,
        detail.transcript.revision,
        &transcript,
    )?;
    projects.save_summary(&project_id, &summary)?;
    Ok(summary)
}

#[tauri::command]
pub fn project_action_items_save(
    app: AppHandle,
    project_id: String,
    items: Vec<crate::model::ActionItem>,
) -> Result<AISummary, String> {
    project_store(&app)?.save_action_items(&project_id, items)
}

/// Cancel a job by id: signal the loop and kill any in-flight ASR child.
#[tauri::command]
pub fn cancel_transcription(state: State<'_, AppState>, job_id: String) -> Result<(), String> {
    let current = state.job.lock().expect("job registry poisoned");
    match current.as_ref() {
        Some(job) if job.id == job_id => {
            job.cancel.store(true, Ordering::SeqCst);
            if let Some(child) = job
                .current_child
                .lock()
                .expect("child slot poisoned")
                .as_mut()
            {
                // Killing is best-effort; the loop reaps whatever happens next.
                let _ = child.kill();
            }
            Ok(())
        }
        _ => Err("任务不存在或已结束".to_string()),
    }
}

#[tauri::command]
pub fn transcription_job_status(
    state: State<'_, AppState>,
    project_id: String,
) -> Option<TranscriptionJobStatus> {
    let current = state.job.lock().expect("job registry poisoned");
    current
        .as_ref()
        .filter(|job| job.project_id == project_id)
        .map(|job| TranscriptionJobStatus {
            project_id: job.project_id.clone(),
            job_id: job.id.clone(),
            running: job.running.load(Ordering::SeqCst),
            paused: job.paused.load(Ordering::SeqCst),
        })
}

#[tauri::command]
pub fn set_transcription_paused(
    state: State<'_, AppState>,
    project_id: String,
    paused: bool,
) -> Result<TranscriptionJobStatus, String> {
    let current = state.job.lock().expect("job registry poisoned");
    let job = current
        .as_ref()
        .filter(|job| job.project_id == project_id && job.running.load(Ordering::SeqCst))
        .ok_or("转录任务不存在或已结束")?;
    job.paused.store(paused, Ordering::SeqCst);
    Ok(TranscriptionJobStatus {
        project_id: job.project_id.clone(),
        job_id: job.id.clone(),
        running: true,
        paused,
    })
}

#[tauri::command]
pub fn cancel_project_transcription(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<(), String> {
    let current = state.job.lock().expect("job registry poisoned");
    let job = current
        .as_ref()
        .filter(|job| job.project_id == project_id && job.running.load(Ordering::SeqCst))
        .ok_or("转录任务不存在或已结束")?;
    job.cancel.store(true, Ordering::SeqCst);
    job.paused.store(false, Ordering::SeqCst);
    if let Some(child) = job
        .current_child
        .lock()
        .expect("child slot poisoned")
        .as_mut()
    {
        let _ = child.kill();
    }
    Ok(())
}

/// Native save dialog; returns the chosen absolute path (None = canceled).
#[tauri::command]
pub fn save_file_dialog(
    app: AppHandle,
    default_name: String,
    ext: String,
) -> Result<Option<String>, String> {
    let filter_name = if ext.eq_ignore_ascii_case("txt") {
        "文本文件"
    } else {
        "SRT 字幕"
    };
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
    export::write_utf8(Path::new(&path), &export::build_txt_segments(&segments))
}

#[tauri::command]
pub fn export_srt(segments_json: String, path: String) -> Result<(), String> {
    let segments = parse_segments(&segments_json)?;
    export::write_utf8(Path::new(&path), &export::build_srt_segments(&segments))
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
pub fn history_rename(
    app: AppHandle,
    old_name: String,
    new_name: String,
) -> Result<String, String> {
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

fn settings_store(app: &AppHandle) -> Result<SettingsStore, String> {
    Ok(SettingsStore::new(config_dir(app)?))
}

fn project_store(app: &AppHandle) -> Result<ProjectStore, String> {
    let settings = settings_store(app)?.load()?;
    Ok(ProjectStore::new(PathBuf::from(settings.data_root)))
}

fn secret_store(app: &AppHandle) -> Result<SecretStore, String> {
    Ok(SecretStore::new(&config_dir(app)?))
}

fn ensure_ai_service_migrated(app: &AppHandle) -> Result<AIServiceStore, String> {
    let store = AIServiceStore::new(config_dir(app)?);
    let legacy_settings = settings_store(app)?.load()?;
    if let Some(model) = store.migrate_legacy(&legacy_settings.ai)? {
        let legacy_secret = secret_store(app)?;
        if legacy_secret.has_key() {
            SecretStore::for_model(store.config_dir(), &model.id).save(&legacy_secret.load()?)?;
        }
    }
    store.ensure_default_presets()?;
    Ok(store)
}

fn hydrate_ai_config(store: &AIServiceStore) -> Result<AIServiceConfig, String> {
    let mut config = store.load()?;
    for model in &mut config.models {
        model.has_api_key = model.auth_type == AIAuthType::None
            || SecretStore::for_model(store.config_dir(), &model.id).has_key();
    }
    Ok(config)
}

fn hydrate_ai_model(
    store: &AIServiceStore,
    mut model: AIModelConfig,
) -> Result<AIModelConfig, String> {
    model.has_api_key = model.auth_type == AIAuthType::None
        || SecretStore::for_model(store.config_dir(), &model.id).has_key();
    Ok(model)
}

fn save_model_secret(
    store: &AIServiceStore,
    model_id: &str,
    draft: &AIModelDraft,
) -> Result<(), String> {
    let secret = SecretStore::for_model(store.config_dir(), model_id);
    if draft.auth_type == AIAuthType::None || draft.clear_api_key {
        secret.clear()?;
    } else if let Some(key) = draft
        .api_key
        .as_deref()
        .filter(|key| !key.trim().is_empty())
    {
        secret.save(key)?;
    } else if !secret.has_key() {
        return Err("请填写 API Key".to_string());
    }
    Ok(())
}

fn config_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map_err(|e| format!("无法确定设置目录：{e}"))
}

fn resolve_api_key(app: &AppHandle, draft: Option<&str>) -> Result<String, String> {
    match draft.map(str::trim).filter(|key| !key.is_empty()) {
        Some(key) => Ok(key.to_string()),
        None => secret_store(app)?.load(),
    }
}
