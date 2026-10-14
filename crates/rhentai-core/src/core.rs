use std::sync::Arc;

use eh_client::{
    AuthStatus, ClientConfig, CookieLogin, EhClient, FavoriteCategory, GalleryDetail, GalleryKey,
    GalleryListPage, ListRequest, PasswordLogin, ThumbnailPage,
};

use crate::download::DownloadManager;
use crate::error::{IpcError, Result};
use crate::events::EventSink;
use crate::history::History;
use crate::image::ImageService;
use crate::library::Library;
use crate::model::{
    BatchFavorite, CacheCleared, DownloadTask, EnqueueRequest, FavoriteUpdate, HistoryEntry,
    LocalGallery, LocalId, LocalPages, PageImageKey, PathRequest, ReadProgress, RemoveDownload,
    RemoveLocal, Settings, TaskId, ThumbnailRequest,
};
use crate::paths::AppPaths;
use crate::settings::SettingsStore;
use crate::store::Store;

#[derive(Clone)]
pub struct Core {
    pub paths: AppPaths,
    pub settings: SettingsStore,
    pub client: EhClient,
    pub store: Store,
    pub images: ImageService,
    pub downloads: DownloadManager,
    pub library: Library,
    pub history: History,
}

impl Core {
    pub fn open(paths: AppPaths, sink: Arc<dyn EventSink>) -> Result<Self> {
        paths.ensure_dirs()?;
        let settings = SettingsStore::open(&paths.config_dir)?;
        let current_settings = settings.get();
        let client_cfg = ClientConfig {
            proxy: current_settings.proxy.clone(),
            site: current_settings.site,
            cookie_path: Some(paths.data_dir.join("cookies.json")),
            ..Default::default()
        };
        let client = EhClient::new(client_cfg).map_err(IpcError::from)?;
        let store = Store::open(&paths.data_dir)?;
        let images = ImageService::new(&paths.cache_dir, client.clone());
        let downloads = DownloadManager::new(store.clone(), client.clone(), sink);
        let library = Library::new(store.clone());
        let history = History::new(store.clone());

        Ok(Self {
            paths,
            settings,
            client,
            store,
            images,
            downloads,
            library,
            history,
        })
    }

    pub async fn auth_status(&self) -> Result<AuthStatus> {
        Err(IpcError::internal("not implemented: auth_status"))
    }

    pub async fn login_password(&self, _req: PasswordLogin) -> Result<AuthStatus> {
        Err(IpcError::internal("not implemented: login_password"))
    }

    pub async fn login_cookies(&self, _req: CookieLogin) -> Result<AuthStatus> {
        Err(IpcError::internal("not implemented: login_cookies"))
    }

    pub async fn logout(&self) -> Result<AuthStatus> {
        Err(IpcError::internal("not implemented: logout"))
    }

    pub async fn get_settings(&self) -> Result<Settings> {
        Ok(self.settings.get())
    }

    pub async fn update_settings(&self, req: Settings) -> Result<Settings> {
        self.settings.update(req)
    }

    pub async fn list_galleries(&self, _req: ListRequest) -> Result<GalleryListPage> {
        Err(IpcError::internal("not implemented: list_galleries"))
    }

    pub async fn gallery_detail(&self, _req: GalleryKey) -> Result<GalleryDetail> {
        Err(IpcError::internal("not implemented: gallery_detail"))
    }

    pub async fn gallery_thumbnails(&self, _req: ThumbnailRequest) -> Result<ThumbnailPage> {
        Err(IpcError::internal("not implemented: gallery_thumbnails"))
    }

    pub async fn reload_page_image(&self, _req: PageImageKey) -> Result<()> {
        Err(IpcError::internal("not implemented: reload_page_image"))
    }

    pub async fn favorite_categories(&self) -> Result<Vec<FavoriteCategory>> {
        Err(IpcError::internal("not implemented: favorite_categories"))
    }

    pub async fn set_favorite(&self, _req: FavoriteUpdate) -> Result<()> {
        Err(IpcError::internal("not implemented: set_favorite"))
    }

    pub async fn batch_favorites(&self, _req: BatchFavorite) -> Result<()> {
        Err(IpcError::internal("not implemented: batch_favorites"))
    }

    pub async fn enqueue_downloads(&self, _req: EnqueueRequest) -> Result<Vec<DownloadTask>> {
        Err(IpcError::internal("not implemented: enqueue_downloads"))
    }

    pub async fn list_downloads(&self) -> Result<Vec<DownloadTask>> {
        Err(IpcError::internal("not implemented: list_downloads"))
    }

    pub async fn pause_download(&self, _req: TaskId) -> Result<()> {
        Err(IpcError::internal("not implemented: pause_download"))
    }

    pub async fn resume_download(&self, _req: TaskId) -> Result<()> {
        Err(IpcError::internal("not implemented: resume_download"))
    }

    pub async fn remove_download(&self, _req: RemoveDownload) -> Result<()> {
        Err(IpcError::internal("not implemented: remove_download"))
    }

    pub async fn pause_all(&self) -> Result<()> {
        Err(IpcError::internal("not implemented: pause_all"))
    }

    pub async fn resume_all(&self) -> Result<()> {
        Err(IpcError::internal("not implemented: resume_all"))
    }

    pub async fn list_library(&self) -> Result<Vec<LocalGallery>> {
        Err(IpcError::internal("not implemented: list_library"))
    }

    pub async fn rescan_library(&self) -> Result<Vec<LocalGallery>> {
        Err(IpcError::internal("not implemented: rescan_library"))
    }

    pub async fn local_pages(&self, _req: LocalId) -> Result<LocalPages> {
        Err(IpcError::internal("not implemented: local_pages"))
    }

    pub async fn add_library_dir(&self, req: PathRequest) -> Result<Settings> {
        let mut s = self.settings.get();
        if !s.library_dirs.contains(&req.path) {
            s.library_dirs.push(req.path);
        }
        self.settings.update(s)
    }

    pub async fn remove_local(&self, _req: RemoveLocal) -> Result<()> {
        Err(IpcError::internal("not implemented: remove_local"))
    }

    pub async fn list_history(&self) -> Result<Vec<HistoryEntry>> {
        Err(IpcError::internal("not implemented: list_history"))
    }

    pub async fn record_progress(&self, _req: ReadProgress) -> Result<()> {
        Err(IpcError::internal("not implemented: record_progress"))
    }

    pub async fn clear_history(&self) -> Result<()> {
        Err(IpcError::internal("not implemented: clear_history"))
    }

    pub async fn open_path(&self, _req: PathRequest) -> Result<()> {
        Err(IpcError::internal("not implemented: open_path"))
    }

    pub async fn clear_image_cache(&self) -> Result<CacheCleared> {
        Err(IpcError::internal("not implemented: clear_image_cache"))
    }
}
