use std::path::PathBuf;
use std::sync::Arc;

use eh_client::{
    AuthStatus, ClientConfig, CookieLogin, EhClient, FavoriteCategory, GalleryDetail, GalleryKey,
    GalleryListPage, ListRequest, PasswordLogin, ThumbnailPage,
};
use tokio::sync::broadcast;

use crate::download::DownloadManager;
use crate::error::{IpcError, Result};
use crate::events::{CoreEvent, EventSink};
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
use crate::source::PageSource;
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
    sink: Arc<dyn EventSink>,
    broadcast_tx: broadcast::Sender<CoreEvent>,
}

impl Core {
    pub fn open(paths: AppPaths, sink: Arc<dyn EventSink>) -> Result<Self> {
        paths.ensure_dirs()?;
        let settings = SettingsStore::open(&paths.config_dir, &paths.download_dir)?;
        let current_settings = settings.get();
        let client_cfg = ClientConfig {
            proxy: current_settings.proxy.clone(),
            site: current_settings.site,
            cookie_path: Some(paths.data_dir.join("cookies.json")),
            ..Default::default()
        };
        let client = EhClient::new(client_cfg).map_err(IpcError::from)?;
        let store = Store::open(&paths.data_dir)?;

        // Restore state: tasks left running become queued
        store.reset_running_downloads_to_queued()?;

        let (broadcast_tx, _) = broadcast::channel(256);
        let source: Arc<dyn PageSource> = Arc::new(client.clone());
        let images = ImageService::new(&paths.cache_dir, source.clone(), store.clone());
        let downloads = DownloadManager::new(
            store.clone(),
            source,
            settings.clone(),
            sink.clone(),
            broadcast_tx.clone(),
        );
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
            sink,
            broadcast_tx,
        })
    }

    pub fn start(&self) {
        self.downloads.start();
    }

    pub fn subscribe(&self) -> broadcast::Receiver<CoreEvent> {
        self.broadcast_tx.subscribe()
    }

    fn emit_event(&self, event: CoreEvent) {
        self.sink.emit(event.clone());
        let _ = self.broadcast_tx.send(event);
    }

    pub async fn auth_status(&self) -> Result<AuthStatus> {
        self.client.auth_status(false).await.map_err(IpcError::from)
    }

    pub async fn login_password(&self, req: PasswordLogin) -> Result<AuthStatus> {
        let status = self
            .client
            .login_password(&req)
            .await
            .map_err(IpcError::from)?;
        self.emit_event(CoreEvent::AuthChanged(status.clone()));
        Ok(status)
    }

    pub async fn login_cookies(&self, req: CookieLogin) -> Result<AuthStatus> {
        let status = self
            .client
            .login_cookies(&req)
            .await
            .map_err(IpcError::from)?;
        self.emit_event(CoreEvent::AuthChanged(status.clone()));
        Ok(status)
    }

    pub async fn logout(&self) -> Result<AuthStatus> {
        self.client.logout().map_err(IpcError::from)?;
        let status = self
            .client
            .auth_status(false)
            .await
            .map_err(IpcError::from)?;
        self.emit_event(CoreEvent::AuthChanged(status.clone()));
        Ok(status)
    }

    pub async fn get_settings(&self) -> Result<Settings> {
        Ok(self.settings.get())
    }

    pub async fn update_settings(&self, req: Settings) -> Result<Settings> {
        let updated = self.settings.update(req)?;
        self.client
            .set_proxy(updated.proxy.clone())
            .map_err(IpcError::from)?;
        self.client.set_site(updated.site);
        Ok(updated)
    }

    pub async fn list_galleries(&self, req: ListRequest) -> Result<GalleryListPage> {
        self.client.list(&req).await.map_err(IpcError::from)
    }

    pub async fn gallery_detail(&self, req: GalleryKey) -> Result<GalleryDetail> {
        self.client
            .gallery_detail(&req)
            .await
            .map_err(IpcError::from)
    }

    pub async fn gallery_thumbnails(&self, req: ThumbnailRequest) -> Result<ThumbnailPage> {
        let key = GalleryKey {
            gid: req.gid,
            token: req.token,
        };
        self.client
            .thumbnails(&key, req.page)
            .await
            .map_err(IpcError::from)
    }

    pub async fn reload_page_image(&self, req: PageImageKey) -> Result<()> {
        self.images.reload(req.gid, &req.token, req.index).await
    }

    pub async fn favorite_categories(&self) -> Result<Vec<FavoriteCategory>> {
        self.client
            .favorite_categories()
            .await
            .map_err(IpcError::from)
    }

    pub async fn set_favorite(&self, req: FavoriteUpdate) -> Result<()> {
        let key = GalleryKey {
            gid: req.gid,
            token: req.token,
        };
        self.client
            .set_favorite(&key, req.slot, &req.note)
            .await
            .map_err(IpcError::from)
    }

    pub async fn batch_favorites(&self, req: BatchFavorite) -> Result<()> {
        self.client
            .batch_favorites(&req.gids, req.slot)
            .await
            .map_err(IpcError::from)
    }

    pub async fn enqueue_downloads(&self, req: EnqueueRequest) -> Result<Vec<DownloadTask>> {
        self.downloads.enqueue(req).await
    }

    pub async fn list_downloads(&self) -> Result<Vec<DownloadTask>> {
        self.downloads.list()
    }

    pub async fn pause_download(&self, req: TaskId) -> Result<()> {
        self.downloads.pause(req.id).await
    }

    pub async fn resume_download(&self, req: TaskId) -> Result<()> {
        self.downloads.resume(req.id).await
    }

    pub async fn remove_download(&self, req: RemoveDownload) -> Result<()> {
        self.downloads.remove(req.id, req.delete_files).await
    }

    pub async fn pause_all(&self) -> Result<()> {
        self.downloads.pause_all().await
    }

    pub async fn resume_all(&self) -> Result<()> {
        self.downloads.resume_all().await
    }

    pub async fn list_library(&self) -> Result<Vec<LocalGallery>> {
        self.library.list()
    }

    pub async fn rescan_library(&self) -> Result<Vec<LocalGallery>> {
        let s = self.settings.get();
        let mut roots = vec![PathBuf::from(&s.download_dir)];
        roots.extend(s.library_dirs.iter().map(PathBuf::from));

        let (res, warnings) = self.library.rescan_report(&roots)?;
        for w in warnings {
            tracing::warn!("skipped {}: {}", w.path.display(), w.reason);
        }
        self.emit_event(CoreEvent::LibraryChanged);
        Ok(res)
    }

    pub async fn local_pages(&self, req: LocalId) -> Result<LocalPages> {
        self.library.pages(&req.id)
    }

    pub async fn add_library_dir(&self, req: PathRequest) -> Result<Settings> {
        let mut s = self.settings.get();
        if !s.library_dirs.contains(&req.path) {
            s.library_dirs.push(req.path);
        }
        let updated = self.settings.update(s)?;
        let _ = self.rescan_library().await;
        Ok(updated)
    }

    pub async fn remove_local(&self, req: RemoveLocal) -> Result<()> {
        let s = self.settings.get();
        let mut roots = vec![PathBuf::from(&s.download_dir)];
        roots.extend(s.library_dirs.iter().map(PathBuf::from));

        self.library.remove(&req.id, req.delete_files, &roots)?;
        self.emit_event(CoreEvent::LibraryChanged);
        Ok(())
    }

    pub async fn list_history(&self) -> Result<Vec<HistoryEntry>> {
        self.history.list()
    }

    pub async fn record_progress(&self, req: ReadProgress) -> Result<()> {
        self.history.record_progress(req)
    }

    pub async fn clear_history(&self) -> Result<()> {
        self.history.clear()
    }

    pub async fn open_path(&self, req: PathRequest) -> Result<()> {
        let path = PathBuf::from(req.path);
        if !path.exists() {
            return Err(IpcError::not_found("Path not found"));
        }
        Ok(())
    }

    pub async fn clear_image_cache(&self) -> Result<CacheCleared> {
        self.images.clear_cache()
    }
}
