use std::path::Path;
use std::process::{Child, Command, Stdio};

use crate::model::TranscriptSegment;

/// One parsed SRT cue: seconds relative to its own segment file.
#[derive(Debug, Clone, PartialEq)]
pub struct SrtCue {
    pub start: f64,
    pub end: f64,
    pub text: String,
}

/// Arguments for one `llama-funasr-sensevoice` invocation (README of
/// funasr-llamacpp): model + FSMN-VAD for in-segment timestamp alignment.
pub fn build_args(asr_exe_model: &Path, vad_model: &Path, audio: &Path) -> Vec<String> {
    [
        "-m",
        asr_exe_model.to_string_lossy().as_ref(),
        "--vad",
        vad_model.to_string_lossy().as_ref(),
        "-a",
        audio.to_string_lossy().as_ref(),
        "--backend",
        "cpu",
        "--srt",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

/// Spawn the ASR process for one segment. The caller owns the child so that
/// cancellation can kill it mid-run.
pub fn spawn(asr_exe: &Path, asr_model: &Path, vad_model: &Path, audio: &Path) -> Result<Child, String> {
    Command::new(asr_exe)
        .args(build_args(asr_model, vad_model, audio))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("无法启动识别进程：{e}"))
}

/// Parse SRT text into cues. Malformed blocks are skipped; blank text blocks are dropped.
pub fn parse_srt(srt: &str) -> Vec<SrtCue> {
    srt.replace("\r\n", "\n")
        .split("\n\n")
        .filter_map(parse_block)
        .collect()
}

fn parse_block(block: &str) -> Option<SrtCue> {
    let lines = block.lines().map(str::trim).filter(|l| !l.is_empty());
    // Block layout: optional numeric index, "start --> end", then text lines.
    let time_line = lines.clone().find(|l| l.contains("-->"))?;
    let (start_raw, end_raw) = time_line.split_once("-->")?;
    let start = parse_srt_time(start_raw.trim())?;
    let end = parse_srt_time(end_raw.trim())?;
    let text: String = lines
        .filter(|l| !l.contains("-->") && l.parse::<u64>().is_err())
        .collect::<Vec<_>>()
        .join(" ");
    let text = strip_special_tags(text.trim());
    if text.is_empty() || end < start {
        return None;
    }
    Some(SrtCue { start, end, text })
}

/// Parse `HH:MM:SS,mmm` (or `HH:MM:SS.mmm`) into seconds.
fn parse_srt_time(raw: &str) -> Option<f64> {
    let (clock, millis) = match raw.split_once(',') {
        Some((c, m)) => (c, m),
        None => raw.split_once('.')?,
    };
    let mut parts = clock.split(':');
    let h: f64 = parts.next()?.parse().ok()?;
    let m: f64 = parts.next()?.parse().ok()?;
    let s: f64 = parts.next()?.parse().ok()?;
    if !(0.0..60.0).contains(&m) || !(0.0..60.0).contains(&s) {
        return None;
    }
    let ms: f64 = millis.parse().ok()?;
    Some(h * 3600.0 + m * 60.0 + s + ms / 1000.0)
}

/// Remove SenseVoice control tags such as `<|zh|><|EMO_UNKNOWN|>` defensively,
/// in case a runtime build emits them even without `--keep-tags`.
fn strip_special_tags(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if text[i..].starts_with("<|") {
            if let Some(rel) = text[i + 2..].find("|>") {
                i += 2 + rel + 2;
                continue;
            }
        }
        let ch_len = utf8_len(bytes[i]);
        out.push_str(&text[i..i + ch_len]);
        i += ch_len;
    }
    out.trim().to_string()
}

fn utf8_len(first_byte: u8) -> usize {
    match first_byte {
        0x00..=0x7F => 1,
        0xC0..=0xDF => 2,
        0xE0..=0xEF => 3,
        _ => 4,
    }
}

/// Convert parsed cues of one segment file into globally-timed transcript
/// segments by adding `offset_seconds` (the fixed window's position).
pub fn offset_cues(
    cues: &[SrtCue],
    offset_seconds: f64,
    segment_index: usize,
) -> Vec<TranscriptSegment> {
    cues.iter()
        .enumerate()
        .map(|(i, cue)| TranscriptSegment {
            id: format!("seg-{segment_index:03}-{i:03}"),
            start: offset_seconds + cue.start,
            end: offset_seconds + cue.end,
            text: cue.text.clone(),
        })
        .collect()
}

/// Read both pipes of the child to completion plus its exit status.
///
/// Two threads avoid the classic pipe-buffer deadlock when stderr produces
/// progress diagnostics while stdout is being consumed.
pub struct AsrOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

pub fn collect_output(mut child: Child) -> Result<AsrOutput, String> {
    use std::io::Read;
    let mut stdout_pipe = child.stdout.take().ok_or("缺少 stdout 管道")?;
    let mut stderr_pipe = child.stderr.take().ok_or("缺少 stderr 管道")?;
    let t = std::thread::spawn(move || {
        let mut buf = String::new();
        let _ = stderr_pipe.read_to_string(&mut buf);
        buf
    });
    let mut out = String::new();
    stdout_pipe
        .read_to_string(&mut out)
        .map_err(|e| format!("读取识别输出失败：{e}"))?;
    let err = t.join().map_err(|_| "stderr 读取线程崩溃".to_string())?;
    let status = child.wait().map_err(|e| format!("等待识别进程失败：{e}"))?;
    Ok(AsrOutput {
        success: status.success(),
        stdout: out,
        stderr: err,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_standard_srt_blocks() {
        let srt = "1\r\n00:00:00,000 --> 00:00:05,320\r\n今天我们讨论一下这个项目。\r\n\r\n2\r\n00:00:05,320 --> 00:00:12,100\r\n首先看一下当前的开发进度。\r\n";
        let cues = parse_srt(srt);
        assert_eq!(cues.len(), 2);
        assert_eq!(cues[0], SrtCue {
            start: 0.0,
            end: 5.32,
            text: "今天我们讨论一下这个项目。".into(),
        });
        assert!((cues[1].end - 12.1).abs() < 1e-9);
    }

    #[test]
    fn skips_malformed_blocks_and_blank_text() {
        let srt = "garbage block without arrow\n\n1\n00:00:01,000 --> bad\nbroken\n\n2\n00:00:02,000 --> 00:00:03,000\n<|zh|><|NEUTRAL|>实际内容\n";
        let cues = parse_srt(srt);
        assert_eq!(cues.len(), 1);
        assert_eq!(cues[0].text, "实际内容");
        assert!((cues[0].start - 2.0).abs() < 1e-9);
    }

    #[test]
    fn offsets_cues_into_global_timeline() {
        let cues = vec![
            SrtCue { start: 0.0, end: 3.5, text: "a".into() },
            SrtCue { start: 4.0, end: 6.0, text: "b".into() },
        ];
        let segments = offset_cues(&cues, 120.0, 2);
        assert_eq!(segments[0].id, "seg-002-000");
        assert_eq!(segments[0].start, 120.0);
        assert_eq!(segments[1].end, 126.0);
    }

    #[test]
    fn builds_expected_cli_arguments() {
        let args = build_args(Path::new("m.gguf"), Path::new("vad.gguf"), Path::new("seg_0001.wav"));
        let joined = args.join(" ");
        assert!(joined.contains("-m m.gguf"));
        assert!(joined.contains("--vad vad.gguf"));
        assert!(joined.contains("-a seg_0001.wav"));
        assert!(joined.ends_with("--srt"));
    }
}
