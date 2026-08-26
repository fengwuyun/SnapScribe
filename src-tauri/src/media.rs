use std::path::Path;
use std::process::{Command, Stdio};

use crate::model::MediaInfo;

/// Read media duration via ffprobe and combine it with file metadata.
pub fn probe(ffprobe: &Path, input: &Path) -> Result<MediaInfo, String> {
    let output = Command::new(ffprobe)
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "json",
        ])
        .arg(input)
        .stdin(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("无法启动 ffprobe：{e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "无法读取媒体信息（文件可能损坏或格式不受支持）：{}",
            first_line(&stderr)
        ));
    }

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).map_err(|e| format!("ffprobe 输出解析失败:{e}"))?;
    let duration_text = json["format"]["duration"]
        .as_str()
        .ok_or("ffprobe 未返回时长")?;
    let duration_secs: f64 = duration_text
        .parse()
        .map_err(|_| format!("ffprobe 时长不是数字：{duration_text}"))?;
    if !duration_secs.is_finite() || duration_secs <= 0.0 {
        return Err(format!("媒体时长无效：{duration_secs}"));
    }

    let meta = std::fs::metadata(input).map_err(|e| format!("无法读取文件元数据：{e}"))?;
    let container = input
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    Ok(MediaInfo {
        file_name: file_name_of(input),
        size_bytes: meta.len(),
        duration_secs,
        container,
    })
}

fn file_name_of(path: &Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_string()
}

fn first_line(text: &str) -> &str {
    text.lines().next().unwrap_or("").trim()
}
