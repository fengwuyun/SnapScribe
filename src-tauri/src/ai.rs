use std::path::Path;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use reqwest::header::{HeaderName, HeaderValue};
use serde_json::{json, Value};

use crate::ai_service_store::AIServiceStore;
use crate::model::{
    AIAuthType, AIEndpointMode, AIModelConfig, AIModelDraft, AIModelStatus, AIModelStatusKind,
    AIProtocol, AISettings, AISummary, ConnectionResult,
};
use crate::secret::SecretStore;

#[derive(Debug)]
struct AIRequestFailure {
    kind: AIModelStatusKind,
    message: String,
}

impl AIRequestFailure {
    fn new(kind: AIModelStatusKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    fn status(&self, elapsed: Duration) -> AIModelStatus {
        AIModelStatus {
            status: self.kind,
            message: Some(self.message.clone()),
            checked_at: Some(now_string()),
            response_time_ms: Some(elapsed.as_millis().min(u128::from(u64::MAX)) as u64),
        }
    }
}

pub fn test_connection(settings: &AISettings, api_key: &str) -> Result<ConnectionResult, String> {
    let model = legacy_model(settings);
    test_config(&model, api_key)
        .map(|_| ConnectionResult {
            ok: true,
            message: "连接成功".to_string(),
        })
        .map_err(|failure| failure.message)
}

pub fn test_model(draft: &AIModelDraft, api_key: &str) -> AIModelStatus {
    let model = draft_model(draft);
    let started = Instant::now();
    match test_config(&model, api_key) {
        Ok(_) => AIModelStatus {
            status: AIModelStatusKind::Available,
            message: Some("连接成功".to_string()),
            checked_at: Some(now_string()),
            response_time_ms: Some(started.elapsed().as_millis() as u64),
        },
        Err(failure) => failure.status(started.elapsed()),
    }
}

pub fn generate_summary_with_failover(
    store: &AIServiceStore,
    config_dir: &Path,
    project_id: &str,
    revision: u32,
    transcript: &str,
) -> Result<AISummary, String> {
    if transcript.trim().is_empty() {
        return Err("转录文本为空，无法生成总结".to_string());
    }
    let config = store.load()?;
    let mut models = config
        .models
        .into_iter()
        .filter(|model| model.enabled)
        .collect::<Vec<_>>();
    models.sort_by_key(|model| model.order);
    if models.is_empty() {
        return Err("NO_AI_MODELS_CONFIGURED:请先在 AI 服务中添加并启用模型".to_string());
    }

    let mut failures = Vec::new();
    for model in models {
        let key = if model.auth_type == AIAuthType::None {
            String::new()
        } else {
            match SecretStore::for_model(config_dir, &model.id).load() {
                Ok(key) => key,
                Err(message) => {
                    let failure = AIRequestFailure::new(AIModelStatusKind::ConfigError, message);
                    store.update_status(&model.id, failure.status(Duration::ZERO))?;
                    failures.push(format!("{}：{}", model.name, failure.message));
                    continue;
                }
            }
        };
        let started = Instant::now();
        match generate_summary_for_model(
            &model,
            &key,
            &config.custom_instruction,
            project_id,
            revision,
            transcript,
        ) {
            Ok(summary) => {
                store.update_status(
                    &model.id,
                    AIModelStatus {
                        status: AIModelStatusKind::Available,
                        message: Some("请求成功".to_string()),
                        checked_at: Some(now_string()),
                        response_time_ms: Some(started.elapsed().as_millis() as u64),
                    },
                )?;
                return Ok(summary);
            }
            Err(failure) => {
                store.update_status(&model.id, failure.status(started.elapsed()))?;
                failures.push(format!("{}：{}", model.name, failure.message));
            }
        }
    }
    Err(format!("所有已启用模型均请求失败：{}", failures.join("；")))
}

fn test_config(model: &AIModelConfig, api_key: &str) -> Result<Value, AIRequestFailure> {
    validate(model)?;
    send(
        model,
        api_key,
        json!({
            "model": model.model_id,
            "messages": [{ "role": "user", "content": "Reply with OK." }],
            "max_tokens": 4,
            "temperature": 0
        }),
    )
}

fn generate_summary_for_model(
    model: &AIModelConfig,
    api_key: &str,
    custom_instruction: &str,
    project_id: &str,
    revision: u32,
    transcript: &str,
) -> Result<AISummary, AIRequestFailure> {
    validate(model)?;
    let system_prompt = build_system_prompt(custom_instruction);
    let prompt = format!(
        "请根据以下转录文本生成 JSON。只返回 JSON 对象，字段固定为 summary（字符串）、keyPoints（字符串数组）、actionItems（字符串数组）。不要添加其他字段。\n\n转录文本：\n{}",
        transcript
    );
    let response = send(
        model,
        api_key,
        json!({
            "model": model.model_id,
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": prompt }
            ],
            "temperature": 0.2
        }),
    )?;
    let content = response
        .pointer("/choices/0/message/content")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            AIRequestFailure::new(AIModelStatusKind::InvalidResponse, "AI 返回中缺少总结内容")
        })?;
    let parsed: Value = serde_json::from_str(clean_json_fence(content)).map_err(|e| {
        AIRequestFailure::new(
            AIModelStatusKind::InvalidResponse,
            format!("AI 总结不是有效 JSON：{e}"),
        )
    })?;
    let summary = parsed
        .get("summary")
        .and_then(Value::as_str)
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| {
            AIRequestFailure::new(AIModelStatusKind::InvalidResponse, "AI 总结缺少 summary")
        })?
        .trim()
        .to_string();
    let key_points = string_array(&parsed, "keyPoints")?;
    let action_items = string_array(&parsed, "actionItems")?;
    Ok(AISummary {
        schema_version: 1,
        project_id: project_id.to_string(),
        source_transcript_revision: revision,
        generated_at: now_string(),
        summary,
        key_points,
        action_items,
        model: model.model_id.clone(),
        model_config_id: Some(model.id.clone()),
        model_name: Some(model.name.clone()),
    })
}

fn build_system_prompt(custom_instruction: &str) -> String {
    let base = "你是会议与音视频内容总结助手，输出准确、简洁、可执行的中文总结。";
    let custom = custom_instruction.trim();
    if custom.is_empty() {
        base.to_string()
    } else {
        format!("{base}\n\n用户自定义指令：\n{custom}")
    }
}

fn validate(model: &AIModelConfig) -> Result<(), AIRequestFailure> {
    if model.protocol != AIProtocol::OpenAIChat {
        return Err(AIRequestFailure::new(
            AIModelStatusKind::ConfigError,
            "暂不支持该接口协议",
        ));
    }
    if model.base_url.trim().is_empty() {
        return Err(AIRequestFailure::new(
            AIModelStatusKind::ConfigError,
            "请填写 API Base URL",
        ));
    }
    if model.model_id.trim().is_empty() {
        return Err(AIRequestFailure::new(
            AIModelStatusKind::ConfigError,
            "请填写模型 ID",
        ));
    }
    Ok(())
}

fn endpoint(model: &AIModelConfig) -> Result<String, AIRequestFailure> {
    let base = model.base_url.trim().trim_end_matches('/');
    match model.endpoint_mode {
        AIEndpointMode::FullUrl => Ok(base.to_string()),
        AIEndpointMode::Auto => {
            if base.ends_with("/chat/completions") {
                Ok(base.to_string())
            } else {
                Ok(format!("{base}/chat/completions"))
            }
        }
        AIEndpointMode::CustomPath => {
            let path = model
                .custom_path
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .ok_or_else(|| {
                    AIRequestFailure::new(AIModelStatusKind::ConfigError, "请填写自定义请求路径")
                })?;
            Ok(format!("{base}/{}", path.trim_start_matches('/')))
        }
    }
}

fn send(model: &AIModelConfig, api_key: &str, body: Value) -> Result<Value, AIRequestFailure> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(model.timeout_secs.clamp(5, 300)))
        .build()
        .map_err(|e| {
            AIRequestFailure::new(
                AIModelStatusKind::ConfigError,
                format!("无法创建 AI 请求：{e}"),
            )
        })?;
    let mut request = client.post(endpoint(model)?).json(&body);
    match model.auth_type {
        AIAuthType::Bearer => request = request.bearer_auth(api_key.trim()),
        AIAuthType::XApiKey => request = request.header("x-api-key", api_key.trim()),
        AIAuthType::CustomHeader => {
            let name = model
                .auth_header
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .ok_or_else(|| {
                    AIRequestFailure::new(AIModelStatusKind::ConfigError, "请填写认证 Header 名称")
                })?;
            let header_name = HeaderName::from_bytes(name.as_bytes()).map_err(|_| {
                AIRequestFailure::new(AIModelStatusKind::ConfigError, "认证 Header 名称无效")
            })?;
            let header_value = HeaderValue::from_str(api_key.trim()).map_err(|_| {
                AIRequestFailure::new(AIModelStatusKind::ConfigError, "API Key 包含无效字符")
            })?;
            request = request.header(header_name, header_value);
        }
        AIAuthType::None => {}
    }
    let response = request.send().map_err(|e| {
        if e.is_timeout() {
            AIRequestFailure::new(AIModelStatusKind::Timeout, "请求超时")
        } else {
            AIRequestFailure::new(AIModelStatusKind::ServiceError, format!("AI 请求失败：{e}"))
        }
    })?;
    let status = response.status();
    let text = response.text().map_err(|e| {
        AIRequestFailure::new(
            AIModelStatusKind::InvalidResponse,
            format!("无法读取 AI 响应：{e}"),
        )
    })?;
    if !status.is_success() {
        let kind = match status.as_u16() {
            401 | 403 => AIModelStatusKind::AuthFailed,
            429 => AIModelStatusKind::RateLimited,
            500..=599 => AIModelStatusKind::ServiceError,
            _ => AIModelStatusKind::ServiceError,
        };
        let detail = text.chars().take(240).collect::<String>();
        return Err(AIRequestFailure::new(
            kind,
            format!("HTTP {}：{}", status.as_u16(), detail),
        ));
    }
    serde_json::from_str(&text).map_err(|e| {
        AIRequestFailure::new(
            AIModelStatusKind::InvalidResponse,
            format!("AI 服务返回无效 JSON：{e}"),
        )
    })
}

fn string_array(value: &Value, key: &str) -> Result<Vec<String>, AIRequestFailure> {
    value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| {
            AIRequestFailure::new(
                AIModelStatusKind::InvalidResponse,
                format!("AI 总结缺少 {key}"),
            )
        })?
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_string)
                .ok_or_else(|| {
                    AIRequestFailure::new(
                        AIModelStatusKind::InvalidResponse,
                        format!("AI 总结字段 {key} 包含无效内容"),
                    )
                })
        })
        .collect()
}

fn clean_json_fence(content: &str) -> &str {
    content
        .trim()
        .strip_prefix("```json")
        .or_else(|| content.trim().strip_prefix("```"))
        .unwrap_or(content.trim())
        .strip_suffix("```")
        .unwrap_or_else(|| {
            content
                .trim()
                .strip_prefix("```json")
                .or_else(|| content.trim().strip_prefix("```"))
                .unwrap_or(content.trim())
        })
        .trim()
}

fn now_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}

fn legacy_model(settings: &AISettings) -> AIModelConfig {
    AIModelConfig {
        id: "legacy".to_string(),
        name: "现有模型".to_string(),
        protocol: AIProtocol::OpenAIChat,
        base_url: settings.base_url.clone(),
        model_id: settings.model.clone(),
        endpoint_mode: AIEndpointMode::Auto,
        custom_path: None,
        auth_type: AIAuthType::Bearer,
        auth_header: None,
        timeout_secs: 60,
        enabled: true,
        order: 0,
        has_api_key: settings.has_api_key,
        last_status: AIModelStatus::default(),
    }
}

fn draft_model(draft: &AIModelDraft) -> AIModelConfig {
    AIModelConfig {
        id: draft.id.clone().unwrap_or_else(|| "test".to_string()),
        name: draft.name.clone(),
        protocol: draft.protocol,
        base_url: draft.base_url.clone(),
        model_id: draft.model_id.clone(),
        endpoint_mode: draft.endpoint_mode,
        custom_path: draft.custom_path.clone(),
        auth_type: draft.auth_type,
        auth_header: draft.auth_header.clone(),
        timeout_secs: draft.timeout_secs,
        enabled: draft.enabled,
        order: 0,
        has_api_key: draft
            .api_key
            .as_ref()
            .is_some_and(|key| !key.trim().is_empty()),
        last_status: AIModelStatus::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::AIServiceType;

    fn model(base_url: &str) -> AIModelConfig {
        legacy_model(&AISettings {
            service_type: AIServiceType::OpenAICompatible,
            base_url: base_url.to_string(),
            model: "model".to_string(),
            has_api_key: true,
        })
    }

    #[test]
    fn builds_supported_endpoints() {
        assert_eq!(
            endpoint(&model("https://api.openai.com/v1/")).unwrap(),
            "https://api.openai.com/v1/chat/completions"
        );
        let mut full = model("https://host/custom");
        full.endpoint_mode = AIEndpointMode::FullUrl;
        assert_eq!(endpoint(&full).unwrap(), "https://host/custom");
        let mut custom = model("https://host/v1");
        custom.endpoint_mode = AIEndpointMode::CustomPath;
        custom.custom_path = Some("messages".to_string());
        assert_eq!(endpoint(&custom).unwrap(), "https://host/v1/messages");
    }

    #[test]
    fn custom_instruction_is_appended_to_system_prompt() {
        let prompt = build_system_prompt("使用项目符号");
        assert!(prompt.contains("使用项目符号"));
        assert!(!build_system_prompt("").contains("用户自定义指令"));
    }

    #[test]
    fn strips_json_code_fences() {
        assert_eq!(clean_json_fence("```json\n{\"a\":1}\n```"), "{\"a\":1}");
    }
}
