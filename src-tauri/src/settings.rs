use std::path::PathBuf;

use crate::model::{AIServiceType, AISettings, AppSettings, ImportStrategy};

pub struct SettingsStore {
    config_dir: PathBuf,
}

impl SettingsStore {
    pub fn new(config_dir: PathBuf) -> Self {
        Self { config_dir }
    }

    pub fn load(&self) -> Result<AppSettings, String> {
        let path = self.config_dir.join("settings.json");
        if !path.exists() {
            return Ok(self.defaults());
        }
        let bytes = std::fs::read(&path).map_err(|e| format!("无法读取设置：{e}"))?;
        serde_json::from_slice(&bytes).map_err(|e| format!("设置文件损坏：{e}"))
    }

    pub fn save(&self, settings: &AppSettings) -> Result<(), String> {
        std::fs::create_dir_all(&self.config_dir).map_err(|e| format!("无法创建设置目录：{e}"))?;
        let target = self.config_dir.join("settings.json");
        let temp = self
            .config_dir
            .join(format!(".settings-{}.tmp", uuid::Uuid::new_v4()));
        let bytes =
            serde_json::to_vec_pretty(settings).map_err(|e| format!("设置序列化失败：{e}"))?;
        std::fs::write(&temp, bytes).map_err(|e| format!("无法写入设置：{e}"))?;
        let backup = self
            .config_dir
            .join(format!(".settings-{}.backup", uuid::Uuid::new_v4()));
        let had_target = target.exists();
        if had_target {
            std::fs::rename(&target, &backup).map_err(|e| format!("无法备份旧设置：{e}"))?;
        }
        if let Err(error) = std::fs::rename(&temp, &target) {
            if had_target {
                let _ = std::fs::rename(&backup, &target);
            }
            let _ = std::fs::remove_file(&temp);
            return Err(format!("无法保存设置：{error}"));
        }
        if had_target {
            std::fs::remove_file(backup).map_err(|e| format!("无法清理设置备份：{e}"))?;
        }
        Ok(())
    }

    pub fn migrate_data_root(&self, old_root: &str, new_root: &str) -> Result<(), String> {
        let old = PathBuf::from(old_root);
        let new = PathBuf::from(new_root);
        if old == new || !old.exists() {
            return Ok(());
        }
        if new.exists()
            && std::fs::read_dir(&new)
                .map_err(|e| format!("无法读取新保存位置：{e}"))?
                .next()
                .is_some()
        {
            return Err("新的转录数据保存位置必须为空目录".to_string());
        }
        let parent = new.parent().ok_or("新的转录数据保存位置无效")?;
        std::fs::create_dir_all(parent).map_err(|e| format!("无法创建新保存位置：{e}"))?;
        let old_canonical = old
            .canonicalize()
            .map_err(|e| format!("无法验证旧保存位置：{e}"))?;
        let parent_canonical = parent
            .canonicalize()
            .map_err(|e| format!("无法验证新保存位置：{e}"))?;
        if parent_canonical.starts_with(&old_canonical) {
            return Err("新的转录数据保存位置不能位于旧位置内部".to_string());
        }
        let staging = parent.join(format!(".snapscribe-migration-{}", uuid::Uuid::new_v4()));
        if let Err(error) = copy_directory(&old, &staging) {
            std::fs::remove_dir_all(&staging).ok();
            return Err(error);
        }
        if new.exists() {
            std::fs::remove_dir(&new).map_err(|e| format!("无法准备新保存位置：{e}"))?;
        }
        if let Err(error) = std::fs::rename(&staging, &new) {
            std::fs::remove_dir_all(&staging).ok();
            return Err(format!("无法启用新保存位置：{error}"));
        }
        Ok(())
    }

    fn defaults(&self) -> AppSettings {
        AppSettings {
            schema_version: 1,
            data_root: self.config_dir.join("data").to_string_lossy().to_string(),
            import_strategy: ImportStrategy::Reference,
            ai: AISettings {
                service_type: AIServiceType::OpenAICompatible,
                base_url: String::new(),
                model: String::new(),
                has_api_key: false,
            },
            api_key: None,
            clear_api_key: false,
        }
    }
}

fn copy_directory(source: &std::path::Path, target: &std::path::Path) -> Result<(), String> {
    std::fs::create_dir_all(target).map_err(|e| format!("无法创建迁移目录：{e}"))?;
    for entry in std::fs::read_dir(source).map_err(|e| format!("无法读取旧数据目录：{e}"))?
    {
        let entry = entry.map_err(|e| format!("无法读取旧数据项：{e}"))?;
        let destination = target.join(entry.file_name());
        if entry
            .file_type()
            .map_err(|e| format!("无法识别旧数据项：{e}"))?
            .is_dir()
        {
            copy_directory(&entry.path(), &destination)?;
        } else {
            std::fs::copy(entry.path(), destination).map_err(|e| format!("迁移数据失败：{e}"))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::SettingsStore;
    use crate::model::{AIServiceType, ImportStrategy};

    fn temp_config(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "snapscribe-settings-{label}-{}",
            uuid::Uuid::new_v4()
        ))
    }

    #[test]
    fn defaults_to_reference_import_and_local_data_directory() {
        let config = temp_config("default");
        let store = SettingsStore::new(config.clone());

        let settings = store.load().unwrap();

        assert_eq!(settings.import_strategy, ImportStrategy::Reference);
        assert_eq!(settings.data_root, config.join("data").to_string_lossy());
        assert_eq!(settings.ai.service_type, AIServiceType::OpenAICompatible);
        assert!(!settings.ai.has_api_key);
        std::fs::remove_dir_all(config).ok();
    }

    #[test]
    fn saves_only_when_save_is_called() {
        let config = temp_config("explicit-save");
        let store = SettingsStore::new(config.clone());
        let mut draft = store.load().unwrap();
        draft.import_strategy = ImportStrategy::Copy;

        let before_save = store.load().unwrap();
        assert_eq!(before_save.import_strategy, ImportStrategy::Reference);

        store.save(&draft).unwrap();
        let after_save = store.load().unwrap();
        assert_eq!(after_save.import_strategy, ImportStrategy::Copy);
        std::fs::remove_dir_all(config).ok();
    }

    #[test]
    fn migrates_data_to_an_empty_directory_and_keeps_the_source() {
        let config = temp_config("migration");
        let old = config.join("old-data");
        let new = config.join("new-data");
        std::fs::create_dir_all(old.join("projects/p1")).unwrap();
        std::fs::write(old.join("projects/p1/project.json"), "project").unwrap();
        let store = SettingsStore::new(config.clone());

        store
            .migrate_data_root(old.to_str().unwrap(), new.to_str().unwrap())
            .unwrap();

        assert_eq!(
            std::fs::read_to_string(new.join("projects/p1/project.json")).unwrap(),
            "project"
        );
        assert!(old.join("projects/p1/project.json").is_file());
        std::fs::remove_dir_all(config).ok();
    }

    #[test]
    fn rejects_migration_into_the_existing_data_tree() {
        let config = temp_config("nested-migration");
        let old = config.join("old-data");
        let nested = old.join("nested/new-data");
        std::fs::create_dir_all(&old).unwrap();
        let store = SettingsStore::new(config.clone());

        assert!(store
            .migrate_data_root(old.to_str().unwrap(), nested.to_str().unwrap())
            .is_err());

        std::fs::remove_dir_all(config).ok();
    }
}
