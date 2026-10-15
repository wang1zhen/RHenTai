use eh_client::{
    AuthStatus, CookieLogin, FavoriteCategory, GalleryDetail, GalleryKey, GalleryListPage,
    ListRequest, PasswordLogin, ThumbnailPage,
};
use rhentai_core::{
    BatchFavorite, CacheCleared, Core, DownloadTask, EnqueueRequest, FavoriteUpdate, HistoryEntry,
    IpcError, LocalGallery, LocalId, LocalPages, PageImageKey, PathRequest, ReadProgress,
    RemoveDownload, RemoveLocal, Settings, TaskId, ThumbnailRequest,
};

#[tauri::command]
pub async fn auth_status(core: tauri::State<'_, Core>) -> Result<AuthStatus, IpcError> {
    core.auth_status().await
}

#[tauri::command]
pub async fn login_password(
    core: tauri::State<'_, Core>,
    req: PasswordLogin,
) -> Result<AuthStatus, IpcError> {
    core.login_password(req).await
}

#[tauri::command]
pub async fn login_cookies(
    core: tauri::State<'_, Core>,
    req: CookieLogin,
) -> Result<AuthStatus, IpcError> {
    core.login_cookies(req).await
}

#[cfg(desktop)]
#[tauri::command]
pub async fn login_web(
    app: tauri::AppHandle,
    core: tauri::State<'_, Core>,
) -> Result<(), IpcError> {
    crate::login_window::login_web(&app, &core).await
}

#[cfg(not(desktop))]
#[tauri::command]
pub async fn login_web(_core: tauri::State<'_, Core>) -> Result<(), IpcError> {
    Err(IpcError::invalid_input(
        "login_web is not supported on mobile",
    ))
}

#[tauri::command]
pub async fn logout(core: tauri::State<'_, Core>) -> Result<AuthStatus, IpcError> {
    core.logout().await
}

#[tauri::command]
pub async fn get_settings(core: tauri::State<'_, Core>) -> Result<Settings, IpcError> {
    core.get_settings().await
}

#[tauri::command]
pub async fn update_settings(
    core: tauri::State<'_, Core>,
    req: Settings,
) -> Result<Settings, IpcError> {
    core.update_settings(req).await
}

#[tauri::command]
pub async fn list_galleries(
    core: tauri::State<'_, Core>,
    req: ListRequest,
) -> Result<GalleryListPage, IpcError> {
    core.list_galleries(req).await
}

#[tauri::command]
pub async fn gallery_detail(
    core: tauri::State<'_, Core>,
    req: GalleryKey,
) -> Result<GalleryDetail, IpcError> {
    core.gallery_detail(req).await
}

#[tauri::command]
pub async fn gallery_thumbnails(
    core: tauri::State<'_, Core>,
    req: ThumbnailRequest,
) -> Result<ThumbnailPage, IpcError> {
    core.gallery_thumbnails(req).await
}

#[tauri::command]
pub async fn reload_page_image(
    core: tauri::State<'_, Core>,
    req: PageImageKey,
) -> Result<(), IpcError> {
    core.reload_page_image(req).await
}

#[tauri::command]
pub async fn favorite_categories(
    core: tauri::State<'_, Core>,
) -> Result<Vec<FavoriteCategory>, IpcError> {
    core.favorite_categories().await
}

#[tauri::command]
pub async fn set_favorite(
    core: tauri::State<'_, Core>,
    req: FavoriteUpdate,
) -> Result<(), IpcError> {
    core.set_favorite(req).await
}

#[tauri::command]
pub async fn batch_favorites(
    core: tauri::State<'_, Core>,
    req: BatchFavorite,
) -> Result<(), IpcError> {
    core.batch_favorites(req).await
}

#[tauri::command]
pub async fn enqueue_downloads(
    core: tauri::State<'_, Core>,
    req: EnqueueRequest,
) -> Result<Vec<DownloadTask>, IpcError> {
    core.enqueue_downloads(req).await
}

#[tauri::command]
pub async fn list_downloads(core: tauri::State<'_, Core>) -> Result<Vec<DownloadTask>, IpcError> {
    core.list_downloads().await
}

#[tauri::command]
pub async fn pause_download(core: tauri::State<'_, Core>, req: TaskId) -> Result<(), IpcError> {
    core.pause_download(req).await
}

#[tauri::command]
pub async fn resume_download(core: tauri::State<'_, Core>, req: TaskId) -> Result<(), IpcError> {
    core.resume_download(req).await
}

#[tauri::command]
pub async fn remove_download(
    core: tauri::State<'_, Core>,
    req: RemoveDownload,
) -> Result<(), IpcError> {
    core.remove_download(req).await
}

#[tauri::command]
pub async fn pause_all(core: tauri::State<'_, Core>) -> Result<(), IpcError> {
    core.pause_all().await
}

#[tauri::command]
pub async fn resume_all(core: tauri::State<'_, Core>) -> Result<(), IpcError> {
    core.resume_all().await
}

#[tauri::command]
pub async fn list_library(core: tauri::State<'_, Core>) -> Result<Vec<LocalGallery>, IpcError> {
    core.list_library().await
}

#[tauri::command]
pub async fn rescan_library(core: tauri::State<'_, Core>) -> Result<Vec<LocalGallery>, IpcError> {
    core.rescan_library().await
}

#[tauri::command]
pub async fn local_pages(
    core: tauri::State<'_, Core>,
    req: LocalId,
) -> Result<LocalPages, IpcError> {
    core.local_pages(req).await
}

#[tauri::command]
pub async fn add_library_dir(
    core: tauri::State<'_, Core>,
    req: PathRequest,
) -> Result<Settings, IpcError> {
    core.add_library_dir(req).await
}

#[tauri::command]
pub async fn remove_local(core: tauri::State<'_, Core>, req: RemoveLocal) -> Result<(), IpcError> {
    core.remove_local(req).await
}

#[tauri::command]
pub async fn list_history(core: tauri::State<'_, Core>) -> Result<Vec<HistoryEntry>, IpcError> {
    core.list_history().await
}

#[tauri::command]
pub async fn record_progress(
    core: tauri::State<'_, Core>,
    req: ReadProgress,
) -> Result<(), IpcError> {
    core.record_progress(req).await
}

#[tauri::command]
pub async fn clear_history(core: tauri::State<'_, Core>) -> Result<(), IpcError> {
    core.clear_history().await
}

#[cfg(desktop)]
#[tauri::command]
pub async fn open_path(core: tauri::State<'_, Core>, req: PathRequest) -> Result<(), IpcError> {
    core.open_path(req.clone()).await?;
    let path = req.path;
    tokio::task::spawn_blocking(move || {
        tauri_plugin_opener::reveal_item_in_dir(&path)
            .or_else(|_| tauri_plugin_opener::open_path(&path, None::<&str>))
            .map_err(|e| IpcError::io(e.to_string()))
    })
    .await
    .map_err(|e| IpcError::internal(e.to_string()))?
}

#[cfg(not(desktop))]
#[tauri::command]
pub async fn open_path(_core: tauri::State<'_, Core>, _req: PathRequest) -> Result<(), IpcError> {
    Err(IpcError::invalid_input(
        "open_path is not supported on mobile",
    ))
}

#[tauri::command]
pub async fn clear_image_cache(core: tauri::State<'_, Core>) -> Result<CacheCleared, IpcError> {
    core.clear_image_cache().await
}
