pub use eh_client::{Category, GalleryKey, Site, Tag};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS, Default)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS, Default)]
#[ts(export)]
pub enum UiLanguage {
    #[serde(rename = "zh-CN")]
    #[ts(rename = "zh-CN")]
    #[default]
    ZhCn,
    #[serde(rename = "en")]
    #[ts(rename = "en")]
    En,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS, Default)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum ReaderMode {
    #[default]
    Vertical,
    Paged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS, Default)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum ReadingDirection {
    #[default]
    Ltr,
    Rtl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS, Default)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum FitMode {
    #[default]
    Width,
    Height,
    Original,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ReaderSettings {
    pub mode: ReaderMode,
    pub direction: ReadingDirection,
    pub fit: FitMode,
    pub preload: u8,
}

impl Default for ReaderSettings {
    fn default() -> Self {
        Self {
            mode: ReaderMode::Vertical,
            direction: ReadingDirection::Ltr,
            fit: FitMode::Width,
            preload: 3,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", default)]
#[ts(export)]
pub struct Settings {
    pub site: Site,
    pub proxy: Option<String>,
    pub download_dir: String,
    pub download_concurrency: u8,
    pub download_original: bool,
    pub library_dirs: Vec<String>,
    pub reader: ReaderSettings,
    pub theme: Theme,
    pub language: UiLanguage,
}

impl Default for Settings {
    fn default() -> Self {
        let download_dir = dirs::download_dir()
            .map(|p| p.join("RHenTai").to_string_lossy().to_string())
            .unwrap_or_else(|| "downloads".to_string());
        Self {
            site: Site::Eh,
            proxy: None,
            download_dir,
            download_concurrency: 4,
            download_original: false,
            library_dirs: Vec::new(),
            reader: ReaderSettings::default(),
            theme: Theme::System,
            language: UiLanguage::ZhCn,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum DownloadState {
    Queued,
    Running,
    Paused,
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DownloadTask {
    #[ts(type = "number")]
    pub id: i64,
    #[ts(type = "number")]
    pub gid: u64,
    pub token: String,
    pub site: Site,
    pub title: String,
    pub thumb: String,
    pub category: Category,
    pub total: u32,
    pub done: u32,
    pub failed: u32,
    pub state: DownloadState,
    pub error: Option<String>,
    pub dir: String,
    pub original: bool,
    #[ts(type = "number")]
    pub created_at: i64,
    #[ts(type = "number")]
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DownloadProgress {
    #[ts(type = "number")]
    pub id: i64,
    pub done: u32,
    pub total: u32,
    pub failed: u32,
    pub state: DownloadState,
    pub error: Option<String>,
    #[ts(type = "number")]
    pub bytes_per_sec: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum LocalKind {
    Folder,
    Archive,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LocalGallery {
    pub id: String,
    pub title: String,
    pub path: String,
    pub kind: LocalKind,
    pub page_count: u32,
    #[ts(type = "number | null")]
    pub gid: Option<u64>,
    pub token: Option<String>,
    pub category: Option<Category>,
    pub tags: Vec<Tag>,
    pub downloaded: bool,
    #[ts(type = "number")]
    pub added_at: i64,
    pub last_read_page: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LocalPages {
    pub count: u32,
    pub names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum ReadTarget {
    Online {
        #[ts(type = "number")]
        gid: u64,
        token: String,
    },
    Local {
        id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct HistoryEntry {
    pub target: ReadTarget,
    pub title: String,
    pub thumb: Option<String>,
    pub category: Option<Category>,
    pub page_count: u32,
    pub last_page: u32,
    #[ts(type = "number")]
    pub read_at: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ReadProgress {
    pub target: ReadTarget,
    pub page: u32,
    pub page_count: u32,
    pub title: String,
    pub thumb: Option<String>,
    pub category: Option<Category>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ThumbnailRequest {
    #[ts(type = "number")]
    pub gid: u64,
    pub token: String,
    pub page: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PageImageKey {
    #[ts(type = "number")]
    pub gid: u64,
    pub token: String,
    pub index: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FavoriteUpdate {
    #[ts(type = "number")]
    pub gid: u64,
    pub token: String,
    pub slot: Option<u8>,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BatchFavorite {
    #[ts(type = "number[]")]
    pub gids: Vec<u64>,
    pub slot: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct EnqueueRequest {
    pub galleries: Vec<GalleryKey>,
    pub original: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TaskId {
    #[ts(type = "number")]
    pub id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RemoveDownload {
    #[ts(type = "number")]
    pub id: i64,
    pub delete_files: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LocalId {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RemoveLocal {
    pub id: String,
    pub delete_files: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PathRequest {
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CacheCleared {
    #[ts(type = "number")]
    pub bytes: u64,
}
