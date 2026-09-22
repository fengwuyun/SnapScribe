#[derive(Debug, Clone, PartialEq)]
pub struct DiarizationTurn {
    pub start: f64,
    pub end: f64,
    pub speaker: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiarizationOutput {
    pub speakers: usize,
    pub turns: Vec<DiarizationTurn>,
}

pub struct SpeakerAssignment {
    pub segments: Vec<TranscriptSegment>,
    pub speakers: Vec<TranscriptSpeaker>,
}

pub struct DiarizationApplied {
    pub assignment: SpeakerAssignment,
    pub state: DiarizationState,
}

pub fn apply_result(
    segments: &[TranscriptSegment],
    existing_speakers: &[TranscriptSpeaker],
    result: Result<DiarizationOutput, String>,
) -> DiarizationApplied {
    match result {
        Ok(output) => DiarizationApplied {
            assignment: assign_speakers(segments, &output.turns),
            state: DiarizationState {
                enabled: true,
                status: DiarizationStatus::Completed,
                error: None,
            },
        },
        Err(error) => DiarizationApplied {
            assignment: SpeakerAssignment {
                segments: segments.to_vec(),
                speakers: existing_speakers.to_vec(),
            },
            state: DiarizationState {
                enabled: true,
                status: DiarizationStatus::Failed,
                error: Some(error),
            },
        },
    }
}

pub fn provisionalize(
    mut assignment: SpeakerAssignment,
    start_number: usize,
) -> SpeakerAssignment {
    let mut ids = std::collections::HashMap::new();
    for (index, speaker) in assignment.speakers.iter_mut().enumerate() {
        let number = start_number + index;
        let next_id = format!("provisional-speaker-{number}");
        ids.insert(speaker.id.clone(), next_id.clone());
        speaker.id = next_id;
        speaker.name = format!("临时说话人 {number}");
        speaker.color_index = ((number - 1) % 8) as u8;
    }
    for segment in &mut assignment.segments {
        if let Some(next) = segment
            .speaker_id
            .as_deref()
            .and_then(|id| ids.get(id))
        {
            segment.speaker_id = Some(next.clone());
        }
    }
    assignment
}

pub fn assign_speakers(
    segments: &[TranscriptSegment],
    turns: &[DiarizationTurn],
) -> SpeakerAssignment {
    let mut stable_ids = std::collections::HashMap::<usize, String>::new();
    let mut speakers = Vec::new();
    let mut assigned = Vec::with_capacity(segments.len());

    for segment in segments {
        let mut overlaps = std::collections::HashMap::<usize, (f64, f64)>::new();
        for turn in turns {
            let overlap = segment.end.min(turn.end) - segment.start.max(turn.start);
            if overlap <= 0.0 {
                continue;
            }
            let entry = overlaps
                .entry(turn.speaker)
                .or_insert((0.0, turn.start));
            entry.0 += overlap;
            entry.1 = entry.1.min(turn.start);
        }
        let winner = overlaps.into_iter().max_by(|(speaker_a, a), (speaker_b, b)| {
            a.0.total_cmp(&b.0)
                .then_with(|| b.1.total_cmp(&a.1))
                .then_with(|| speaker_b.cmp(speaker_a))
        });
        let mut next = segment.clone();
        if let Some((raw_speaker, _)) = winner {
            let id = stable_ids.entry(raw_speaker).or_insert_with(|| {
                let number = speakers.len() + 1;
                let id = format!("speaker-{number}");
                speakers.push(TranscriptSpeaker {
                    id: id.clone(),
                    name: format!("说话人 {number}"),
                    color_index: ((number - 1) % 8) as u8,
                });
                id
            });
            next.speaker_id = Some(id.clone());
        }
        assigned.push(next);
    }

    SpeakerAssignment {
        segments: assigned,
        speakers,
    }
}

pub fn build_args(
    segmentation_model: &Path,
    embedding_model: &Path,
    audio: &Path,
) -> Vec<String> {
    vec![
        "--clustering.num-clusters=-1".to_string(),
        "--clustering.cluster-threshold=0.90".to_string(),
        "--segmentation.num-threads=2".to_string(),
        "--embedding.num-threads=2".to_string(),
        format!(
            "--segmentation.pyannote-model={}",
            segmentation_model.to_string_lossy()
        ),
        format!("--embedding.model={}", embedding_model.to_string_lossy()),
        audio.to_string_lossy().into_owned(),
    ]
}

pub fn spawn(runtime: &DiarizationRuntime, audio: &Path) -> Result<Child, String> {
    let mut command = Command::new(&runtime.exe);
    crate::proc::hide_console(&mut command);
    command
        .args(build_args(
            &runtime.segmentation_model,
            &runtime.embedding_model,
            audio,
        ))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("无法启动说话人识别进程：{error}"))
}

pub fn parse_cli_output(output: &str) -> Result<DiarizationOutput, String> {
    let mut turns = Vec::new();
    for line in output.lines().map(str::trim) {
        let Some((times, speaker_raw)) = line.rsplit_once(" speaker_") else {
            continue;
        };
        let Some((start_raw, end_raw)) = times.split_once(" -- ") else {
            return Err(format!("无法解析说话人时间区间：{line}"));
        };
        let start = start_raw
            .trim()
            .parse::<f64>()
            .map_err(|_| format!("无法解析说话人开始时间：{line}"))?;
        let end = end_raw
            .trim()
            .parse::<f64>()
            .map_err(|_| format!("无法解析说话人结束时间：{line}"))?;
        let speaker = speaker_raw
            .trim()
            .parse::<usize>()
            .map_err(|_| format!("无法解析说话人编号：{line}"))?;
        if !start.is_finite() || !end.is_finite() || start < 0.0 || end <= start {
            return Err(format!("说话人时间区间无效：{line}"));
        }
        turns.push(DiarizationTurn {
            start,
            end,
            speaker,
        });
    }
    if turns.is_empty() {
        return Err("说话人识别没有返回有效区间".to_string());
    }
    let speakers = turns
        .iter()
        .map(|turn| turn.speaker)
        .collect::<std::collections::HashSet<_>>()
        .len();
    Ok(DiarizationOutput { speakers, turns })
}

pub fn parse_progress(line: &str) -> Option<u32> {
    let value = line.trim().strip_prefix("progress ")?.strip_suffix('%')?;
    let percent = value.parse::<f64>().ok()?;
    percent.is_finite().then(|| percent.floor().clamp(0.0, 100.0) as u32)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{apply_result, assign_speakers, build_args, parse_cli_output, parse_progress, provisionalize, DiarizationTurn};
    use crate::model::DiarizationStatus;
    use crate::model::TranscriptSegment;

    fn segment(id: &str, start: f64, end: f64, text: &str) -> TranscriptSegment {
        TranscriptSegment {
            id: id.into(),
            start,
            end,
            text: text.into(),
            speaker_id: None,
        }
    }

    #[test]
    fn assign_speakers_uses_greatest_overlap_without_changing_text_or_time() {
        let original = segment("s1", 0.0, 10.0, "保持原文");
        let assignment = assign_speakers(
            std::slice::from_ref(&original),
            &[
                DiarizationTurn { start: 0.0, end: 6.0, speaker: 7 },
                DiarizationTurn { start: 6.0, end: 10.0, speaker: 3 },
            ],
        );

        assert_eq!(assignment.segments[0].speaker_id.as_deref(), Some("speaker-1"));
        assert_eq!(assignment.segments[0].id, original.id);
        assert_eq!(assignment.segments[0].start, original.start);
        assert_eq!(assignment.segments[0].end, original.end);
        assert_eq!(assignment.segments[0].text, original.text);
        assert_eq!(assignment.speakers[0].name, "说话人 1");
        assert_eq!(assignment.speakers[0].color_index, 0);
    }

    #[test]
    fn assign_speakers_resolves_equal_overlap_by_earliest_turn() {
        let assignment = assign_speakers(
            &[segment("s1", 0.0, 4.0, "平分")],
            &[
                DiarizationTurn { start: 2.0, end: 4.0, speaker: 1 },
                DiarizationTurn { start: 0.0, end: 2.0, speaker: 9 },
            ],
        );

        assert_eq!(assignment.segments[0].speaker_id.as_deref(), Some("speaker-1"));
        assert_eq!(assignment.speakers[0].id, "speaker-1");
    }

    #[test]
    fn assign_speakers_leaves_non_overlapping_segments_unlabeled() {
        let original = segment("s1", 10.0, 12.0, "没有重叠");
        let assignment = assign_speakers(
            std::slice::from_ref(&original),
            &[DiarizationTurn { start: 0.0, end: 2.0, speaker: 0 }],
        );

        assert_eq!(assignment.segments[0].speaker_id, None);
        assert_eq!(assignment.segments[0].text, original.text);
        assert!(assignment.speakers.is_empty());
    }

    #[test]
    fn diarization_failure_preserves_transcript_and_records_nonfatal_error() {
        let original = vec![segment("s1", 0.0, 2.0, "保留文本")];

        let result = apply_result(&original, &[], Err("model missing".into()));

        assert_eq!(result.assignment.segments[0].text, "保留文本");
        assert_eq!(result.assignment.segments[0].speaker_id, None);
        assert_eq!(result.state.status, DiarizationStatus::Failed);
        assert!(result.state.error.as_deref().unwrap().contains("model missing"));
    }

    #[test]
    fn provisionalize_allocates_unique_temporary_speakers_per_chunk() {
        let assignment = assign_speakers(
            &[segment("s1", 0.0, 2.0, "临时字幕")],
            &[DiarizationTurn { start: 0.0, end: 2.0, speaker: 0 }],
        );

        let provisional = provisionalize(assignment, 3);

        assert_eq!(provisional.speakers[0].id, "provisional-speaker-3");
        assert_eq!(provisional.speakers[0].name, "临时说话人 3");
        assert_eq!(provisional.segments[0].speaker_id.as_deref(), Some("provisional-speaker-3"));
    }

    #[test]
    fn builds_official_cli_arguments_for_automatic_speaker_count() {
        let args = build_args(
            Path::new("seg.onnx"),
            Path::new("embed.onnx"),
            Path::new("meeting.wav"),
        );

        assert_eq!(
            args,
            vec![
                "--clustering.num-clusters=-1",
                "--clustering.cluster-threshold=0.90",
                "--segmentation.num-threads=2",
                "--embedding.num-threads=2",
                "--segmentation.pyannote-model=seg.onnx",
                "--embedding.model=embed.onnx",
                "meeting.wav",
            ]
        );
    }

    #[test]
    fn parses_official_cli_turns_and_speaker_count() {
        let output = "Started\n0.638 -- 6.848 speaker_00\n7.017 -- 10.679 speaker_01\n11.472 -- 13.548 speaker_01\n";

        let parsed = parse_cli_output(output).unwrap();

        assert_eq!(parsed.speakers, 2);
        assert_eq!(parsed.turns.len(), 3);
        assert_eq!(parsed.turns[0].start, 0.638);
        assert_eq!(parsed.turns[0].end, 6.848);
        assert_eq!(parsed.turns[0].speaker, 0);
        assert_eq!(parsed.turns[2].speaker, 1);
    }

    #[test]
    fn rejects_invalid_or_empty_cli_results() {
        assert!(parse_cli_output("Started\n").is_err());
        assert!(parse_cli_output("2.0 -- 1.0 speaker_00\n").is_err());
        assert!(parse_cli_output("NaN -- 3.0 speaker_00\n").is_err());
    }

    #[test]
    fn parses_official_progress_lines() {
        assert_eq!(parse_progress("progress 41.33%"), Some(41));
        assert_eq!(parse_progress("Started"), None);
    }
}
use std::path::Path;
use std::process::{Child, Command, Stdio};

use crate::model::{
    DiarizationState, DiarizationStatus, TranscriptSegment, TranscriptSpeaker,
};
use crate::runtime::DiarizationRuntime;
