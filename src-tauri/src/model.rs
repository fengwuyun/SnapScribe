use serde::{Deserialize, Serialize};

/// Unified transcript segment handed to the frontend (DEVELOPMENT_SPEC §5).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptSegment {
    pub id: String,
    /// Segment start in seconds from the beginning of the source media.
    pub start: f64,
    pub end: f64,
    pub text: String,
}

/// Full transcription result for one media file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptResult {
    pub file_name: String,
    pub duration: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    pub segments: Vec<TranscriptSegment>,
}

/// Media metadata returned by `get_media_info`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaInfo {
    pub file_name: String,
    pub size_bytes: u64,
    pub duration_secs: f64,
    pub container: String,
}

/// One entry of the history directory listing.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub file_name: String,
    pub size_bytes: u64,
    pub modified_ms: u64,
}

/// Payload of the `transcript://progress` event.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressEvent {
    pub job_id: String,
    pub stage: &'static str,
    pub percent: u32,
    pub processed_seconds: f64,
    pub total_seconds: f64,
    pub segment_index: u32,
    pub segment_count: u32,
}

/// Payload of the `transcript://segments` event: segments finished since the previous emit.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SegmentsEvent {
    pub job_id: String,
    pub segments: Vec<TranscriptSegment>,
}

/// Payload of the `transcript://completed` event.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletedEvent {
    pub result: TranscriptResult,
}

/// Payload of the `transcript://failed` event.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FailedEvent {
    pub message: String,
}
