use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub data_dir: PathBuf,
    pub config_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub download_dir: PathBuf,
}

impl AppPaths {
    pub fn new(
        data_dir: PathBuf,
        config_dir: PathBuf,
        cache_dir: PathBuf,
        download_dir: PathBuf,
    ) -> Self {
        Self {
            data_dir,
            config_dir,
            cache_dir,
            download_dir,
        }
    }

    pub fn from_dirs() -> Self {
        let data_dir = dirs::data_dir()
            .map(|d| d.join("com.rhentai.app"))
            .unwrap_or_else(|| PathBuf::from("data"));
        let config_dir = dirs::config_dir()
            .map(|c| c.join("com.rhentai.app"))
            .unwrap_or_else(|| PathBuf::from("config"));
        let cache_dir = dirs::cache_dir()
            .map(|c| c.join("com.rhentai.app"))
            .unwrap_or_else(|| PathBuf::from("cache"));
        let download_dir = dirs::download_dir()
            .map(|d| d.join("RHenTai"))
            .unwrap_or_else(|| PathBuf::from("downloads"));
        Self {
            data_dir,
            config_dir,
            cache_dir,
            download_dir,
        }
    }

    pub fn with_data_dir_override(mut self, data_dir: impl AsRef<Path>) -> Self {
        let p = data_dir.as_ref().to_path_buf();
        self.data_dir = p.clone();
        self.config_dir = p.join("config");
        self.cache_dir = p.join("cache");
        self
    }

    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.data_dir)?;
        std::fs::create_dir_all(&self.config_dir)?;
        std::fs::create_dir_all(&self.cache_dir)?;
        std::fs::create_dir_all(&self.download_dir)?;
        Ok(())
    }
}
