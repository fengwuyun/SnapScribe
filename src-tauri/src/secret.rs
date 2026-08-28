use std::path::{Path, PathBuf};

use windows_sys::Win32::Foundation::LocalFree;
use windows_sys::Win32::Security::Cryptography::{
    CryptProtectData, CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
};

pub struct SecretStore {
    path: PathBuf,
}

impl SecretStore {
    pub fn new(config_dir: &Path) -> Self {
        Self {
            path: config_dir.join("api-key.dpapi"),
        }
    }

    pub fn for_model(config_dir: &Path, model_id: &str) -> Self {
        let safe_id = model_id
            .chars()
            .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '-')
            .collect::<String>();
        Self {
            path: config_dir
                .join("credentials")
                .join(format!("ai-model-{safe_id}.dpapi")),
        }
    }

    pub fn has_key(&self) -> bool {
        self.path.is_file()
    }

    pub fn save(&self, value: &str) -> Result<(), String> {
        let value = value.trim();
        if value.is_empty() {
            return Err("API Key 不能为空".to_string());
        }
        let encrypted = protect(value.as_bytes())?;
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("无法创建密钥目录：{e}"))?;
        }
        std::fs::write(&self.path, encrypted).map_err(|e| format!("无法保存 API Key：{e}"))
    }

    pub fn load(&self) -> Result<String, String> {
        let encrypted = std::fs::read(&self.path).map_err(|_| "尚未配置 API Key".to_string())?;
        let decrypted = unprotect(&encrypted)?;
        String::from_utf8(decrypted).map_err(|_| "API Key 数据损坏".to_string())
    }

    pub fn clear(&self) -> Result<(), String> {
        if self.path.exists() {
            std::fs::remove_file(&self.path).map_err(|e| format!("无法清除 API Key：{e}"))?;
        }
        Ok(())
    }
}

fn protect(value: &[u8]) -> Result<Vec<u8>, String> {
    let input = CRYPT_INTEGER_BLOB {
        cbData: value.len() as u32,
        pbData: value.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };
    let ok = unsafe {
        CryptProtectData(
            &input,
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
    };
    if ok == 0 {
        return Err("Windows 无法加密 API Key".to_string());
    }
    let bytes =
        unsafe { std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec() };
    unsafe { LocalFree(output.pbData.cast()) };
    Ok(bytes)
}

fn unprotect(value: &[u8]) -> Result<Vec<u8>, String> {
    let input = CRYPT_INTEGER_BLOB {
        cbData: value.len() as u32,
        pbData: value.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };
    let ok = unsafe {
        CryptUnprotectData(
            &input,
            std::ptr::null_mut(),
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
    };
    if ok == 0 {
        return Err("Windows 无法解密 API Key".to_string());
    }
    let bytes =
        unsafe { std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec() };
    unsafe { LocalFree(output.pbData.cast()) };
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::SecretStore;

    #[test]
    fn stores_the_api_key_encrypted_for_the_current_windows_user() {
        let dir = std::env::temp_dir().join(format!("snapscribe-secret-{}", uuid::Uuid::new_v4()));
        let store = SecretStore::new(&dir);
        store.save("secret-value").unwrap();

        let raw = std::fs::read(dir.join("api-key.dpapi")).unwrap();
        assert!(!String::from_utf8_lossy(&raw).contains("secret-value"));
        assert_eq!(store.load().unwrap(), "secret-value");
        store.clear().unwrap();
        assert!(!store.has_key());
        std::fs::remove_dir_all(dir).ok();
    }
}
