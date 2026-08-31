use std::path::{Path, PathBuf};

use crate::model::{
    AIAuthType, AIEndpointMode, AIModelConfig, AIModelDraft, AIModelStatus, AIProtocol,
    AIServiceConfig, AISettings,
};

pub const GLM_47_FLASH_PRESET_KEY: &str = "glm-4.7-flash";

pub struct AIServiceStore {
    config_dir: PathBuf,
    path: PathBuf,
}

impl AIServiceStore {
    pub fn new(config_dir: PathBuf) -> Self {
        let path = config_dir.join("ai-service.json");
        Self { config_dir, path }
    }

    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    pub fn load(&self) -> Result<AIServiceConfig, String> {
        if !self.path.exists() {
            return Ok(AIServiceConfig::default());
        }
        let bytes = std::fs::read(&self.path).map_err(|e| format!("无法读取 AI 服务配置：{e}"))?;
        let mut config: AIServiceConfig =
            serde_json::from_slice(&bytes).map_err(|e| format!("AI 服务配置已损坏：{e}"))?;
        normalize_order(&mut config.models);
        Ok(config)
    }

    pub fn save(&self, config: &AIServiceConfig) -> Result<(), String> {
        std::fs::create_dir_all(&self.config_dir)
            .map_err(|e| format!("无法创建 AI 服务配置目录：{e}"))?;
        let temp = self
            .config_dir
            .join(format!(".ai-service-{}.tmp", uuid::Uuid::new_v4()));
        let bytes = serde_json::to_vec_pretty(config)
            .map_err(|e| format!("无法序列化 AI 服务配置：{e}"))?;
        std::fs::write(&temp, bytes).map_err(|e| format!("无法写入 AI 服务配置：{e}"))?;
        if self.path.exists() {
            std::fs::remove_file(&self.path).map_err(|e| format!("无法替换 AI 服务配置：{e}"))?;
        }
        std::fs::rename(&temp, &self.path).map_err(|e| format!("无法启用 AI 服务配置：{e}"))
    }

    pub fn save_instruction(&self, instruction: &str) -> Result<AIServiceConfig, String> {
        if instruction.chars().count() > 4000 {
            return Err("自定义指令不能超过 4000 个字符".to_string());
        }
        let mut config = self.load()?;
        config.custom_instruction = instruction.trim().to_string();
        self.save(&config)?;
        Ok(config)
    }

    pub fn ensure_default_presets(&self) -> Result<AIServiceConfig, String> {
        let mut config = self.load()?;
        if !config.models.iter().any(|model| {
            model.preset_key.as_deref() == Some(GLM_47_FLASH_PRESET_KEY)
        }) {
            config.models.insert(0, AIModelConfig {
                id: uuid::Uuid::new_v4().to_string(),
                preset_key: Some(GLM_47_FLASH_PRESET_KEY.to_string()),
                name: "GLM 4.7 Flash (免费)".to_string(),
                protocol: AIProtocol::OpenAIChat,
                base_url: "https://open.bigmodel.cn/api/paas/v4".to_string(),
                model_id: GLM_47_FLASH_PRESET_KEY.to_string(),
                endpoint_mode: AIEndpointMode::Auto,
                custom_path: None,
                auth_type: AIAuthType::Bearer,
                auth_header: None,
                timeout_secs: 60,
                enabled: true,
                order: 0,
                has_api_key: false,
                last_status: AIModelStatus::default(),
            });
            assign_order(&mut config.models);
            self.save(&config)?;
        }
        Ok(config)
    }

    pub fn create_model(&self, draft: &AIModelDraft) -> Result<AIModelConfig, String> {
        validate_draft(draft)?;
        let mut config = self.load()?;
        let model = AIModelConfig {
            id: uuid::Uuid::new_v4().to_string(),
            preset_key: None,
            name: draft.name.trim().to_string(),
            protocol: draft.protocol,
            base_url: draft.base_url.trim().trim_end_matches('/').to_string(),
            model_id: draft.model_id.trim().to_string(),
            endpoint_mode: draft.endpoint_mode,
            custom_path: clean_optional(draft.custom_path.as_deref()),
            auth_type: draft.auth_type,
            auth_header: clean_optional(draft.auth_header.as_deref()),
            timeout_secs: draft.timeout_secs.clamp(5, 300),
            enabled: draft.enabled,
            order: config.models.len() as u32,
            has_api_key: false,
            last_status: AIModelStatus::default(),
        };
        config.models.push(model.clone());
        self.save(&config)?;
        Ok(model)
    }

    pub fn update_model(
        &self,
        model_id: &str,
        draft: &AIModelDraft,
    ) -> Result<AIModelConfig, String> {
        validate_draft(draft)?;
        let mut config = self.load()?;
        let model = config
            .models
            .iter_mut()
            .find(|model| model.id == model_id)
            .ok_or("AI 模型不存在")?;
        model.name = draft.name.trim().to_string();
        model.protocol = draft.protocol;
        model.base_url = draft.base_url.trim().trim_end_matches('/').to_string();
        model.model_id = draft.model_id.trim().to_string();
        model.endpoint_mode = draft.endpoint_mode;
        model.custom_path = clean_optional(draft.custom_path.as_deref());
        model.auth_type = draft.auth_type;
        model.auth_header = clean_optional(draft.auth_header.as_deref());
        model.timeout_secs = draft.timeout_secs.clamp(5, 300);
        model.enabled = draft.enabled;
        model.last_status = AIModelStatus::default();
        let result = model.clone();
        self.save(&config)?;
        Ok(result)
    }

    pub fn delete_model(&self, model_id: &str) -> Result<(), String> {
        let mut config = self.load()?;
        let before = config.models.len();
        config.models.retain(|model| model.id != model_id);
        if config.models.len() == before {
            return Err("AI 模型不存在".to_string());
        }
        normalize_order(&mut config.models);
        self.save(&config)
    }

    pub fn reorder_models(&self, ordered_ids: &[String]) -> Result<AIServiceConfig, String> {
        let mut config = self.load()?;
        if ordered_ids.len() != config.models.len() {
            return Err("模型排序数据不完整".to_string());
        }
        let mut reordered = Vec::with_capacity(config.models.len());
        for id in ordered_ids {
            let index = config
                .models
                .iter()
                .position(|model| &model.id == id)
                .ok_or("模型排序包含未知项目")?;
            reordered.push(config.models.remove(index));
        }
        assign_order(&mut reordered);
        config.models = reordered;
        self.save(&config)?;
        Ok(config)
    }

    pub fn set_enabled(&self, model_id: &str, enabled: bool) -> Result<AIModelConfig, String> {
        let mut config = self.load()?;
        let model = config
            .models
            .iter_mut()
            .find(|model| model.id == model_id)
            .ok_or("AI 模型不存在")?;
        model.enabled = enabled;
        let result = model.clone();
        self.save(&config)?;
        Ok(result)
    }

    pub fn update_status(&self, model_id: &str, status: AIModelStatus) -> Result<(), String> {
        let mut config = self.load()?;
        let model = config
            .models
            .iter_mut()
            .find(|model| model.id == model_id)
            .ok_or("AI 模型不存在")?;
        model.last_status = status;
        self.save(&config)
    }

    pub fn migrate_legacy(&self, settings: &AISettings) -> Result<Option<AIModelConfig>, String> {
        let config = self.load()?;
        if !config.models.is_empty()
            || !settings.has_api_key
            || settings.base_url.trim().is_empty()
            || settings.model.trim().is_empty()
        {
            return Ok(None);
        }
        let draft = AIModelDraft {
            id: None,
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
            api_key: None,
            clear_api_key: false,
        };
        self.create_model(&draft).map(Some)
    }
}

fn clean_optional(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn validate_draft(draft: &AIModelDraft) -> Result<(), String> {
    if draft.name.trim().is_empty() {
        return Err("请填写模型名称".to_string());
    }
    if draft.base_url.trim().is_empty() {
        return Err("请填写 API Base URL".to_string());
    }
    let url = draft.base_url.trim();
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("API Base URL 必须以 http:// 或 https:// 开头".to_string());
    }
    if draft.model_id.trim().is_empty() {
        return Err("请填写模型 ID".to_string());
    }
    if draft.endpoint_mode == AIEndpointMode::CustomPath
        && clean_optional(draft.custom_path.as_deref()).is_none()
    {
        return Err("请填写自定义请求路径".to_string());
    }
    if draft.auth_type == AIAuthType::CustomHeader
        && clean_optional(draft.auth_header.as_deref()).is_none()
    {
        return Err("请填写认证 Header 名称".to_string());
    }
    Ok(())
}

fn normalize_order(models: &mut [AIModelConfig]) {
    models.sort_by_key(|model| model.order);
    assign_order(models);
}

fn assign_order(models: &mut [AIModelConfig]) {
    for (index, model) in models.iter_mut().enumerate() {
        model.order = index as u32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft(name: &str) -> AIModelDraft {
        AIModelDraft {
            id: None,
            name: name.to_string(),
            protocol: AIProtocol::OpenAIChat,
            base_url: "https://example.com/v1".to_string(),
            model_id: "model".to_string(),
            endpoint_mode: AIEndpointMode::Auto,
            custom_path: None,
            auth_type: AIAuthType::Bearer,
            auth_header: None,
            timeout_secs: 60,
            enabled: true,
            api_key: None,
            clear_api_key: false,
        }
    }

    #[test]
    fn creates_and_reorders_models() {
        let dir =
            std::env::temp_dir().join(format!("snapscribe-ai-store-{}", uuid::Uuid::new_v4()));
        let store = AIServiceStore::new(dir.clone());
        let first = store.create_model(&draft("one")).unwrap();
        let second = store.create_model(&draft("two")).unwrap();
        let config = store
            .reorder_models(&[second.id.clone(), first.id.clone()])
            .unwrap();
        assert_eq!(config.models[0].name, "two");
        assert_eq!(config.models[0].order, 0);
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn ensures_glm_preset_once_and_keeps_existing_order() {
        let dir =
            std::env::temp_dir().join(format!("snapscribe-ai-preset-{}", uuid::Uuid::new_v4()));
        let store = AIServiceStore::new(dir.clone());
        store.create_model(&draft("existing-a")).unwrap();
        store.create_model(&draft("existing-b")).unwrap();

        let first = store.ensure_default_presets().unwrap();
        let second = store.ensure_default_presets().unwrap();

        assert_eq!(
            second
                .models
                .iter()
                .filter(|model| model.preset_key.as_deref() == Some("glm-4.7-flash"))
                .count(),
            1
        );
        assert_eq!(first.models[0].model_id, "glm-4.7-flash");
        assert_eq!(first.models[1].name, "existing-a");
        assert_eq!(first.models[2].name, "existing-b");
        std::fs::remove_dir_all(dir).ok();
    }
}
