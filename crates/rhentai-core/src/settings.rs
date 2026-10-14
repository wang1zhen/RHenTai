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
    pub fn open(config_dir: &Path, default_download_dir: &Path) -> Result<Self> {
        let path = config_dir.join("settings.json");
        let default_download_str = default_download_dir.to_string_lossy().to_string();
        let settings = if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    let mut s =
                        serde_json::from_str::<Settings>(&content).unwrap_or_else(|_| Settings {
                            download_dir: default_download_str.clone(),
                            ..Default::default()
                        });
                    if s.download_dir.is_empty() {
                        s.download_dir = default_download_str;
                    }
                    s
                }
                Err(_) => Settings {
                    download_dir: default_download_str,
                    ..Default::default()
                },
            }
        } else {
            let s = Settings {
                download_dir: default_download_str,
                ..Default::default()
            };
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
            let trimmed = proxy.trim();
            if trimmed.is_empty() {
                new_settings.proxy = None;
            } else {
                let valid = trimmed.starts_with("http://")
                    || trimmed.starts_with("https://")
                    || trimmed.starts_with("socks5://")
                    || trimmed.starts_with("socks5h://");
                if !valid {
                    return Err(IpcError::invalid_input(
                        "Proxy scheme must be http://, https://, socks5://, or socks5h://",
                    ));
                }
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
        let default_download = temp_dir.join("downloads");
        let _ = std::fs::remove_dir_all(&temp_dir);
        let store = SettingsStore::open(&temp_dir, &default_download).unwrap();
        let s = store.get();
        assert_eq!(s.download_concurrency, 4);
        assert_eq!(s.download_dir, default_download.to_string_lossy());

        let mut updated = s.clone();
        updated.download_concurrency = 8;
        updated.proxy = Some("socks5://127.0.0.1:1080".to_string());
        store.update(updated).unwrap();

        // Re-open from disk
        let store2 = SettingsStore::open(&temp_dir, &default_download).unwrap();
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
        let default_download = temp_dir.join("downloads");
        let store = SettingsStore::open(&temp_dir, &default_download).unwrap();
        let mut s = store.get();
        s.proxy = Some("ftp://bad.com".to_string());
        assert!(store.update(s).is_err());
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_concurrency_clamping() {
        let temp_dir =
            std::env::temp_dir().join(format!("rhentai_test_clamp_{}", std::process::id()));
        let default_download = temp_dir.join("downloads");
        let store = SettingsStore::open(&temp_dir, &default_download).unwrap();
        let mut s = store.get();
        s.download_concurrency = 0;
        let updated = store.update(s).unwrap();
        assert_eq!(updated.download_concurrency, 1);

        let mut s2 = updated.clone();
        s2.download_concurrency = 99;
        let updated2 = store.update(s2).unwrap();
        assert_eq!(updated2.download_concurrency, 10);
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
