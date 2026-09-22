use std::path::Path;

use crate::model::{AISummary, TranscriptDocument, TranscriptSegment};

/// Plain-text full transcript: one line per segment, UTF-8, no timestamps.
pub fn build_txt(document: &TranscriptDocument) -> String {
    build_txt_with_speakers(&document.segments, &document.speakers)
}

pub fn build_txt_segments(segments: &[TranscriptSegment]) -> String {
    build_txt_with_speakers(segments, &[])
}

fn build_txt_with_speakers(
    segments: &[TranscriptSegment],
    speakers: &[crate::model::TranscriptSpeaker],
) -> String {
    let mut out = String::new();
    for segment in segments {
        if let Some(name) = segment
            .speaker_id
            .as_deref()
            .and_then(|id| speakers.iter().find(|speaker| speaker.id == id))
            .map(|speaker| speaker.name.trim())
            .filter(|name| !name.is_empty())
        {
            out.push_str(&format!("[{}] {name}：", clock_time(segment.start)));
        }
        out.push_str(segment.text.trim());
        out.push('\n');
    }
    out
}

/// Standard SRT subtitle text built from globally-timed segments.
pub fn build_srt(document: &TranscriptDocument) -> String {
    build_srt_with_speakers(&document.segments, &document.speakers)
}

pub fn build_srt_segments(segments: &[TranscriptSegment]) -> String {
    build_srt_with_speakers(segments, &[])
}

fn build_srt_with_speakers(
    segments: &[TranscriptSegment],
    speakers: &[crate::model::TranscriptSpeaker],
) -> String {
    let mut out = String::new();
    for (index, segment) in segments.iter().enumerate() {
        out.push_str(&(index + 1).to_string());
        out.push('\n');
        out.push_str(&format!(
            "{} --> {}\n",
            srt_time(segment.start.max(0.0)),
            srt_time(segment.end.max(segment.start.max(0.0))),
        ));
        if let Some(name) = segment
            .speaker_id
            .as_deref()
            .and_then(|id| speakers.iter().find(|speaker| speaker.id == id))
            .map(|speaker| speaker.name.trim())
            .filter(|name| !name.is_empty())
        {
            out.push_str(name);
            out.push('：');
        }
        out.push_str(segment.text.trim());
        out.push_str("\n\n");
    }
    out
}

fn clock_time(secs: f64) -> String {
    let total = secs.max(0.0).floor() as u64;
    let hours = total / 3600;
    let minutes = (total % 3600) / 60;
    let seconds = total % 60;
    if hours > 0 {
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    }
}

pub fn build_summary_txt(summary: &AISummary) -> String {
    let key_points = summary
        .key_points
        .iter()
        .map(|item| format!("- {}", item.trim()))
        .collect::<Vec<_>>()
        .join("\n");
    let action_items = summary
        .action_items
        .iter()
        .map(|item| format!("- {}", item.text.trim()))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "摘要\n{}\n\n关键要点\n{}\n\n待办事项\n{}",
        summary.summary.trim(),
        key_points,
        action_items,
    )
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
        TranscriptSegment {
            id: "x".into(),
            start,
            end,
            text: text.into(),
            speaker_id: None,
        }
    }

    #[test]
    fn speaker_aware_exports_use_display_names() {
        let document = crate::model::TranscriptDocument {
            schema_version: 2,
            project_id: "p1".into(),
            revision: 1,
            speakers: vec![crate::model::TranscriptSpeaker {
                id: "speaker-1".into(),
                name: "刘德华".into(),
                color_index: 0,
            }],
            segments: vec![crate::model::TranscriptSegment {
                id: "s1".into(),
                start: 17.0,
                end: 20.0,
                text: "这里是第一段。".into(),
                speaker_id: Some("speaker-1".into()),
            }],
        };

        assert_eq!(build_txt(&document), "[00:17] 刘德华：这里是第一段。\n");
        assert_eq!(
            build_srt(&document),
            "1\n00:00:17,000 --> 00:00:20,000\n刘德华：这里是第一段。\n\n"
        );
    }

    #[test]
    fn txt_is_one_line_per_segment() {
        let txt = build_txt_segments(&[seg(0.0, 1.0, "第一句"), seg(2.0, 3.0, "第二句")]);
        assert_eq!(txt, "第一句\n第二句\n");
    }

    #[test]
    fn srt_uses_comma_millis_and_global_times() {
        let srt = build_srt_segments(&[seg(3661.5, 3665.25, "内容")]);
        assert_eq!(srt, "1\n01:01:01,500 --> 01:01:05,250\n内容\n\n");
    }

    #[test]
    fn srt_time_covers_boundaries() {
        assert_eq!(srt_time(0.0), "00:00:00,000");
        assert_eq!(srt_time(59.9994), "00:00:59,999");
        assert_eq!(srt_time(7225.0), "02:00:25,000");
    }

    #[test]
    fn summary_txt_contains_all_three_sections() {
        let summary = AISummary {
            schema_version: 1,
            project_id: "p1".into(),
            source_transcript_revision: 1,
            generated_at: "1".into(),
            summary: "结论".into(),
            key_points: vec!["要点一".into(), "要点二".into()],
            action_items: vec![crate::model::ActionItem { id: "a".into(), text: "跟进".into(), completed: false }],
            model: "model".into(),
            model_config_id: None,
            model_name: None,
        };
        assert_eq!(
            build_summary_txt(&summary),
            "摘要\n结论\n\n关键要点\n- 要点一\n- 要点二\n\n待办事项\n- 跟进"
        );
    }
}
