use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Emitter, Manager};

use crate::asr;
use crate::export::build_txt;
use crate::ffmpeg;
use crate::history;
use crate::model::{
    CompletedEvent, FailedEvent, ProgressEvent, SegmentsEvent, TranscriptResult,
    TranscriptSegment,
};
use crate::runtime::RuntimePaths;

/// Handle the frontend keeps for the running transcription job.
#[derive(Clone)]
pub struct ActiveJob {
    pub id: String,
    pub cancel: Arc<AtomicBool>,
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

/// Spawn the pipeline thread for a job. All progress travels through events;
/// the caller only needs the job handle for cancellation.
pub fn spawn_job(app: AppHandle, job: ActiveJob, runtime: RuntimePaths, input: PathBuf) {
    let running = job.running.clone();
    std::thread::spawn(move || {
        let temp_dir = std::env::temp_dir().join("snapscribe").join(job.id.clone());
        let outcome = run(&app, &job, &runtime, &input, &temp_dir);
        // Remove segment files on every exit path (completed / failed / canceled).
        let _ = std::fs::remove_dir_all(&temp_dir);
        match outcome {
            Ok(Outcome::Completed(result)) => {
                emit_ok(&app, "transcript://completed", CompletedEvent { result });
            }
            Ok(Outcome::Canceled) => {}
            Err(message) => emit_ok(&app, "transcript://failed", FailedEvent { message }),
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
) -> Result<Outcome, String> {
    if job.canceled() {
        return Ok(Outcome::Canceled);
    }

    let info = crate::media::probe(&runtime.ffprobe, input)?;
    let total_seconds = info.duration_secs;
    emit_progress(app, &job.id, "preparing", 0, 0.0, total_seconds, 0, 1);

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
        emit_progress(
            app,
            &job.id,
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

        let child = asr::spawn(&runtime.asr_exe, &runtime.asr_model, &runtime.vad_model, seg_path)?;
        *slot(job) = Some(child);
        let taken = slot(job).take().ok_or("识别进程句柄丢失")?;
        let output = asr::collect_output(taken)?;

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
            return Err(format!("第 {seg_index}/{segment_count} 段识别失败：{detail}"));
        }

        // A successful run with no cues is silent audio: no text, progress still advances.
        let new_segments = asr::offset_cues(&asr::parse_srt(&output.stdout), offset_seconds, seg_index);
        emit_ok(
            app,
            "transcript://segments",
            SegmentsEvent {
                job_id: job.id.clone(),
                segments: new_segments.clone(),
            },
        );
        all.extend(new_segments);

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
            &job.id,
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

    auto_save_history(app, &file_name, &all);

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
    let base = app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
    base.join("transcripts")
}

fn emit_progress(
    app: &AppHandle,
    job_id: &str,
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
            job_id: job_id.to_string(),
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
}
