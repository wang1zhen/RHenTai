use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use crate::error::{IpcError, Result};
use crate::model::Settings;

#[derive(Clone)]
pub struct SettingsStore {
    path: PathBuf,
    settings: Arc<RwLock<Settings>>,
}

impl SettingsStore {
    pub fn open(config_dir: &Path) -> Result<Self> {
        let path = config_dir.join("settings.json");
        let settings = if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => serde_json::from_str::<Settings>(&content).unwrap_or_default(),
                Err(_) => Settings::default(),
            }
        } else {
            let s = Settings::default();
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Ok(json) = serde_json::to_string_pretty(&s) {
                let _ = std::fs::write(&path, json);
            }
            s
        };

        Ok(Self {
            path,
            settings: Arc::new(RwLock::new(settings)),
        })
    }

    pub fn get(&self) -> Settings {
        self.settings.read().unwrap().clone()
    }

    pub fn update(&self, mut new_settings: Settings) -> Result<Settings> {
        // Validate proxy scheme if present
        if let Some(ref proxy) = new_settings.proxy {
            let valid = proxy.starts_with("http://")
                || proxy.starts_with("https://")
                || proxy.starts_with("socks5://")
                || proxy.starts_with("socks5h://");
            if !valid {
                return Err(IpcError::invalid_input(
                    "Proxy scheme must be http://, https://, socks5://, or socks5h://",
                ));
            }
        }

        // Clamp download concurrency to 1..=10
        new_settings.download_concurrency = new_settings.download_concurrency.clamp(1, 10);

        // Ensure download_dir exists
        let _ = std::fs::create_dir_all(&new_settings.download_dir);

        // Save to disk
        if let Some(parent) = self.path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let json = serde_json::to_string_pretty(&new_settings)
            .map_err(|e| IpcError::new(eh_client::ErrorKind::Parse, e.to_string()))?;
        std::fs::write(&self.path, json)?;

        *self.settings.write().unwrap() = new_settings.clone();
        Ok(new_settings)
    }

    pub fn save(&self) -> Result<()> {
        let current = self.get();
        if let Some(parent) = self.path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let json = serde_json::to_string_pretty(&current)
            .map_err(|e| IpcError::new(eh_client::ErrorKind::Parse, e.to_string()))?;
        std::fs::write(&self.path, json)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_store_save_load() {
        let temp_dir = std::env::temp_dir().join(format!("rhentai_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);
        let store = SettingsStore::open(&temp_dir).unwrap();
        let s = store.get();
        assert_eq!(s.download_concurrency, 4);

        let mut updated = s.clone();
        updated.download_concurrency = 8;
        updated.proxy = Some("socks5://127.0.0.1:1080".to_string());
        store.update(updated).unwrap();

        // Re-open from disk
        let store2 = SettingsStore::open(&temp_dir).unwrap();
        assert_eq!(store2.get().download_concurrency, 8);
        assert_eq!(
            store2.get().proxy.as_deref(),
            Some("socks5://127.0.0.1:1080")
        );

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_invalid_proxy() {
        let temp_dir =
            std::env::temp_dir().join(format!("rhentai_test_proxy_{}", std::process::id()));
        let store = SettingsStore::open(&temp_dir).unwrap();
        let mut s = store.get();
        s.proxy = Some("ftp://bad.com".to_string());
        assert!(store.update(s).is_err());
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
