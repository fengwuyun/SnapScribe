use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ImportStrategy {
    Reference,
    Copy,
    Move,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AIServiceType {
    OpenAICompatible,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AISettings {
    pub service_type: AIServiceType,
    pub base_url: String,
    pub model: String,
    pub has_api_key: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub schema_version: u32,
    pub data_root: String,
    pub import_strategy: ImportStrategy,
    pub ai: AISettings,
    #[serde(default, skip_serializing)]
    pub api_key: Option<String>,
    #[serde(default, skip_serializing)]
    pub clear_api_key: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionResult {
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AIProtocol {
    OpenAIChat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AIEndpointMode {
    Auto,
    FullUrl,
    CustomPath,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AIAuthType {
    Bearer,
    XApiKey,
    CustomHeader,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AIModelStatusKind {
    Untested,
    Available,
    Timeout,
    AuthFailed,
    RateLimited,
    ServiceError,
    ConfigError,
    InvalidResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AIModelStatus {
    pub status: AIModelStatusKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checked_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_time_ms: Option<u64>,
}

impl Default for AIModelStatus {
    fn default() -> Self {
        Self {
            status: AIModelStatusKind::Untested,
            message: None,
            checked_at: None,
            response_time_ms: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AIModelConfig {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preset_key: Option<String>,
    pub name: String,
    pub protocol: AIProtocol,
    pub base_url: String,
    pub model_id: String,
    pub endpoint_mode: AIEndpointMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_path: Option<String>,
    pub auth_type: AIAuthType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_header: Option<String>,
    pub timeout_secs: u64,
    pub enabled: bool,
    pub order: u32,
    #[serde(default)]
    pub has_api_key: bool,
    #[serde(default)]
    pub last_status: AIModelStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AIModelDraft {
    #[serde(default)]
    pub id: Option<String>,
    pub name: String,
    pub protocol: AIProtocol,
    pub base_url: String,
    pub model_id: String,
    pub endpoint_mode: AIEndpointMode,
    #[serde(default)]
    pub custom_path: Option<String>,
    pub auth_type: AIAuthType,
    #[serde(default)]
    pub auth_header: Option<String>,
    pub timeout_secs: u64,
    pub enabled: bool,
    #[serde(default, skip_serializing)]
    pub api_key: Option<String>,
    #[serde(default, skip_serializing)]
    pub clear_api_key: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AIServiceConfig {
    pub schema_version: u32,
    pub custom_instruction: String,
    pub models: Vec<AIModelConfig>,
}

impl Default for AIServiceConfig {
    fn default() -> Self {
        Self {
            schema_version: 1,
            custom_instruction: String::new(),
            models: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AboutInfo {
    pub version: String,
    pub build_time: String,
    pub repository_url: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProjectStatus {
    Transcribing,
    Completed,
    Canceled,
    Failed,
}

#[cfg(test)]
impl ProjectStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Transcribing => "transcribing",
            Self::Completed => "completed",
            Self::Canceled => "canceled",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProjectSort {
    RecentlyUpdated,
    CreatedAt,
    Name,
    Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MediaOrigin {
    Imported,
    Recorded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MediaStorage {
    Reference,
    ManagedCopy,
    Recording,
}

#[cfg(test)]
impl MediaStorage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Reference => "reference",
            Self::ManagedCopy => "managedCopy",
            Self::Recording => "recording",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaReference {
    pub origin: MediaOrigin,
    pub storage: MediaStorage,
    pub path: Option<String>,
    pub original_file_name: String,
    pub size_bytes: u64,
    pub container: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptionProject {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub status: ProjectStatus,
    pub duration_secs: f64,
    pub media: MediaReference,
    pub transcript_revision: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary_revision: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptDocument {
    pub schema_version: u32,
    pub project_id: String,
    pub revision: u32,
    pub segments: Vec<TranscriptSegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionItem {
    pub id: String,
    pub text: String,
    pub completed: bool,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ActionItemCompat {
    Text(String),
    Item(ActionItem),
}

fn deserialize_action_items<'de, D>(deserializer: D) -> Result<Vec<ActionItem>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let items = Vec::<ActionItemCompat>::deserialize(deserializer)?;
    Ok(items.into_iter().map(|item| match item {
        ActionItemCompat::Text(text) => ActionItem {
            id: uuid::Uuid::new_v4().to_string(),
            text,
            completed: false,
        },
        ActionItemCompat::Item(item) => item,
    }).collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AISummary {
    pub schema_version: u32,
    pub project_id: String,
    pub source_transcript_revision: u32,
    pub generated_at: String,
    pub summary: String,
    pub key_points: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_action_items")]
    pub action_items: Vec<ActionItem>,
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_config_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDetail {
    pub project: TranscriptionProject,
    pub transcript: TranscriptDocument,
    pub summary: Option<AISummary>,
    pub media_available: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectListItem {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub status: ProjectStatus,
    pub duration_secs: f64,
    pub media: MediaReference,
    pub media_available: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartProjectResult {
    pub project_id: String,
    pub job_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingStartResult {
    pub recording_id: String,
    pub project_id: String,
    pub job_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingLevelEvent {
    pub recording_id: String,
    pub project_id: String,
    pub level: f32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptionJobStatus {
    pub project_id: String,
    pub job_id: String,
    pub running: bool,
    pub paused: bool,
}

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
    pub project_id: String,
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
    pub project_id: String,
    pub job_id: String,
    pub segments: Vec<TranscriptSegment>,
}

/// Payload of the `transcript://completed` event.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletedEvent {
    pub project_id: String,
    pub job_id: String,
    pub result: TranscriptResult,
}

/// Payload of the `transcript://failed` event.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FailedEvent {
    pub project_id: String,
    pub job_id: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CanceledEvent {
    pub project_id: String,
    pub job_id: String,
}

#[cfg(test)]
mod project_event_tests {
    use super::{AboutInfo, ProgressEvent, TranscriptionJobStatus};

    #[test]
    fn progress_event_serializes_project_and_job_scope() {
        let value = serde_json::to_value(ProgressEvent {
            project_id: "project-1".to_string(),
            job_id: "job-1".to_string(),
            stage: "transcribing",
            percent: 50,
            processed_seconds: 5.0,
            total_seconds: 10.0,
            segment_index: 1,
            segment_count: 2,
        })
        .unwrap();

        assert_eq!(value["projectId"], "project-1");
        assert_eq!(value["jobId"], "job-1");
    }

    #[test]
    fn active_job_status_exposes_pause_state_to_the_ui() {
        let value = serde_json::to_value(TranscriptionJobStatus {
            project_id: "project-1".to_string(),
            job_id: "job-1".to_string(),
            running: true,
            paused: true,
        })
        .unwrap();

        assert_eq!(value["projectId"], "project-1");
        assert_eq!(value["running"], true);
        assert_eq!(value["paused"], true);
    }

    #[test]
    fn about_info_exposes_version_build_time_and_repository() {
        let value = serde_json::to_value(AboutInfo {
            version: "0.1.0".to_string(),
            build_time: "2026-08-28".to_string(),
            repository_url: "https://github.com/fengwuyun/SnapScribe".to_string(),
        })
        .unwrap();

        assert_eq!(value["version"], "0.1.0");
        assert_eq!(value["buildTime"], "2026-08-28");
        assert_eq!(
            value["repositoryUrl"],
            "https://github.com/fengwuyun/SnapScribe"
        );
    }

    #[test]
    fn legacy_string_action_items_deserialize_as_editable_items() {
        let value = serde_json::json!({
            "schemaVersion": 1,
            "projectId": "p",
            "sourceTranscriptRevision": 1,
            "generatedAt": "now",
            "summary": "摘要",
            "keyPoints": [],
            "actionItems": ["跟进合同"],
            "model": "m"
        });
        let summary: super::AISummary = serde_json::from_value(value).unwrap();
        assert_eq!(summary.action_items[0].text, "跟进合同");
        assert!(!summary.action_items[0].completed);
        assert!(!summary.action_items[0].id.is_empty());
    }
}
