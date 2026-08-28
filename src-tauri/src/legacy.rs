use std::collections::BTreeSet;

use tauri::{AppHandle, Manager};

use crate::project_store::ProjectStore;
use crate::settings::SettingsStore;

pub fn migrate(app: &AppHandle) -> Result<usize, String> {
    let config = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("无法确定设置目录：{e}"))?;
    let history = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("无法确定旧记录目录：{e}"))?
        .join("transcripts");
    if !history.is_dir() {
        return Ok(0);
    }
    let ledger_path = config.join("legacy-import.json");
    let mut migrated: BTreeSet<String> = if ledger_path.is_file() {
        serde_json::from_slice(
            &std::fs::read(&ledger_path).map_err(|e| format!("无法读取迁移记录：{e}"))?,
        )
        .map_err(|e| format!("迁移记录损坏：{e}"))?
    } else {
        BTreeSet::new()
    };
    let settings = SettingsStore::new(config.clone()).load()?;
    let store = ProjectStore::new(settings.data_root.into());
    let mut count = 0;
    for entry in std::fs::read_dir(history).map_err(|e| format!("无法读取旧转录目录：{e}"))?
    {
        let entry = entry.map_err(|e| format!("无法读取旧转录记录：{e}"))?;
        let path = entry.path();
        let Some(file_name) = path
            .file_name()
            .and_then(|value| value.to_str())
            .map(str::to_string)
        else {
            continue;
        };
        if migrated.contains(&file_name)
            || !path.is_file()
            || !path
                .extension()
                .is_some_and(|value| value.eq_ignore_ascii_case("txt"))
        {
            continue;
        }
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("无法读取旧转录 {file_name}：{e}"))?;
        let name = path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("旧转录记录");
        store.create_legacy_project(name, &text)?;
        migrated.insert(file_name);
        count += 1;
    }
    if count > 0 {
        std::fs::create_dir_all(&config).map_err(|e| format!("无法创建迁移记录目录：{e}"))?;
        let bytes =
            serde_json::to_vec_pretty(&migrated).map_err(|e| format!("无法生成迁移记录：{e}"))?;
        std::fs::write(ledger_path, bytes).map_err(|e| format!("无法保存迁移记录：{e}"))?;
    }
    Ok(count)
}
