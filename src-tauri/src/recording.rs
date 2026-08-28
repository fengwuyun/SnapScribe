use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

type Writer = hound::WavWriter<BufWriter<std::fs::File>>;
pub const LIVE_CHUNK_SECONDS: u64 = 15;

#[derive(Debug)]
pub struct RecordedChunk {
    pub path: PathBuf,
    pub start_seconds: f64,
    pub duration_seconds: f64,
    pub index: u32,
}

pub struct RecordingStart {
    pub session: RecordingSession,
    pub chunks: mpsc::Receiver<RecordedChunk>,
    pub levels: mpsc::Receiver<f32>,
}

enum RecordingCommand {
    Pause(mpsc::Sender<Result<(), String>>),
    Resume(mpsc::Sender<Result<(), String>>),
    Stop,
}

pub struct RecordingSession {
    pub id: String,
    command_tx: mpsc::Sender<RecordingCommand>,
    handle: Option<JoinHandle<Result<PathBuf, String>>>,
}

impl RecordingSession {
    pub fn pause(&self) -> Result<(), String> {
        self.control(true)
    }
    pub fn resume(&self) -> Result<(), String> {
        self.control(false)
    }

    fn control(&self, pause: bool) -> Result<(), String> {
        let (reply_tx, reply_rx) = mpsc::channel();
        let command = if pause {
            RecordingCommand::Pause(reply_tx)
        } else {
            RecordingCommand::Resume(reply_tx)
        };
        self.command_tx
            .send(command)
            .map_err(|_| "录音线程已结束".to_string())?;
        reply_rx
            .recv_timeout(Duration::from_secs(2))
            .map_err(|_| "录音状态切换超时".to_string())?
    }

    pub fn stop(mut self) -> Result<PathBuf, String> {
        self.command_tx
            .send(RecordingCommand::Stop)
            .map_err(|_| "录音线程已结束".to_string())?;
        self.handle
            .take()
            .ok_or("录音线程句柄丢失")?
            .join()
            .map_err(|_| "录音线程异常退出".to_string())?
    }
}

pub fn start(path: PathBuf, chunk_dir: PathBuf) -> Result<RecordingStart, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let (command_tx, command_rx) = mpsc::channel();
    let (ready_tx, ready_rx) = mpsc::channel();
    let (chunk_tx, chunk_rx) = mpsc::channel();
    let (level_tx, level_rx) = mpsc::sync_channel(4);
    let thread_path = path.clone();
    let handle = std::thread::spawn(move || {
        run(
            thread_path,
            chunk_dir,
            command_rx,
            ready_tx,
            chunk_tx,
            level_tx,
        )
    });
    match ready_rx.recv_timeout(Duration::from_secs(8)) {
        Ok(Ok(())) => Ok(RecordingStart {
            session: RecordingSession {
                id,
                command_tx,
                handle: Some(handle),
            },
            chunks: chunk_rx,
            levels: level_rx,
        }),
        Ok(Err(message)) => {
            let _ = handle.join();
            Err(message)
        }
        Err(_) => {
            let _ = command_tx.send(RecordingCommand::Stop);
            let _ = handle.join();
            Err("启动麦克风录音超时".to_string())
        }
    }
}

fn run(
    path: PathBuf,
    chunk_dir: PathBuf,
    command_rx: mpsc::Receiver<RecordingCommand>,
    ready_tx: mpsc::Sender<Result<(), String>>,
    chunk_tx: mpsc::Sender<RecordedChunk>,
    level_tx: mpsc::SyncSender<f32>,
) -> Result<PathBuf, String> {
    let host = cpal::default_host();
    let device = match host.default_input_device() {
        Some(device) => device,
        None => return ready_error(ready_tx, "未找到默认麦克风"),
    };
    let supported = match device.default_input_config() {
        Ok(config) => config,
        Err(error) => return ready_error(ready_tx, &format!("无法读取默认麦克风配置：{error}")),
    };
    let sample_format = supported.sample_format();
    let config: cpal::StreamConfig = supported.into();
    let spec = hound::WavSpec {
        channels: config.channels,
        sample_rate: config.sample_rate.0,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    std::fs::create_dir_all(&chunk_dir).map_err(|e| format!("无法创建录音切片目录：{e}"))?;
    let capture = match CaptureState::new(path.clone(), chunk_dir, spec, chunk_tx) {
        Ok(state) => Arc::new(Mutex::new(Some(state))),
        Err(message) => {
            let _ = ready_tx.send(Err(message.clone()));
            return Err(message);
        }
    };
    let stream_error = Arc::new(Mutex::new(None::<String>));
    let error_slot = stream_error.clone();
    let error_callback = move |error| {
        *error_slot.lock().expect("recording error slot poisoned") =
            Some(format!("麦克风录音失败：{error}"));
    };
    let stream = match sample_format {
        cpal::SampleFormat::F32 => {
            build_stream::<f32>(&device, &config, capture.clone(), level_tx, error_callback)
        }
        cpal::SampleFormat::I16 => {
            build_stream::<i16>(&device, &config, capture.clone(), level_tx, error_callback)
        }
        cpal::SampleFormat::U16 => {
            build_stream::<u16>(&device, &config, capture.clone(), level_tx, error_callback)
        }
        other => Err(format!("暂不支持麦克风采样格式：{other:?}")),
    };
    let stream = match stream {
        Ok(stream) => stream,
        Err(message) => {
            let _ = ready_tx.send(Err(message.clone()));
            return Err(message);
        }
    };
    if let Err(error) = stream.play() {
        return ready_error(ready_tx, &format!("无法启动麦克风：{error}"));
    }
    let _ = ready_tx.send(Ok(()));
    loop {
        match command_rx.recv() {
            Ok(RecordingCommand::Pause(reply)) => {
                let _ = reply.send(stream.pause().map_err(|e| format!("无法暂停录音：{e}")));
            }
            Ok(RecordingCommand::Resume(reply)) => {
                let _ = reply.send(stream.play().map_err(|e| format!("无法继续录音：{e}")));
            }
            Ok(RecordingCommand::Stop) | Err(_) => break,
        }
    }
    drop(stream);
    if let Some(message) = stream_error
        .lock()
        .expect("recording error slot poisoned")
        .take()
    {
        return Err(message);
    }
    let state = capture
        .lock()
        .expect("recording capture poisoned")
        .take()
        .ok_or("录音写入器已关闭")?;
    state.finish()?;
    Ok(path)
}

fn ready_error<T>(ready_tx: mpsc::Sender<Result<(), String>>, message: &str) -> Result<T, String> {
    let message = message.to_string();
    let _ = ready_tx.send(Err(message.clone()));
    Err(message)
}

struct CaptureState {
    full: Option<Writer>,
    chunk: Option<Writer>,
    chunk_dir: PathBuf,
    spec: hound::WavSpec,
    chunk_tx: mpsc::Sender<RecordedChunk>,
    chunk_index: u32,
    chunk_samples: u64,
    total_samples: u64,
}

impl CaptureState {
    fn new(
        path: PathBuf,
        chunk_dir: PathBuf,
        spec: hound::WavSpec,
        chunk_tx: mpsc::Sender<RecordedChunk>,
    ) -> Result<Self, String> {
        let full =
            hound::WavWriter::create(path, spec).map_err(|e| format!("无法创建录音文件：{e}"))?;
        let chunk = hound::WavWriter::create(chunk_path(&chunk_dir, 1), spec)
            .map_err(|e| format!("无法创建录音切片：{e}"))?;
        Ok(Self {
            full: Some(full),
            chunk: Some(chunk),
            chunk_dir,
            spec,
            chunk_tx,
            chunk_index: 1,
            chunk_samples: 0,
            total_samples: 0,
        })
    }

    fn write(&mut self, sample: i16) -> Result<(), String> {
        self.full
            .as_mut()
            .ok_or("录音写入器已关闭")?
            .write_sample(sample)
            .map_err(|e| format!("写入录音失败：{e}"))?;
        self.chunk
            .as_mut()
            .ok_or("录音切片写入器已关闭")?
            .write_sample(sample)
            .map_err(|e| format!("写入录音切片失败：{e}"))?;
        self.chunk_samples += 1;
        self.total_samples += 1;
        if self.chunk_samples >= chunk_sample_limit(self.spec.sample_rate, self.spec.channels) {
            self.finish_chunk()?;
            self.chunk = Some(
                hound::WavWriter::create(chunk_path(&self.chunk_dir, self.chunk_index), self.spec)
                    .map_err(|e| format!("无法创建录音切片：{e}"))?,
            );
        }
        Ok(())
    }

    fn finish(mut self) -> Result<(), String> {
        self.finish_chunk()?;
        if let Some(writer) = self.full.take() {
            writer
                .finalize()
                .map_err(|e| format!("无法完成录音文件：{e}"))?;
        }
        Ok(())
    }

    fn finish_chunk(&mut self) -> Result<(), String> {
        if self.chunk_samples == 0 {
            if let Some(writer) = self.chunk.take() {
                writer
                    .finalize()
                    .map_err(|e| format!("无法完成空录音切片：{e}"))?;
            }
            return Ok(());
        }
        let index = self.chunk_index;
        let path = chunk_path(&self.chunk_dir, index);
        if let Some(writer) = self.chunk.take() {
            writer
                .finalize()
                .map_err(|e| format!("无法完成录音切片：{e}"))?;
        }
        let samples_per_second = self.spec.sample_rate as f64 * self.spec.channels as f64;
        let duration_seconds = self.chunk_samples as f64 / samples_per_second;
        let start_seconds = (self.total_samples - self.chunk_samples) as f64 / samples_per_second;
        // A live-ASR failure must never interrupt or corrupt the full WAV.
        let _ = self.chunk_tx.send(RecordedChunk {
            path,
            start_seconds,
            duration_seconds,
            index,
        });
        self.chunk_index += 1;
        self.chunk_samples = 0;
        Ok(())
    }
}

fn chunk_path(dir: &Path, index: u32) -> PathBuf {
    dir.join(format!("recording-{index:04}.wav"))
}

pub fn chunk_sample_limit(sample_rate: u32, channels: u16) -> u64 {
    sample_rate as u64 * channels as u64 * LIVE_CHUNK_SECONDS
}

trait ToI16 {
    fn to_i16(self) -> i16;
}
impl ToI16 for f32 {
    fn to_i16(self) -> i16 {
        (self.clamp(-1.0, 1.0) * i16::MAX as f32) as i16
    }
}
impl ToI16 for i16 {
    fn to_i16(self) -> i16 {
        self
    }
}
impl ToI16 for u16 {
    fn to_i16(self) -> i16 {
        (self as i32 - 32768) as i16
    }
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    capture: Arc<Mutex<Option<CaptureState>>>,
    level_tx: mpsc::SyncSender<f32>,
    error_callback: impl FnMut(cpal::StreamError) + Send + 'static,
) -> Result<cpal::Stream, String>
where
    T: cpal::SizedSample + ToI16,
{
    device
        .build_input_stream(
            config,
            move |samples: &[T], _| {
                let mut peak = 0.0f32;
                let mut guard = capture.lock().expect("recording capture poisoned");
                let Some(capture) = guard.as_mut() else {
                    return;
                };
                for sample in samples.iter().copied() {
                    let sample = sample.to_i16();
                    peak = peak.max(sample.unsigned_abs() as f32 / i16::MAX as f32);
                    if capture.write(sample).is_err() {
                        break;
                    }
                }
                let _ = level_tx.try_send(peak.clamp(0.0, 1.0));
            },
            error_callback,
            None,
        )
        .map_err(|e| format!("无法建立麦克风输入流：{e}"))
}

#[cfg(test)]
mod tests {
    use super::{chunk_sample_limit, CaptureState};

    #[test]
    fn live_transcription_rotates_after_fifteen_seconds_of_interleaved_samples() {
        assert_eq!(chunk_sample_limit(16_000, 1), 240_000);
        assert_eq!(chunk_sample_limit(48_000, 2), 1_440_000);
    }

    #[test]
    fn full_recording_continues_if_the_live_transcription_receiver_exits() {
        let root =
            std::env::temp_dir().join(format!("snapscribe-recording-{}", uuid::Uuid::new_v4()));
        let chunk_dir = root.join("chunks");
        std::fs::create_dir_all(&chunk_dir).unwrap();
        let (sender, receiver) = std::sync::mpsc::channel();
        drop(receiver);
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 1,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut state =
            CaptureState::new(root.join("recording.wav"), chunk_dir, spec, sender).unwrap();

        let result = (0..15).try_for_each(|_| state.write(100));

        assert!(result.is_ok());
        drop(state);
        std::fs::remove_dir_all(root).ok();
    }
}
