use std::path::Path;

use crate::model::TranscriptSegment;

/// Plain-text full transcript: one line per segment, UTF-8, no timestamps.
pub fn build_txt(segments: &[TranscriptSegment]) -> String {
    let mut out = String::new();
    for segment in segments {
        out.push_str(segment.text.trim());
        out.push('\n');
    }
    out
}

/// Standard SRT subtitle text built from globally-timed segments.
pub fn build_srt(segments: &[TranscriptSegment]) -> String {
    let mut out = String::new();
    for (index, segment) in segments.iter().enumerate() {
        out.push_str(&(index + 1).to_string());
        out.push('\n');
        out.push_str(&format!(
            "{} --> {}\n",
            srt_time(segment.start.max(0.0)),
            srt_time(segment.end.max(segment.start.max(0.0))),
        ));
        out.push_str(segment.text.trim());
        out.push_str("\n\n");
    }
    out
}

/// Format seconds as `HH:MM:SS,mmm` (SRT timestamp style).
pub fn srt_time(secs: f64) -> String {
    let total_ms = (secs * 1000.0).round() as u64;
    let h = total_ms / 3_600_000;
    let m = (total_ms % 3_600_000) / 60_000;
    let s = (total_ms % 60_000) / 1000;
    let ms = total_ms % 1000;
    format!("{h:02}:{m:02}:{s:02},{ms:03}")
}

/// Write text as UTF-8 (no BOM), creating parent directories on demand.
pub fn write_utf8(path: &Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("无法创建输出目录：{e}"))?;
    }
    std::fs::write(path, content.as_bytes()).map_err(|e| format!("写入文件失败：{e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seg(start: f64, end: f64, text: &str) -> TranscriptSegment {
        TranscriptSegment { id: "x".into(), start, end, text: text.into() }
    }

    #[test]
    fn txt_is_one_line_per_segment() {
        let txt = build_txt(&[seg(0.0, 1.0, "第一句"), seg(2.0, 3.0, "第二句")]);
        assert_eq!(txt, "第一句\n第二句\n");
    }

    #[test]
    fn srt_uses_comma_millis_and_global_times() {
        let srt = build_srt(&[seg(3661.5, 3665.25, "内容")]);
        assert_eq!(srt, "1\n01:01:01,500 --> 01:01:05,250\n内容\n\n");
    }

    #[test]
    fn srt_time_covers_boundaries() {
        assert_eq!(srt_time(0.0), "00:00:00,000");
        assert_eq!(srt_time(59.9994), "00:00:59,999");
        assert_eq!(srt_time(7225.0), "02:00:25,000");
    }
}
