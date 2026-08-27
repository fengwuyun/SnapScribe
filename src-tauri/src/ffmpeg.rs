use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Fixed segment window (ADR-002): a constant, not a user setting.
pub const SEGMENT_SECONDS: f64 = 60.0;

/// Build the one-pass ffmpeg arguments that extract the audio track, convert it
/// to 16 kHz mono 16-bit PCM and split it into fixed-length WAV segments.
pub fn build_segment_args(input: &Path, seg_pattern: &Path) -> Vec<String> {
    [
        "-hide_banner",
        "-loglevel",
        "error",
        "-y",
        "-i",
        input.to_string_lossy().as_ref(),
        "-vn",
        "-ac",
        "1",
        "-ar",
        "16000",
        "-c:a",
        "pcm_s16le",
        "-f",
        "segment",
        "-segment_time",
        &SEGMENT_SECONDS.to_string(),
        seg_pattern.to_string_lossy().as_ref(),
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

/// Run the extract+split conversion and return the produced segment files in
/// playback order.
pub fn run_extract_split(ffmpeg: &Path, input: &Path, temp_dir: &Path) -> Result<Vec<PathBuf>, String> {
    std::fs::create_dir_all(temp_dir).map_err(|e| format!("无法创建临时目录：{e}"))?;
    let pattern = temp_dir.join("seg_%04d.wav");

    let args = build_segment_args(input, &pattern);
    let mut cmd = Command::new(ffmpeg);
    crate::proc::hide_console(&mut cmd);
    let output = cmd
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("无法启动 ffmpeg：{e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let detail: String = stderr.lines().take(3).collect::<Vec<_>>().join(" | ");
        return Err(format!("音频提取/切片失败：{detail}"));
    }

    let mut segments: Vec<PathBuf> = list_segments(temp_dir)?;
    if segments.is_empty() {
        return Err("音频切片未产生任何分段，文件可能没有有效音轨".to_string());
    }
    // The segment muxer numbers files sequentially; a plain sort keeps them in
    // order because every name shares the `seg_` prefix and 4-digit padding.
    segments.sort();
    Ok(segments)
}

/// List `seg_*.wav` files directly inside `dir`.
fn list_segments(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("无法读取临时目录：{e}"))?;
    Ok(entries
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            let is_seg_wav = path.file_name()?.to_str()?.starts_with("seg_")
                && path.extension()?.eq_ignore_ascii_case("wav");
            is_seg_wav.then_some(path)
        })
        .collect())
}

/// Duration of a PCM WAV file computed from its header (`data` size / byte rate).
///
/// Avoids one ffprobe call per segment. Only supports the RIFF layout ffmpeg
/// produces for `-c:a pcm_s16le`.
pub fn wav_duration_seconds(path: &Path) -> Result<f64, String> {
    let mut file = std::fs::File::open(path).map_err(|e| format!("无法打开分段音频：{e}"))?;
    let mut riff = [0u8; 12];
    file.read_exact(&mut riff)
        .map_err(|_| "WAV 文件过短".to_string())?;
    if &riff[0..4] != b"RIFF" || &riff[8..12] != b"WAVE" {
        return Err("不是有效的 WAV 文件".to_string());
    }

    let mut byte_rate: Option<u32> = None;
    loop {
        let mut header = [0u8; 8];
        match file.read_exact(&mut header) {
            Ok(()) => {}
            Err(_) => break,
        }
        let id = &header[0..4];
        let size = u32::from_le_bytes([header[4], header[5], header[6], header[7]]) as usize;
        if id == b"fmt " {
            let mut fmt = vec![0u8; size];
            file.read_exact(&mut fmt)
                .map_err(|_| "WAVE fmt 块不完整".to_string())?;
            if fmt.len() >= 16 {
                byte_rate = Some(u32::from_le_bytes([
                    fmt[8], fmt[9], fmt[10], fmt[11],
                ]));
            }
        } else if id == b"data" {
            let rate = byte_rate.filter(|r| *r > 0).ok_or("WAV 缺少有效 fmt 块")?;
            return Ok(size as f64 / rate as f64);
        } else {
            let skip = size + (size & 1); // chunks are word-aligned
            std::io::copy(
                &mut file.by_ref().take(skip as u64),
                &mut std::io::sink(),
            )
            .map_err(|_| "WAV 块读取失败".to_string())?;
        }
    }
    Err("WAV 缺少 data 块".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_minimal_wav(path: &Path, sample_count: usize) {
        let sample_rate: u32 = 16000;
        let channels: u16 = 1;
        let bits: u16 = 16;
        let block_align: u32 = u32::from(channels) * u32::from(bits) / 8;
        let byte_rate = sample_rate * block_align;
        let data = vec![0u8; sample_count * block_align as usize];

        let mut bytes: Vec<u8> = Vec::new();
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&(36 + data.len() as u32).to_le_bytes());
        bytes.extend_from_slice(b"WAVE");
        bytes.extend_from_slice(b"fmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes()); // PCM
        bytes.extend_from_slice(&channels.to_le_bytes());
        bytes.extend_from_slice(&sample_rate.to_le_bytes());
        bytes.extend_from_slice(&byte_rate.to_le_bytes());
        bytes.extend_from_slice(&(block_align as u16).to_le_bytes());
        bytes.extend_from_slice(&bits.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&(data.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&data);
        std::fs::write(path, bytes).unwrap();
    }

    #[test]
    fn segment_args_use_pcm_and_fixed_window() {
        let args = build_segment_args(Path::new("C:/in/meeting.mp4"), Path::new("T/t/seg_%04d.wav"));
        let joined = args.join(" ");
        assert!(joined.contains("-ac 1"));
        assert!(joined.contains("-ar 16000"));
        assert!(joined.contains("-c:a pcm_s16le"));
        assert!(joined.contains("-segment_time 60"));
        assert!(joined.ends_with("seg_%04d.wav"));
    }

    #[test]
    fn wav_duration_computes_data_over_byte_rate() {
        let dir = std::env::temp_dir().join(format!("snapscribe-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("seg_test.wav");
        // One second of mono 16 kHz 16-bit audio.
        write_minimal_wav(&path, 16000);
        let secs = wav_duration_seconds(&path).unwrap();
        assert!((secs - 1.0).abs() < 1e-6);
        std::fs::remove_file(&path).ok();
    }
}
