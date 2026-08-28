use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

use crate::model::{AISettings, AISummary, ConnectionResult};

pub fn test_connection(settings: &AISettings, api_key: &str) -> Result<ConnectionResult, String> {
    validate(settings)?;
    let body = json!({
        "model": settings.model,
        "messages": [{ "role": "user", "content": "Reply with OK." }],
        "max_tokens": 4,
        "temperature": 0
    });
    send(settings, api_key, body)?;
    Ok(ConnectionResult {
        ok: true,
        message: "连接成功".to_string(),
    })
}

pub fn generate_summary(
    settings: &AISettings,
    api_key: &str,
    project_id: &str,
    revision: u32,
    transcript: &str,
) -> Result<AISummary, String> {
    validate(settings)?;
    if transcript.trim().is_empty() {
        return Err("转录文本为空，无法生成总结".to_string());
    }
    let prompt = format!(
        "请根据以下转录文本生成 JSON。只返回 JSON 对象，字段固定为 summary（字符串）、keyPoints（字符串数组）、actionItems（字符串数组）。不要添加其他字段。\n\n转录文本：\n{}",
        transcript
    );
    let body = json!({
        "model": settings.model,
        "messages": [
            { "role": "system", "content": "你是会议与音视频内容总结助手，输出准确、简洁、可执行的中文总结。" },
            { "role": "user", "content": prompt }
        ],
        "temperature": 0.2
    });
    let response = send(settings, api_key, body)?;
    let content = response
        .pointer("/choices/0/message/content")
        .and_then(Value::as_str)
        .ok_or("AI 返回中缺少总结内容")?;
    let cleaned = content
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();
    let parsed: Value =
        serde_json::from_str(cleaned).map_err(|e| format!("AI 总结不是有效 JSON：{e}"))?;
    let summary = parsed
        .get("summary")
        .and_then(Value::as_str)
        .filter(|v| !v.trim().is_empty())
        .ok_or("AI 总结缺少 summary")?
        .trim()
        .to_string();
    let key_points = string_array(&parsed, "keyPoints")?;
    let action_items = string_array(&parsed, "actionItems")?;
    Ok(AISummary {
        schema_version: 1,
        project_id: project_id.to_string(),
        source_transcript_revision: revision,
        generated_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_string(),
        summary,
        key_points,
        action_items,
        model: settings.model.clone(),
    })
}

fn validate(settings: &AISettings) -> Result<(), String> {
    if settings.base_url.trim().is_empty() {
        return Err("请填写 API Base URL".to_string());
    }
    if settings.model.trim().is_empty() {
        return Err("请填写模型名称".to_string());
    }
    Ok(())
}

fn endpoint(base_url: &str) -> String {
    let base = base_url.trim().trim_end_matches('/');
    if base.ends_with("/chat/completions") {
        base.to_string()
    } else {
        format!("{base}/chat/completions")
    }
}

fn send(settings: &AISettings, api_key: &str, body: Value) -> Result<Value, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| format!("无法创建 AI 请求：{e}"))?;
    let response = client
        .post(endpoint(&settings.base_url))
        .bearer_auth(api_key.trim())
        .json(&body)
        .send()
        .map_err(|e| format!("AI 请求失败：{e}"))?;
    let status = response.status();
    let text = response
        .text()
        .map_err(|e| format!("无法读取 AI 响应：{e}"))?;
    if !status.is_success() {
        let detail = text.chars().take(300).collect::<String>();
        return Err(format!("AI 服务返回 HTTP {}：{}", status.as_u16(), detail));
    }
    serde_json::from_str(&text).map_err(|e| format!("AI 服务返回无效 JSON：{e}"))
}

fn string_array(value: &Value, key: &str) -> Result<Vec<String>, String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("AI 总结缺少 {key}"))?
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_string)
                .ok_or_else(|| format!("AI 总结字段 {key} 包含无效内容"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AIServiceType, AISettings};

    fn settings(base_url: &str, model: &str) -> AISettings {
        AISettings {
            service_type: AIServiceType::OpenAICompatible,
            base_url: base_url.to_string(),
            model: model.to_string(),
            has_api_key: true,
        }
    }

    #[test]
    fn builds_openai_compatible_chat_completions_endpoint() {
        assert_eq!(
            endpoint("https://api.openai.com/v1/"),
            "https://api.openai.com/v1/chat/completions"
        );
        assert_eq!(
            endpoint("https://host/v1/chat/completions"),
            "https://host/v1/chat/completions"
        );
    }

    #[test]
    fn rejects_incomplete_ai_configuration() {
        assert!(validate(&settings("", "model")).is_err());
        assert!(validate(&settings("https://host/v1", "")).is_err());
        assert!(validate(&settings("https://host/v1", "model")).is_ok());
    }

    #[test]
    fn accepts_only_non_empty_string_arrays() {
        let value = serde_json::json!({"items": [" one ", "two"]});
        assert_eq!(string_array(&value, "items").unwrap(), vec!["one", "two"]);
        assert!(string_array(&serde_json::json!({"items": [1]}), "items").is_err());
    }
}
