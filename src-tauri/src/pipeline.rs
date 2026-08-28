use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Emitter, Manager};

use crate::asr;
use crate::export::build_txt;
use crate::ffmpeg;
use crate::history;
use crate::model::{
    CanceledEvent, CompletedEvent, FailedEvent, ProgressEvent, SegmentsEvent, TranscriptResult,
    TranscriptSegment,
};
use crate::project_store::ProjectStore;
use crate::runtime::RuntimePaths;

/// Handle the frontend keeps for the running transcription job.
#[derive(Clone)]
pub struct ActiveJob {
    pub id: String,
    pub project_id: String,
    pub cancel: Arc<AtomicBool>,
    pub paused: Arc<AtomicBool>,
    /// False once the pipeline thread has finished (completed, failed or canceled).
    pub running: Arc<AtomicBool>,
    /// Slot holding the currently running ASR child so cancellation can kill it
    /// mid-segment instead of waiting up to one segment window.
    pub current_child: Arc<Mutex<Option<std::process::Child>>>,
}

enum Outcome {
    Completed(TranscriptResult),
    Canceled,
}

pub enum LiveOutcome {
    Completed(TranscriptResult),
    Canceled,
}

pub fn run_live_chunks(
    app: &AppHandle,
    job: &ActiveJob,
    runtime: &RuntimePaths,
    chunks: std::sync::mpsc::Receiver<crate::recording::RecordedChunk>,
    store: &ProjectStore,
) -> Result<LiveOutcome, String> {
    let mut all = Vec::new();
    let mut duration = 0.0f64;
    for chunk in chunks {
        if job.canceled() {
            return Ok(LiveOutcome::Canceled);
        }
        emit_progress(
            app,
            job,
            "recording",
            0,
            chunk.start_seconds,
            chunk.start_seconds + chunk.duration_seconds,
            chunk.index,
            0,
        );
        let child = asr::spawn(
            &runtime.asr_exe,
            &runtime.asr_model,
            &runtime.vad_model,
            &chunk.path,
        )?;
        *slot(job) = Some(child);
        let output = asr::collect_output_shared(&job.current_child)?;
        if job.canceled() {
            return Ok(LiveOutcome::Canceled);
        }
        if !output.success {
            let detail = output
                .stderr
                .lines()
                .filter(|line| !line.trim().is_empty())
                .take(2)
                .collect::<Vec<_>>()
                .join(" | ");
            return Err(format!("录音第 {} 段识别失败：{detail}", chunk.index));
        }
        let next = asr::offset_cues(
            &asr::parse_transcript_output(&output.stdout, chunk.duration_seconds),
            chunk.start_seconds,
            chunk.index as usize,
        );
        all.extend(next.clone());
        store.write_transcript(
            &job.project_id,
            &crate::model::TranscriptDocument {
                schema_version: 1,
                project_id: job.project_id.clone(),
                revision: 0,
                segments: all.clone(),
            },
        )?;
        emit_ok(
            app,
            "transcript://segments",
            SegmentsEvent {
                project_id: job.project_id.clone(),
                job_id: job.id.clone(),
                segments: next,
            },
        );
        duration = duration.max(chunk.start_seconds + chunk.duration_seconds);
        let _ = std::fs::remove_file(&chunk.path);
    }
    if job.canceled() {
        return Ok(LiveOutcome::Canceled);
    }
    Ok(LiveOutcome::Completed(TranscriptResult {
        file_name: "recording.wav".to_string(),
        duration,
        language: None,
        segments: all,
    }))
}

/// Spawn the pipeline thread for a job. All progress travels through events;
/// the caller only needs the job handle for cancellation.
pub fn spawn_job(
    app: AppHandle,
    job: ActiveJob,
    runtime: RuntimePaths,
    input: PathBuf,
    project_store: Option<ProjectStore>,
) {
    let running = job.running.clone();
    std::thread::spawn(move || {
        let temp_dir = std::env::temp_dir().join("snapscribe").join(job.id.clone());
        let outcome = run(
            &app,
            &job,
            &runtime,
            &input,
            &temp_dir,
            project_store.as_ref(),
        );
        // Remove segment files on every exit path (completed / failed / canceled).
        let _ = std::fs::remove_dir_all(&temp_dir);
        match outcome {
            Ok(Outcome::Completed(result)) => {
                if let Some(store) = project_store.as_ref() {
                    if let Err(message) =
                        store.complete_transcription(&job.project_id, &result.segments)
                    {
                        emit_ok(
                            &app,
                            "transcript://failed",
                            FailedEvent {
                                project_id: job.project_id.clone(),
                                job_id: job.id.clone(),
                                message,
                            },
                        );
                        running.store(false, Ordering::SeqCst);
                        return;
                    }
                }
                emit_ok(
                    &app,
                    "transcript://completed",
                    CompletedEvent {
                        project_id: job.project_id.clone(),
                        job_id: job.id.clone(),
                        result,
                    },
                );
            }
            Ok(Outcome::Canceled) => {
                if let Some(store) = project_store.as_ref() {
                    if let Err(message) = store.cancel_transcription(&job.project_id) {
                        emit_ok(
                            &app,
                            "transcript://failed",
                            FailedEvent {
                                project_id: job.project_id.clone(),
                                job_id: job.id.clone(),
                                message,
                            },
                        );
                        running.store(false, Ordering::SeqCst);
                        return;
                    }
                }
                emit_ok(
                    &app,
                    "transcript://canceled",
                    CanceledEvent {
                        project_id: job.project_id.clone(),
                        job_id: job.id.clone(),
                    },
                );
            }
            Err(message) => emit_ok(
                &app,
                "transcript://failed",
                FailedEvent {
                    project_id: job.project_id.clone(),
                    job_id: job.id.clone(),
                    message,
                },
            ),
        }
        running.store(false, Ordering::SeqCst);
    });
}

fn run(
    app: &AppHandle,
    job: &ActiveJob,
    runtime: &RuntimePaths,
    input: &PathBuf,
    temp_dir: &PathBuf,
    project_store: Option<&ProjectStore>,
) -> Result<Outcome, String> {
    if job.canceled() {
        return Ok(Outcome::Canceled);
    }

    let info = crate::media::probe(&runtime.ffprobe, input)?;
    let total_seconds = info.duration_secs;
    emit_progress(app, job, "preparing", 0, 0.0, total_seconds, 0, 1);

    let segments = ffmpeg::run_extract_split(&runtime.ffmpeg, input, temp_dir)?;
    let segment_count = segments.len() as u32;

    let mut all: Vec<TranscriptSegment> = Vec::new();
    let mut offset_seconds = 0.0f64;
    let mut processed_seconds = 0.0f64;
    let mut emitted_percent = 0u32;

    for (index, seg_path) in segments.iter().enumerate() {
        if job.canceled() {
            return Ok(Outcome::Canceled);
        }
        let seg_index = index + 1;
        let mut pause_announced = false;
        while job.paused() {
            if job.canceled() {
                return Ok(Outcome::Canceled);
            }
            if !pause_announced {
                emit_progress(
                    app,
                    job,
                    "paused",
                    emitted_percent,
                    processed_seconds,
                    total_seconds,
                    seg_index as u32,
                    segment_count,
                );
                pause_announced = true;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        emit_progress(
            app,
            job,
            "transcribing",
            emitted_percent,
            processed_seconds,
            total_seconds,
            seg_index as u32,
            segment_count,
        );

        // Measured per-segment durations keep the global timeline exact even if
        // the last window is short.
        let duration = ffmpeg::wav_duration_seconds(seg_path)?.max(0.0);

        let child = asr::spawn(
            &runtime.asr_exe,
            &runtime.asr_model,
            &runtime.vad_model,
            seg_path,
        )?;
        *slot(job) = Some(child);
        let output = asr::collect_output_shared(&job.current_child)?;

        // Cancellation outranks the nonzero exit caused by the kill itself.
        if job.canceled() {
            return Ok(Outcome::Canceled);
        }
        if !output.success {
            let detail: String = output
                .stderr
                .lines()
                .filter(|l| !l.trim().is_empty())
                .take(2)
                .collect::<Vec<_>>()
                .join(" | ");
            return Err(format!(
                "第 {seg_index}/{segment_count} 段识别失败：{detail}"
            ));
        }

        // A successful run with no cues is silent audio: no text, progress still advances.
        let new_segments = asr::offset_cues(
            &asr::parse_transcript_output(&output.stdout, duration),
            offset_seconds,
            seg_index,
        );
        all.extend(new_segments.clone());
        if let Some(store) = project_store {
            store.write_transcript(
                &job.project_id,
                &crate::model::TranscriptDocument {
                    schema_version: 1,
                    project_id: job.project_id.clone(),
                    revision: 0,
                    segments: all.clone(),
                },
            )?;
        }
        emit_ok(
            app,
            "transcript://segments",
            SegmentsEvent {
                project_id: job.project_id.clone(),
                job_id: job.id.clone(),
                segments: new_segments,
            },
        );

        offset_seconds += duration;
        processed_seconds += duration;
        let percent = if total_seconds > 0.0 {
            ((processed_seconds / total_seconds) * 100.0).floor() as u32
        } else {
            100
        };
        emitted_percent = percent.clamp(emitted_percent, 100);
        emit_progress(
            app,
            job,
            "transcribing",
            emitted_percent,
            processed_seconds,
            total_seconds,
            seg_index as u32,
            segment_count,
        );
    }

    let file_name = input
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_string();

    if job.project_id.is_empty() {
        auto_save_history(app, &file_name, &all);
    }

    Ok(Outcome::Completed(TranscriptResult {
        file_name,
        duration: total_seconds,
        language: None,
        segments: all,
    }))
}

fn slot<'a>(job: &'a ActiveJob) -> MutexGuard<'a, Option<std::process::Child>> {
    job.current_child.lock().expect("child slot poisoned")
}

type MutexGuard<'a, T> = std::sync::MutexGuard<'a, T>;

/// Auto-save the finished transcript into the history directory. A history
/// write failure must not discard an otherwise successful transcription, so it
/// is reported on stderr only; the UI still receives the completed event.
fn auto_save_history(app: &AppHandle, source_file_name: &str, segments: &[TranscriptSegment]) {
    let dir = history_dir(app);
    let stem = std::path::Path::new(source_file_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("transcript");
    if let Err(err) = history::save_transcript(&dir, stem, &build_txt(segments)) {
        eprintln!("历史保存失败（不影响转写结果）: {err}");
    }
}

pub fn history_dir(app: &AppHandle) -> PathBuf {
    let base = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    base.join("transcripts")
}

fn emit_progress(
    app: &AppHandle,
    job: &ActiveJob,
    stage: &'static str,
    percent: u32,
    processed: f64,
    total: f64,
    segment_index: u32,
    segment_count: u32,
) {
    emit_ok(
        app,
        "transcript://progress",
        ProgressEvent {
            project_id: job.project_id.clone(),
            job_id: job.id.clone(),
            stage,
            percent,
            processed_seconds: processed,
            total_seconds: total,
            segment_index,
            segment_count,
        },
    );
}

fn emit_ok<T: serde::Serialize + Clone>(app: &AppHandle, event: &'static str, payload: T) {
    if let Err(err) = app.emit(event, payload) {
        eprintln!("事件 {event} 发送失败: {err}");
    }
}

impl ActiveJob {
    pub fn canceled(&self) -> bool {
        self.cancel.load(Ordering::SeqCst)
    }

    pub fn paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }
}
