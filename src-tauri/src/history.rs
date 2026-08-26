use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::model::HistoryEntry;

/// Characters Windows forbids in file names.
const FORBIDDEN: [char; 9] = ['\\', '/', ':', '*', '?', '"', '<', '>', '|'];
const MAX_NAME_LEN: usize = 120;

/// List `.txt` files newest-first. Missing directory yields an empty list.
pub fn list(dir: &Path) -> Vec<HistoryEntry> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut items: Vec<HistoryEntry> = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if !path.extension()?.eq_ignore_ascii_case("txt") || !path.is_file() {
                return None;
            }
            let meta = entry.metadata().ok()?;
            Some(HistoryEntry {
                file_name: path.file_name()?.to_str()?.to_string(),
                size_bytes: meta.len(),
                modified_ms: meta
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0),
            })
        })
        .collect();
    items.sort_by(|a, b| b.modified_ms.cmp(&a.modified_ms).then(a.file_name.cmp(&b.file_name)));
    items
}

/// Read one history file by exact name (validated, no traversal).
pub fn read_file(dir: &Path, name: &str) -> Result<String, String> {
    validate_filename(name)?;
    let content = std::fs::read_to_string(dir.join(name))
        .map_err(|e| format!("无法读取历史文件：{e}"))?;
    Ok(content)
}

/// Rename a history file, returning the final name. The `.txt` suffix is kept;
/// a colliding target gets `-2`, `-3`, … inserted before the suffix.
pub fn rename(dir: &Path, old_name: &str, new_name: &str) -> Result<String, String> {
    validate_filename(old_name)?;
    let base = normalize(new_name)?;
    let from = dir.join(old_name);
    if !from.is_file() {
        return Err(format!("文件不存在：{old_name}"));
    }
    for attempt in 1..=99u32 {
        let candidate = if attempt == 1 {
            base.clone()
        } else {
            with_sequence_suffix(&base, attempt)
        };
        let target_path = dir.join(&candidate);
        if !target_path.exists() {
            std::fs::rename(&from, &target_path).map_err(|e| format!("重命名失败：{e}"))?;
            return Ok(candidate);
        }
        // Renaming onto itself is an accepted no-op.
        let same_target = target_path
            .canonicalize()
            .ok()
            .zip(from.canonicalize().ok())
            .is_some_and(|(t, f)| t == f);
        if same_target {
            return Ok(candidate);
        }
    }
    Err("目标名称冲突过多".to_string())
}

/// Delete one history file.
pub fn delete(dir: &Path, name: &str) -> Result<(), String> {
    validate_filename(name)?;
    let path = dir.join(name);
    if !path.is_file() {
        return Err(format!("文件不存在：{name}"));
    }
    std::fs::remove_file(path).map_err(|e| format!("删除失败：{e}"))
}

/// Auto-save the transcript of a completed job as `<stem>-<yyyyMMdd-HHmmss>.txt`.
pub fn save_transcript(dir: &Path, source_stem: &str, content: &str) -> Result<PathBuf, String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("无法创建历史目录：{e}"))?;
    let stem = sanitize_component(source_stem);
    let stamp = format_timestamp(SystemTime::now());
    let mut name = format!("{stem}-{stamp}.txt");
    for attempt in 1..=99 {
        if !dir.join(&name).exists() {
            let path = dir.join(&name);
            crate::export::write_utf8(&path, content)?;
            return Ok(path);
        }
        name = format!("{stem}-{stamp}-{attempt}.txt");
    }
    Err("历史文件名冲突过多".to_string())
}

fn normalize(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
    let with_ext = if trimmed.to_ascii_lowercase().ends_with(".txt") {
        trimmed.to_string()
    } else {
        format!("{trimmed}.txt")
    };
    validate_filename(&with_ext)?;
    Ok(with_ext)
}

/// Reject empty/overlong names, forbidden characters, control characters and
/// path separators so history commands can never escape the history directory.
pub fn validate_filename(name: &str) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("文件名不能为空".to_string());
    }
    if name.len() > MAX_NAME_LEN + 4 {
        return Err("文件名过长".to_string());
    }
    if FORBIDDEN.iter().any(|c| name.contains(*c)) {
        return Err("文件名包含非法字符：\\ / : * ? \" < > |".to_string());
    }
    if name.chars().any(|c| c.is_control()) {
        return Err("文件名包含控制字符".to_string());
    }
    match name {
        "." | ".." | ".txt" => Err("无效的文件名".to_string()),
        _ => Ok(()),
    }
}

/// Reduce an arbitrary source-file stem to a safe name component.
pub fn sanitize_component(stem: &str) -> String {
    let cleaned: String = stem
        .trim()
        .chars()
        .map(|c| {
            if FORBIDDEN.contains(&c) || c.is_control() {
                '_'
            } else {
                c
            }
        })
        .collect();
    let cleaned = cleaned.trim_matches('.').trim();
    if cleaned.is_empty() {
        "transcript".to_string()
    } else if cleaned.len() > 60 {
        cleaned.chars().take(60).collect()
    } else {
        cleaned.to_string()
    }
}

fn with_sequence_suffix(name: &str, seq: u32) -> String {
    match name.strip_suffix(".txt") {
        Some(stem) => format!("{stem}-{seq}.txt"),
        None => format!("{name}-{seq}"),
    }
}

/// `yyyyMMdd-HHmmss` local-free timestamp derived from the Unix clock.
fn format_timestamp(t: SystemTime) -> String {
    let secs = t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
    let days = secs.div_euclid(86_400);
    let secs_of_day = secs.rem_euclid(86_400);
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}{m:02}{d:02}-{h:02}{min:02}{s:02}",
        h = secs_of_day / 3600,
        min = (secs_of_day % 3600) / 60,
        s = secs_of_day % 60,
    )
}

/// Howard Hinnant's civil-from-days algorithm (public domain).
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    ((if m <= 2 { y + 1 } else { y }), m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_traversal_and_forbidden_characters() {
        assert!(validate_filename("../evil.txt").is_err());
        assert!(validate_filename("a/b.txt").is_err());
        assert!(validate_filename("a:b.txt").is_err());
        assert!(validate_filename("").is_err());
        assert!(validate_filename(".txt").is_err());
        assert!(validate_filename("正常名称.txt").is_ok());
    }

    #[test]
    fn rename_appends_extension_and_resolves_collisions() {
        let dir = std::env::temp_dir().join(format!("snapscribe-hist-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.txt"), "one").unwrap();
        std::fs::write(dir.join("b.txt"), "two").unwrap();

        let renamed = rename(&dir, "a.txt", "b").unwrap();
        assert_eq!(renamed, "b-2.txt");
        assert_eq!(std::fs::read_to_string(dir.join("b-2.txt")).unwrap(), "one");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn save_transcript_generates_unique_dated_files() {
        let dir = std::env::temp_dir().join(format!("snapscribe-save-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p1 = save_transcript(&dir, "会议记录", "hello").unwrap();
        let p2 = save_transcript(&dir, "会议记录", "hello again").unwrap();
        let n1 = p1.file_name().unwrap().to_str().unwrap().to_string();
        assert_ne!(n1, p2.file_name().unwrap().to_str().unwrap());
        assert!(n1.starts_with("会议记录-2"));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn sanitize_component_replaces_illegal_chars() {
        assert_eq!(sanitize_component("a:b*c"), "a_b_c");
        assert_eq!(sanitize_component("..."), "transcript");
    }
}
