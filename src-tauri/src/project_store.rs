use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

use crate::model::{
    ActionItem, AISummary, ImportStrategy, MediaInfo, MediaOrigin, MediaReference, MediaStorage, ProjectDetail,
    ProjectListItem, ProjectSort, ProjectStatus, TranscriptDocument, TranscriptionProject,
};

#[derive(Clone)]
pub struct ProjectStore {
    root: PathBuf,
}

impl ProjectStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn project_dir(&self, project_id: &str) -> PathBuf {
        self.root.join("projects").join(project_id)
    }

    pub fn create_imported_project(
        &self,
        source: &Path,
        info: &MediaInfo,
        strategy: ImportStrategy,
    ) -> Result<TranscriptionProject, String> {
        if !source.is_file() {
            return Err(format!("文件不存在：{}", source.display()));
        }
        let id = uuid::Uuid::new_v4().to_string();
        let dir = self.project_dir(&id);
        std::fs::create_dir_all(&dir).map_err(|e| format!("无法创建项目目录：{e}"))?;

        let should_remove_source = strategy == ImportStrategy::Move;
        let (storage, stored_path) = match strategy {
            ImportStrategy::Reference => (
                MediaStorage::Reference,
                source.to_string_lossy().to_string(),
            ),
            ImportStrategy::Copy | ImportStrategy::Move => {
                let target = dir.join(safe_file_name(&info.file_name));
                let temp = dir.join(format!(".media-{}.tmp", uuid::Uuid::new_v4()));
                let mut input = File::open(source).map_err(|e| format!("无法读取源媒体：{e}"))?;
                let mut output = File::create(&temp).map_err(|e| format!("无法创建目标媒体：{e}"))?;
                std::io::copy(&mut input, &mut output).map_err(|e| format!("复制媒体失败：{e}"))?;
                output.sync_all().map_err(|e| format!("无法写入目标媒体：{e}"))?;
                drop(output);
                let copied_size = std::fs::metadata(&temp).map_err(|e| format!("无法校验目标媒体：{e}"))?.len();
                if copied_size != info.size_bytes {
                    std::fs::remove_file(&temp).ok();
                    return Err("目标媒体大小校验失败".to_string());
                }
                std::fs::rename(&temp, &target).map_err(|e| format!("无法启用目标媒体：{e}"))?;
                (
                    MediaStorage::ManagedCopy,
                    target.to_string_lossy().to_string(),
                )
            }
        };

        let now = utc_timestamp();
        let project = TranscriptionProject {
            schema_version: 1,
            id: id.clone(),
            name: source
                .file_stem()
                .and_then(|value| value.to_str())
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("未命名转录")
                .to_string(),
            created_at: now.clone(),
            updated_at: now,
            status: ProjectStatus::Transcribing,
            duration_secs: info.duration_secs,
            media: MediaReference {
                origin: MediaOrigin::Imported,
                storage,
                path: Some(stored_path),
                original_file_name: info.file_name.clone(),
                size_bytes: info.size_bytes,
                container: info.container.clone(),
            },
            transcript_revision: 0,
            summary_revision: None,
            error: None,
        };
        let transcript = TranscriptDocument {
            schema_version: 1,
            project_id: id,
            revision: 0,
            segments: Vec::new(),
        };
        atomic_write_json(&dir.join("project.json"), &project)?;
        atomic_write_json(&dir.join("transcript.json"), &transcript)?;
        if should_remove_source {
            std::fs::remove_file(source).map_err(|e| format!("媒体已保存到 SnapScribe，但无法删除原文件：{e}"))?;
        }
        Ok(project)
    }

    pub fn create_recording_project(&self) -> Result<TranscriptionProject, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let dir = self.project_dir(&id);
        let media_dir = dir.join("media");
        std::fs::create_dir_all(&media_dir).map_err(|e| format!("无法创建录音目录：{e}"))?;
        let path = media_dir.join("recording.wav");
        let now = utc_timestamp();
        let project = TranscriptionProject {
            schema_version: 1,
            id: id.clone(),
            name: format!("录音 {now}"),
            created_at: now.clone(),
            updated_at: now,
            status: ProjectStatus::Transcribing,
            duration_secs: 0.0,
            media: MediaReference {
                origin: MediaOrigin::Recorded,
                storage: MediaStorage::Recording,
                path: Some(path.to_string_lossy().to_string()),
                original_file_name: "recording.wav".to_string(),
                size_bytes: 0,
                container: "wav".to_string(),
            },
            transcript_revision: 0,
            summary_revision: None,
            error: None,
        };
        atomic_write_json(&dir.join("project.json"), &project)?;
        atomic_write_json(
            &dir.join("transcript.json"),
            &TranscriptDocument {
                schema_version: 1,
                project_id: id,
                revision: 0,
                segments: Vec::new(),
            },
        )?;
        Ok(project)
    }

    pub fn create_legacy_project(
        &self,
        name: &str,
        text: &str,
    ) -> Result<TranscriptionProject, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let dir = self.project_dir(&id);
        std::fs::create_dir_all(&dir).map_err(|e| format!("无法创建旧记录项目：{e}"))?;
        let now = utc_timestamp();
        let project = TranscriptionProject {
            schema_version: 1,
            id: id.clone(),
            name: validate_project_name(name)?,
            created_at: now.clone(),
            updated_at: now,
            status: ProjectStatus::Completed,
            duration_secs: 0.0,
            media: MediaReference {
                origin: MediaOrigin::Imported,
                storage: MediaStorage::Reference,
                path: None,
                original_file_name: String::new(),
                size_bytes: 0,
                container: String::new(),
            },
            transcript_revision: 1,
            summary_revision: None,
            error: None,
        };
        let transcript = TranscriptDocument {
            schema_version: 1,
            project_id: id,
            revision: 1,
            segments: vec![crate::model::TranscriptSegment {
                id: "legacy-1".to_string(),
                start: 0.0,
                end: 0.0,
                text: text.to_string(),
            }],
        };
        atomic_write_json(&dir.join("project.json"), &project)?;
        atomic_write_json(&dir.join("transcript.json"), &transcript)?;
        Ok(project)
    }

    pub fn finalize_recording(
        &self,
        project_id: &str,
        info: &MediaInfo,
    ) -> Result<TranscriptionProject, String> {
        let mut project = self.read_project(project_id)?;
        project.media.size_bytes = info.size_bytes;
        project.media.container = info.container.clone();
        project.media.original_file_name = info.file_name.clone();
        project.duration_secs = info.duration_secs;
        project.updated_at = utc_timestamp();
        atomic_write_json(&self.project_dir(project_id).join("project.json"), &project)?;
        Ok(project)
    }

    pub fn read_project(&self, project_id: &str) -> Result<TranscriptionProject, String> {
        validate_project_id(project_id)?;
        read_json(&self.project_dir(project_id).join("project.json"), "项目")
    }

    pub fn write_transcript(
        &self,
        project_id: &str,
        transcript: &TranscriptDocument,
    ) -> Result<(), String> {
        validate_project_id(project_id)?;
        if transcript.project_id != project_id {
            return Err("转录文档与项目不匹配".to_string());
        }
        atomic_write_json(
            &self.project_dir(project_id).join("transcript.json"),
            transcript,
        )
    }

    pub fn read_detail(&self, project_id: &str) -> Result<ProjectDetail, String> {
        let project = self.read_project(project_id)?;
        let transcript: TranscriptDocument = read_json(
            &self.project_dir(project_id).join("transcript.json"),
            "转录文档",
        )?;
        let media_available = project
            .media
            .path
            .as_deref()
            .is_some_and(|path| Path::new(path).is_file());
        let summary_path = self.project_dir(project_id).join("summary.json");
        let summary = if summary_path.is_file() {
            Some(read_json(&summary_path, "AI 总结")?)
        } else {
            None
        };
        Ok(ProjectDetail {
            project,
            transcript,
            summary,
            media_available,
        })
    }

    pub fn list_projects(
        &self,
        query: &str,
        sort: ProjectSort,
    ) -> Result<Vec<ProjectListItem>, String> {
        let projects_dir = self.root.join("projects");
        let Ok(entries) = std::fs::read_dir(&projects_dir) else {
            return Ok(Vec::new());
        };
        let needle = query.trim().to_lowercase();
        let mut items = Vec::new();
        for entry in entries.flatten() {
            if !entry.path().is_dir() {
                continue;
            }
            let Some(id) = entry.file_name().to_str().map(ToOwned::to_owned) else {
                continue;
            };
            let Ok(project) = self.read_project(&id) else {
                continue;
            };
            let transcript: TranscriptDocument =
                read_json(&self.project_dir(&id).join("transcript.json"), "转录文档")?;
            let transcript_text = transcript
                .segments
                .iter()
                .map(|segment| segment.text.as_str())
                .collect::<Vec<_>>()
                .join("\n")
                .to_lowercase();
            if !needle.is_empty()
                && !project.name.to_lowercase().contains(&needle)
                && !transcript_text.contains(&needle)
            {
                continue;
            }
            let media_available = project
                .media
                .path
                .as_deref()
                .is_some_and(|path| Path::new(path).is_file());
            items.push(ProjectListItem {
                id: project.id,
                name: project.name,
                created_at: project.created_at,
                updated_at: project.updated_at,
                status: project.status,
                duration_secs: project.duration_secs,
                media: project.media,
                media_available,
            });
        }
        match sort {
            ProjectSort::RecentlyUpdated => {
                items.sort_by(|a, b| b.updated_at.cmp(&a.updated_at).then(a.name.cmp(&b.name)))
            }
            ProjectSort::CreatedAt => {
                items.sort_by(|a, b| b.created_at.cmp(&a.created_at).then(a.name.cmp(&b.name)))
            }
            ProjectSort::Name => items.sort_by(|a, b| a.name.cmp(&b.name)),
            ProjectSort::Duration => items.sort_by(|a, b| {
                b.duration_secs
                    .total_cmp(&a.duration_secs)
                    .then(a.name.cmp(&b.name))
            }),
        }
        Ok(items)
    }

    pub fn delete_project(&self, project_id: &str) -> Result<(), String> {
        validate_project_id(project_id)?;
        let dir = self.project_dir(project_id);
        if !dir.is_dir() {
            return Err("项目不存在".to_string());
        }
        std::fs::remove_dir_all(dir).map_err(|e| format!("删除项目失败：{e}"))
    }

    pub fn complete_transcription(
        &self,
        project_id: &str,
        segments: &[crate::model::TranscriptSegment],
    ) -> Result<(), String> {
        let mut project = self.read_project(project_id)?;
        let next_revision = project.transcript_revision.saturating_add(1);
        let transcript = TranscriptDocument {
            schema_version: 1,
            project_id: project_id.to_string(),
            revision: next_revision,
            segments: segments.to_vec(),
        };
        self.write_transcript(project_id, &transcript)?;
        project.status = ProjectStatus::Completed;
        project.transcript_revision = next_revision;
        project.updated_at = utc_timestamp();
        project.error = None;
        atomic_write_json(&self.project_dir(project_id).join("project.json"), &project)
    }

    pub fn cancel_transcription(&self, project_id: &str) -> Result<(), String> {
        let mut project = self.read_project(project_id)?;
        let mut transcript: TranscriptDocument = read_json(
            &self.project_dir(project_id).join("transcript.json"),
            "转录文档",
        )?;
        let revision = project.transcript_revision.saturating_add(1);
        transcript.revision = revision;
        self.write_transcript(project_id, &transcript)?;
        project.status = ProjectStatus::Canceled;
        project.transcript_revision = revision;
        project.updated_at = utc_timestamp();
        project.error = None;
        atomic_write_json(&self.project_dir(project_id).join("project.json"), &project)
    }

    pub fn rename_project(
        &self,
        project_id: &str,
        name: &str,
    ) -> Result<TranscriptionProject, String> {
        let name = validate_project_name(name)?;
        let mut project = self.read_project(project_id)?;
        project.name = name;
        project.updated_at = utc_timestamp();
        atomic_write_json(&self.project_dir(project_id).join("project.json"), &project)?;
        Ok(project)
    }

    pub fn save_transcript_segments(
        &self,
        project_id: &str,
        segments: Vec<crate::model::TranscriptSegment>,
    ) -> Result<TranscriptDocument, String> {
        let mut project = self.read_project(project_id)?;
        let revision = project.transcript_revision.saturating_add(1);
        let document = TranscriptDocument {
            schema_version: 1,
            project_id: project_id.to_string(),
            revision,
            segments,
        };
        self.write_transcript(project_id, &document)?;
        project.transcript_revision = revision;
        project.updated_at = utc_timestamp();
        atomic_write_json(&self.project_dir(project_id).join("project.json"), &project)?;
        Ok(document)
    }

    pub fn relink_media(
        &self,
        project_id: &str,
        source: &Path,
        info: &MediaInfo,
    ) -> Result<TranscriptionProject, String> {
        if !source.is_file() {
            return Err(format!("文件不存在：{}", source.display()));
        }
        let mut project = self.read_project(project_id)?;
        project.media.path = Some(source.to_string_lossy().to_string());
        project.media.storage = MediaStorage::Reference;
        project.media.origin = MediaOrigin::Imported;
        project.media.original_file_name = info.file_name.clone();
        project.media.size_bytes = info.size_bytes;
        project.media.container = info.container.clone();
        project.duration_secs = info.duration_secs;
        project.updated_at = utc_timestamp();
        atomic_write_json(&self.project_dir(project_id).join("project.json"), &project)?;
        Ok(project)
    }

    pub fn delete_managed_media(&self, project_id: &str) -> Result<TranscriptionProject, String> {
        let mut project = self.read_project(project_id)?;
        if project.media.storage == MediaStorage::Reference {
            return Err("引用的外部文件不能由 SnapScribe 删除".to_string());
        }
        if let Some(path) = project.media.path.as_deref() {
            let path = PathBuf::from(path);
            let project_dir = self.project_dir(project_id);
            let parent = path.parent().ok_or("媒体路径无父目录")?;
            let canonical_project = project_dir
                .canonicalize()
                .map_err(|e| format!("无法验证项目目录：{e}"))?;
            let canonical_parent = parent
                .canonicalize()
                .map_err(|e| format!("无法验证媒体目录：{e}"))?;
            if !canonical_parent.starts_with(&canonical_project) {
                return Err("拒绝删除项目目录之外的媒体".to_string());
            }
            if path.is_file() {
                std::fs::remove_file(&path).map_err(|e| format!("删除媒体失败：{e}"))?;
            }
        }
        project.media.path = None;
        project.updated_at = utc_timestamp();
        atomic_write_json(&self.project_dir(project_id).join("project.json"), &project)?;
        Ok(project)
    }

    pub fn save_summary(&self, project_id: &str, summary: &AISummary) -> Result<(), String> {
        if summary.project_id != project_id {
            return Err("AI 总结与项目不匹配".to_string());
        }
        let mut project = self.read_project(project_id)?;
        atomic_write_json(&self.project_dir(project_id).join("summary.json"), summary)?;
        project.summary_revision = Some(summary.source_transcript_revision);
        project.updated_at = utc_timestamp();
        atomic_write_json(&self.project_dir(project_id).join("project.json"), &project)
    }

    pub fn save_action_items(&self, project_id: &str, items: Vec<ActionItem>) -> Result<AISummary, String> {
        let mut ids = std::collections::HashSet::new();
        for item in &items {
            if item.id.trim().is_empty() || item.text.trim().is_empty() || !ids.insert(item.id.clone()) {
                return Err("待办事项包含空内容或重复 ID".to_string());
            }
        }
        let mut detail = self.read_detail(project_id)?;
        let mut summary = detail.summary.take().ok_or("尚未生成 AI 总结")?;
        summary.schema_version = 2;
        summary.action_items = items;
        self.save_summary(project_id, &summary)?;
        Ok(summary)
    }
}

fn validate_project_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("项目名称不能为空".to_string());
    }
    if name.chars().count() > 120 {
        return Err("项目名称不能超过 120 个字符".to_string());
    }
    if name.chars().any(char::is_control) {
        return Err("项目名称包含无效字符".to_string());
    }
    Ok(name.to_string())
}

fn validate_project_id(project_id: &str) -> Result<(), String> {
    let parsed = uuid::Uuid::parse_str(project_id).map_err(|_| "无效的项目 ID".to_string())?;
    if parsed.to_string() != project_id.to_ascii_lowercase() {
        return Err("无效的项目 ID".to_string());
    }
    Ok(())
}

fn atomic_write_json<T: Serialize>(target: &Path, value: &T) -> Result<(), String> {
    let parent = target.parent().ok_or("目标路径无父目录")?;
    std::fs::create_dir_all(parent).map_err(|e| format!("无法创建数据目录：{e}"))?;
    let temp = parent.join(format!(".{}.tmp", uuid::Uuid::new_v4()));
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| format!("JSON 序列化失败：{e}"))?;
    let mut file = File::create(&temp).map_err(|e| format!("无法创建临时文件：{e}"))?;
    file.write_all(&bytes)
        .and_then(|_| file.sync_all())
        .map_err(|e| format!("无法写入临时文件：{e}"))?;
    let backup = parent.join(format!(".{}.backup", uuid::Uuid::new_v4()));
    let had_target = target.exists();
    if had_target {
        std::fs::rename(target, &backup).map_err(|e| format!("无法备份旧数据：{e}"))?;
    }
    if let Err(error) = std::fs::rename(&temp, target) {
        if had_target {
            let _ = std::fs::rename(&backup, target);
        }
        let _ = std::fs::remove_file(&temp);
        return Err(format!("无法保存数据：{error}"));
    }
    if had_target {
        std::fs::remove_file(backup).map_err(|e| format!("无法清理数据备份：{e}"))?;
    }
    Ok(())
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path, label: &str) -> Result<T, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("无法读取{label}：{e}"))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("{label}数据损坏：{e}"))
}

fn safe_file_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|character| match character {
            '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            value if value.is_control() => '_',
            value => value,
        })
        .collect();
    if cleaned.trim().is_empty() {
        "media".to_string()
    } else {
        cleaned
    }
}

fn utc_timestamp() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{seconds}")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::ProjectStore;
    use crate::model::{
        ActionItem, AISummary, ImportStrategy, MediaInfo, ProjectSort, TranscriptDocument, TranscriptSegment,
    };

    fn temp_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "snapscribe-project-store-{label}-{}",
            uuid::Uuid::new_v4()
        ))
    }

    fn media_info() -> MediaInfo {
        MediaInfo {
            file_name: "meeting.wav".to_string(),
            size_bytes: 4,
            duration_secs: 12.5,
            container: "wav".to_string(),
        }
    }

    #[test]
    fn creates_a_reference_project_without_copying_media() {
        let root = temp_root("reference");
        let source = root.join("source").join("meeting.wav");
        std::fs::create_dir_all(source.parent().unwrap()).unwrap();
        std::fs::write(&source, b"RIFF").unwrap();
        let store = ProjectStore::new(root.join("data"));

        let created = store
            .create_imported_project(&source, &media_info(), ImportStrategy::Reference)
            .unwrap();

        assert_eq!(created.media.storage.as_str(), "reference");
        assert_eq!(created.media.path.as_deref(), source.to_str());
        assert!(!store.project_dir(&created.id).join("media").exists());
        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn moves_media_into_the_project_directory_after_commit() {
        let root = temp_root("move");
        let source = root.join("source").join("meeting.wav");
        std::fs::create_dir_all(source.parent().unwrap()).unwrap();
        std::fs::write(&source, b"RIFF").unwrap();
        let store = ProjectStore::new(root.join("data"));

        let created = store
            .create_imported_project(&source, &media_info(), ImportStrategy::Move)
            .unwrap();
        let target = PathBuf::from(created.media.path.unwrap());

        assert!(!source.exists());
        assert_eq!(target.parent(), Some(store.project_dir(&created.id).as_path()));
        assert_eq!(std::fs::read(target).unwrap(), b"RIFF");
        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn saves_editable_action_items_without_replacing_summary_content() {
        let root = temp_root("action-items");
        let source = root.join("source").join("meeting.wav");
        std::fs::create_dir_all(source.parent().unwrap()).unwrap();
        std::fs::write(&source, b"RIFF").unwrap();
        let store = ProjectStore::new(root.join("data"));
        let project = store.create_imported_project(&source, &media_info(), ImportStrategy::Reference).unwrap();
        let summary = AISummary {
            schema_version: 2,
            project_id: project.id.clone(),
            source_transcript_revision: 0,
            generated_at: "now".into(),
            summary: "原摘要".into(),
            key_points: vec!["原要点".into()],
            action_items: Vec::new(),
            model: "model".into(),
            model_config_id: None,
            model_name: None,
        };
        store.save_summary(&project.id, &summary).unwrap();

        let saved = store.save_action_items(&project.id, vec![ActionItem { id: "a".into(), text: "跟进".into(), completed: true }]).unwrap();

        assert_eq!(saved.summary, "原摘要");
        assert_eq!(saved.key_points, vec!["原要点"]);
        assert!(saved.action_items[0].completed);
        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn transcript_survives_when_referenced_media_is_removed() {
        let root = temp_root("missing-media");
        let source = root.join("source").join("meeting.wav");
        std::fs::create_dir_all(source.parent().unwrap()).unwrap();
        std::fs::write(&source, b"RIFF").unwrap();
        let store = ProjectStore::new(root.join("data"));
        let project = store
            .create_imported_project(&source, &media_info(), ImportStrategy::Reference)
            .unwrap();
        let transcript = TranscriptDocument {
            schema_version: 1,
            project_id: project.id.clone(),
            revision: 1,
            segments: vec![TranscriptSegment {
                id: "segment-1".to_string(),
                start: 0.0,
                end: 1.0,
                text: "保留的文本".to_string(),
            }],
        };
        store.write_transcript(&project.id, &transcript).unwrap();

        std::fs::remove_file(&source).unwrap();
        let detail = store.read_detail(&project.id).unwrap();

        assert!(!detail.media_available);
        assert_eq!(detail.transcript.segments[0].text, "保留的文本");
        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn rejects_project_id_path_traversal() {
        let root = temp_root("traversal");
        let store = ProjectStore::new(root.clone());

        let result = store.read_project("../outside");

        assert!(result.is_err());
        assert!(!root.join("outside").exists());
        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn searches_transcript_text_without_tags() {
        let root = temp_root("search");
        let source = root.join("source").join("meeting.wav");
        std::fs::create_dir_all(source.parent().unwrap()).unwrap();
        std::fs::write(&source, b"RIFF").unwrap();
        let store = ProjectStore::new(root.join("data"));
        let project = store
            .create_imported_project(&source, &media_info(), ImportStrategy::Reference)
            .unwrap();
        store
            .write_transcript(
                &project.id,
                &TranscriptDocument {
                    schema_version: 1,
                    project_id: project.id.clone(),
                    revision: 1,
                    segments: vec![TranscriptSegment {
                        id: "segment-1".to_string(),
                        start: 0.0,
                        end: 1.0,
                        text: "讨论产品发布节奏".to_string(),
                    }],
                },
            )
            .unwrap();

        let matches = store
            .list_projects("发布节奏", ProjectSort::RecentlyUpdated)
            .unwrap();

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].id, project.id);
        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn deleting_a_reference_project_never_deletes_the_external_media() {
        let root = temp_root("delete-reference");
        let source = root.join("source").join("meeting.wav");
        std::fs::create_dir_all(source.parent().unwrap()).unwrap();
        std::fs::write(&source, b"RIFF").unwrap();
        let store = ProjectStore::new(root.join("data"));
        let project = store
            .create_imported_project(&source, &media_info(), ImportStrategy::Reference)
            .unwrap();

        store.delete_project(&project.id).unwrap();

        assert!(source.is_file());
        assert!(!store.project_dir(&project.id).exists());
        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn completing_transcription_persists_segments_and_completed_status() {
        let root = temp_root("complete");
        let source = root.join("source").join("meeting.wav");
        std::fs::create_dir_all(source.parent().unwrap()).unwrap();
        std::fs::write(&source, b"RIFF").unwrap();
        let store = ProjectStore::new(root.join("data"));
        let project = store
            .create_imported_project(&source, &media_info(), ImportStrategy::Reference)
            .unwrap();
        let segments = vec![TranscriptSegment {
            id: "segment-1".to_string(),
            start: 0.0,
            end: 1.0,
            text: "完成内容".to_string(),
        }];

        store
            .complete_transcription(&project.id, &segments)
            .unwrap();
        let detail = store.read_detail(&project.id).unwrap();

        assert_eq!(detail.project.status.as_str(), "completed");
        assert_eq!(detail.project.transcript_revision, 1);
        assert_eq!(detail.transcript.revision, 1);
        assert_eq!(detail.transcript.segments[0].text, "完成内容");
        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn canceling_transcription_keeps_partial_text_and_marks_the_project_canceled() {
        let root = temp_root("cancel-partial");
        let source = root.join("source").join("meeting.wav");
        std::fs::create_dir_all(source.parent().unwrap()).unwrap();
        std::fs::write(&source, b"RIFF").unwrap();
        let store = ProjectStore::new(root.join("data"));
        let project = store
            .create_imported_project(&source, &media_info(), ImportStrategy::Reference)
            .unwrap();
        store
            .write_transcript(
                &project.id,
                &TranscriptDocument {
                    schema_version: 1,
                    project_id: project.id.clone(),
                    revision: 0,
                    segments: vec![TranscriptSegment {
                        id: "partial-1".to_string(),
                        start: 0.0,
                        end: 1.0,
                        text: "已经识别的内容".to_string(),
                    }],
                },
            )
            .unwrap();

        store.cancel_transcription(&project.id).unwrap();
        let detail = store.read_detail(&project.id).unwrap();

        assert_eq!(detail.project.status.as_str(), "canceled");
        assert_eq!(detail.project.transcript_revision, 1);
        assert_eq!(detail.transcript.revision, 1);
        assert_eq!(detail.transcript.segments[0].text, "已经识别的内容");
        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn imports_legacy_text_as_a_media_less_completed_project() {
        let root = temp_root("legacy");
        let store = ProjectStore::new(root.join("data"));

        let project = store
            .create_legacy_project("旧会议", "保留下来的转录文本")
            .unwrap();
        let detail = store.read_detail(&project.id).unwrap();

        assert_eq!(detail.project.status.as_str(), "completed");
        assert!(!detail.media_available);
        assert_eq!(detail.transcript.segments[0].text, "保留下来的转录文本");
        std::fs::remove_dir_all(root).ok();
    }
}
