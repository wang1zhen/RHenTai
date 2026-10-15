use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock as StdRwLock};
use std::time::{Duration, Instant};

use eh_client::{ErrorKind, GalleryKey, PageImage, PageRef};
use tokio::sync::{Mutex, Notify, Semaphore, broadcast};
use tokio_util::sync::CancellationToken;

use crate::error::{IpcError, Result};
use crate::events::{CoreEvent, EventSink};
use crate::library::{RhentaiMetadata, compute_local_id};
use crate::model::{
    Category, DownloadProgress, DownloadState, DownloadTask, EnqueueRequest, LocalGallery,
    LocalKind,
};
use crate::sanitizer::sanitize_title;
use crate::settings::SettingsStore;
use crate::source::PageSource;
use crate::store::{Store, current_time_secs};

struct ProgressTracker {
    last_emit: Instant,
    bytes_since_last: u64,
}

#[derive(Clone)]
pub struct DownloadManager {
    store: Store,
    source: Arc<dyn PageSource>,
    settings: SettingsStore,
    sink: Arc<dyn EventSink>,
    broadcast_tx: broadcast::Sender<CoreEvent>,
    scheduler_started: Arc<AtomicBool>,
    scheduler_notify: Arc<Notify>,
    task_tokens: Arc<Mutex<HashMap<i64, CancellationToken>>>,
    image_semaphore: Arc<StdRwLock<Arc<Semaphore>>>,
    override_dir: Arc<StdRwLock<Option<PathBuf>>>,
    running_task_ids: Arc<Mutex<Vec<i64>>>,
}

impl DownloadManager {
    pub fn new(
        store: Store,
        source: Arc<dyn PageSource>,
        settings: SettingsStore,
        sink: Arc<dyn EventSink>,
        broadcast_tx: broadcast::Sender<CoreEvent>,
    ) -> Self {
        let concurrency = settings.get().download_concurrency.clamp(1, 10) as usize;
        Self {
            store,
            source,
            settings,
            sink,
            broadcast_tx,
            scheduler_started: Arc::new(AtomicBool::new(false)),
            scheduler_notify: Arc::new(Notify::new()),
            task_tokens: Arc::new(Mutex::new(HashMap::new())),
            image_semaphore: Arc::new(StdRwLock::new(Arc::new(Semaphore::new(concurrency)))),
            override_dir: Arc::new(StdRwLock::new(None)),
            running_task_ids: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn set_run_overrides(&self, concurrency: Option<u8>, dir: Option<PathBuf>) {
        if let Some(c) = concurrency {
            let clamped = c.clamp(1, 10) as usize;
            *self.image_semaphore.write().unwrap() = Arc::new(Semaphore::new(clamped));
        }
        if let Some(d) = dir {
            *self.override_dir.write().unwrap() = Some(d);
        }
    }

    pub fn start(&self) {
        if self.scheduler_started.swap(true, Ordering::SeqCst) {
            return;
        }

        let mgr = self.clone();
        tokio::spawn(async move {
            mgr.scheduler_loop().await;
        });
    }

    async fn scheduler_loop(&self) {
        loop {
            // At most 2 galleries running concurrently
            let running_count = self.running_task_ids.lock().await.len();
            if running_count < 2
                && let Ok(tasks) = self.store.list_downloads()
            {
                // Find queued tasks
                let mut queued: Vec<DownloadTask> = tasks
                    .into_iter()
                    .filter(|t| t.state == DownloadState::Queued)
                    .collect();
                queued.sort_by_key(|t| t.created_at);

                if let Some(next_task) = queued.first() {
                    let id = next_task.id;
                    let mut running_ids = self.running_task_ids.lock().await;
                    if running_ids.len() < 2 && !running_ids.contains(&id) {
                        running_ids.push(id);
                        drop(running_ids);

                        let _ = self
                            .store
                            .update_download_state(id, DownloadState::Running, None);
                        self.emit_progress(DownloadProgress {
                            id,
                            done: next_task.done,
                            total: next_task.total,
                            failed: next_task.failed,
                            state: DownloadState::Running,
                            error: None,
                            bytes_per_sec: 0,
                        });
                        self.emit_event(CoreEvent::DownloadsChanged);

                        let mgr = self.clone();
                        let task_to_run = next_task.clone();
                        tokio::spawn(async move {
                            mgr.run_task(task_to_run).await;
                        });
                        continue;
                    }
                }
            }

            tokio::select! {
                _ = self.scheduler_notify.notified() => {},
                _ = tokio::time::sleep(Duration::from_millis(500)) => {},
            }
        }
    }

    async fn run_task(&self, mut task: DownloadTask) {
        let id = task.id;
        let token = CancellationToken::new();
        self.task_tokens.lock().await.insert(id, token.clone());

        let res = self.execute_task(&mut task, &token).await;

        self.task_tokens.lock().await.remove(&id);
        self.running_task_ids.lock().await.retain(|&x| x != id);

        match res {
            Ok(completed) => {
                if completed {
                    let _ = self.store.reconcile_download_progress(id);
                    let _ = self
                        .store
                        .update_download_state(id, DownloadState::Completed, None);
                    let t = self.store.get_download(id).ok().flatten().unwrap_or(task);
                    self.emit_progress(DownloadProgress {
                        id,
                        done: t.done,
                        total: t.total,
                        failed: t.failed,
                        state: DownloadState::Completed,
                        error: None,
                        bytes_per_sec: 0,
                    });
                    self.emit_event(CoreEvent::DownloadsChanged);
                    self.emit_event(CoreEvent::LibraryChanged);
                } else if token.is_cancelled() {
                    let _ = self.store.reconcile_download_progress(id);
                    if let Ok(Some(current)) = self.store.get_download(id)
                        && current.state != DownloadState::Queued
                        && current.state != DownloadState::Running
                    {
                        let _ = self
                            .store
                            .update_download_state(id, DownloadState::Paused, None);
                        let t = self.store.get_download(id).ok().flatten().unwrap_or(task);
                        self.emit_progress(DownloadProgress {
                            id,
                            done: t.done,
                            total: t.total,
                            failed: t.failed,
                            state: DownloadState::Paused,
                            error: None,
                            bytes_per_sec: 0,
                        });
                        self.emit_event(CoreEvent::DownloadsChanged);
                    }
                } else {
                    let _ = self.store.reconcile_download_progress(id);
                    let t = self.store.get_download(id).ok().flatten().unwrap_or(task);
                    let last_err_str = self
                        .store
                        .get_download_pages(id)
                        .ok()
                        .and_then(|pages| {
                            pages
                                .into_iter()
                                .filter(|p| p.state == "failed")
                                .filter_map(|p| p.error)
                                .next_back()
                        })
                        .unwrap_or_else(|| "unknown error".to_string());
                    let summary = format!("{} page(s) failed: {}", t.failed, last_err_str);
                    let _ =
                        self.store
                            .update_download_state(id, DownloadState::Failed, Some(&summary));
                    self.emit_progress(DownloadProgress {
                        id,
                        done: t.done,
                        total: t.total,
                        failed: t.failed,
                        state: DownloadState::Failed,
                        error: Some(summary),
                        bytes_per_sec: 0,
                    });
                    self.emit_event(CoreEvent::DownloadsChanged);
                }
            }
            Err(e) => {
                let _ = self.store.reconcile_download_progress(id);
                let t = self.store.get_download(id).ok().flatten().unwrap_or(task);
                if token.is_cancelled() {
                    if let Ok(Some(current)) = self.store.get_download(id)
                        && current.state != DownloadState::Queued
                        && current.state != DownloadState::Running
                    {
                        let _ = self
                            .store
                            .update_download_state(id, DownloadState::Paused, None);
                        self.emit_progress(DownloadProgress {
                            id,
                            done: t.done,
                            total: t.total,
                            failed: t.failed,
                            state: DownloadState::Paused,
                            error: None,
                            bytes_per_sec: 0,
                        });
                        self.emit_event(CoreEvent::DownloadsChanged);
                    }
                } else if e.kind == ErrorKind::ImageLimit || e.kind == ErrorKind::Banned {
                    self.pause_all_with_error(e.kind, &e.message).await;
                } else {
                    let err_msg = e.message.clone();
                    let _ =
                        self.store
                            .update_download_state(id, DownloadState::Failed, Some(&err_msg));
                    self.emit_progress(DownloadProgress {
                        id,
                        done: t.done,
                        total: t.total,
                        failed: t.failed,
                        state: DownloadState::Failed,
                        error: Some(err_msg),
                        bytes_per_sec: 0,
                    });
                    self.emit_event(CoreEvent::DownloadsChanged);
                }
            }
        }

        self.scheduler_notify.notify_one();
    }

    async fn execute_task(
        &self,
        task: &mut DownloadTask,
        token: &CancellationToken,
    ) -> Result<bool> {
        let key = GalleryKey {
            gid: task.gid,
            token: task.token.clone(),
        };

        // 1. Fetch detail if not already populated or if pages table is empty
        let existing_pages = self.store.get_download_pages(task.id)?;
        let mut detail_opt = None;

        if existing_pages.is_empty() || task.total == 0 || task.dir.is_empty() {
            if token.is_cancelled() {
                return Ok(false);
            }

            let detail = tokio::select! {
                res = self.source.gallery_detail(&key) => res?,
                _ = token.cancelled() => return Ok(false),
            };

            let title = detail.summary.title.clone();
            let sanitized = sanitize_title(&title);
            let dir_name = format!("{} - {}", task.gid, sanitized);

            let base_dir = if let Some(ref d) = *self.override_dir.read().unwrap() {
                d.clone()
            } else {
                PathBuf::from(&self.settings.get().download_dir)
            };
            let target_dir = base_dir.join(&dir_name);
            fs::create_dir_all(&target_dir)?;

            task.title = title;
            task.thumb = detail.summary.thumb.clone();
            task.category = detail.summary.category;
            task.total = detail.summary.file_count;
            task.dir = target_dir.to_string_lossy().to_string();

            self.store.update_download_detail(
                task.id,
                &task.title,
                &task.thumb,
                &format!("{:?}", task.category),
                task.total,
                &task.dir,
            )?;

            // Enumerate all PageRefs
            let mut all_pages = detail.thumbnails.pages.clone();
            if detail.thumbnail_page_count > 1 {
                for p in 1..detail.thumbnail_page_count {
                    if token.is_cancelled() {
                        return Ok(false);
                    }
                    let t_page = tokio::select! {
                        res = self.source.thumbnails(&key, p) => res?,
                        _ = token.cancelled() => return Ok(false),
                    };
                    all_pages.extend(t_page.pages);
                }
            }

            self.store
                .insert_download_pages_if_empty(task.id, &all_pages)?;
            detail_opt = Some(detail);
        }

        let target_dir = PathBuf::from(&task.dir);
        fs::create_dir_all(&target_dir)?;

        // Clean up any remaining .part files on startup/resume
        if let Ok(entries) = fs::read_dir(&target_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() && p.extension().and_then(|e| e.to_str()) == Some("part") {
                    let _ = fs::remove_file(p);
                }
            }
        }

        let pages = self.store.get_download_pages(task.id)?;
        task.total = pages.len() as u32;
        task.done = pages.iter().filter(|p| p.state == "done").count() as u32;
        task.failed = pages.iter().filter(|p| p.state == "failed").count() as u32;

        let to_download: Vec<_> = pages.into_iter().filter(|p| p.state != "done").collect();
        if to_download.is_empty() {
            // Already all done!
            self.finalize_completed_task(task, detail_opt.as_ref())
                .await?;
            return Ok(true);
        }

        // Shared progress state
        let task_id = task.id;
        let done_count = Arc::new(std::sync::atomic::AtomicU32::new(task.done));
        let failed_count = Arc::new(std::sync::atomic::AtomicU32::new(task.failed));
        let total_count = task.total;
        let tracker = Arc::new(Mutex::new(ProgressTracker {
            last_emit: Instant::now(),
            bytes_since_last: 0,
        }));

        let mut join_set = tokio::task::JoinSet::new();

        for page_row in to_download {
            if token.is_cancelled() {
                break;
            }

            let sem = self.image_semaphore.read().unwrap().clone();
            let source = self.source.clone();
            let key = key.clone();
            let target_dir = target_dir.clone();
            let original = task.original;
            let token = token.clone();
            let store = self.store.clone();
            let done_count = done_count.clone();
            let failed_count = failed_count.clone();
            let tracker = tracker.clone();
            let mgr = self.clone();

            join_set.spawn(async move {
                let _permit = tokio::select! {
                    p = sem.acquire() => match p {
                        Ok(permit) => permit,
                        Err(_) => return Err(IpcError::internal("Semaphore closed")),
                    },
                    _ = token.cancelled() => return Ok(()),
                };

                if token.is_cancelled() {
                    return Ok(());
                }

                let page_ref = PageRef {
                    index: page_row.idx,
                    page_token: page_row.page_token.unwrap_or_default(),
                    thumb: eh_client::ThumbSprite {
                        url: String::new(),
                        width: 0,
                        height: 0,
                        offset_x: 0,
                        offset_y: 0,
                    },
                    file_name: page_row.file_name.clone(),
                };

                // Up to 5 attempts per page, with reload key
                let mut attempt = 0;
                let mut reload_key: Option<String> = None;
                let mut page_image: Option<PageImage> = None;
                let mut last_err = None;

                while attempt < 5 {
                    if token.is_cancelled() {
                        return Ok(());
                    }
                    attempt += 1;

                    if page_image.is_none() {
                        let page_res = tokio::select! {
                            res = source.page_image(&key, &page_ref, reload_key.as_deref()) => res,
                            _ = token.cancelled() => return Ok(()),
                        };
                        match page_res {
                            Ok(pi) => {
                                if pi.url.contains("509.gif")
                                    || pi.url.contains("509.png")
                                    || pi.url.ends_with("/509.jpg")
                                {
                                    return Err(IpcError::new(
                                        ErrorKind::ImageLimit,
                                        "Image limit exceeded (509)",
                                    ));
                                }
                                reload_key = pi.reload_key.clone();
                                page_image = Some(pi);
                            }
                            Err(e) => {
                                if e.kind == ErrorKind::ImageLimit || e.kind == ErrorKind::Banned {
                                    return Err(e);
                                }
                                last_err = Some(e);
                                continue;
                            }
                        }
                    }

                    let pi = page_image.as_ref().unwrap();
                    let img_url = match (original, pi.original_url.as_ref()) {
                        (true, Some(orig)) => orig,
                        _ => &pi.url,
                    };

                    let fetch_res = tokio::select! {
                        res = source.fetch_image(img_url, None) => res,
                        _ = token.cancelled() => return Ok(()),
                    };

                    match fetch_res {
                        Ok(fetched) => {
                            if fetched.bytes.is_empty() {
                                page_image = None;
                                last_err = Some(IpcError::new(
                                    ErrorKind::Http,
                                    "Invalid image body: empty body",
                                ));
                                continue;
                            }

                            let ext = fetched
                                .content_type
                                .as_deref()
                                .and_then(content_type_to_ext)
                                .or_else(|| {
                                    Path::new(img_url)
                                        .extension()
                                        .and_then(|e| e.to_str())
                                        .map(|s| s.to_string())
                                })
                                .unwrap_or_else(|| "jpg".to_string());

                            let file_name = format!("{:04}.{}", page_row.idx + 1, ext);
                            let part_path = target_dir.join(format!("{file_name}.part"));
                            let final_path = target_dir.join(&file_name);

                            let mut file = File::create(&part_path)?;
                            file.write_all(&fetched.bytes)?;
                            file.sync_all()?;
                            fs::rename(&part_path, &final_path)?;

                            let bytes_len = fetched.bytes.len() as u64;

                            store.update_download_page_state(
                                task_id,
                                page_row.idx,
                                "done",
                                Some(&file_name),
                                None,
                                Some(bytes_len as i64),
                            )?;

                            let d = done_count.fetch_add(1, Ordering::SeqCst) + 1;
                            let f = failed_count.load(Ordering::SeqCst);
                            let _ = store.update_download_progress(task_id, d, f, total_count);
                            mgr.throttled_progress(task_id, d, f, total_count, bytes_len, &tracker)
                                .await;

                            return Ok(());
                        }
                        Err(e) => {
                            if e.kind == ErrorKind::ImageLimit || e.kind == ErrorKind::Banned {
                                return Err(e);
                            }
                            // Every retryable image failure must invalidate it and re-resolve with retained reload key
                            page_image = None;
                            last_err = Some(e);
                        }
                    }
                }

                // If exhausted 5 attempts
                let err_msg = last_err
                    .map(|e| e.message)
                    .unwrap_or_else(|| "Exceeded 5 download attempts".to_string());
                store.update_download_page_state(
                    task_id,
                    page_row.idx,
                    "failed",
                    None,
                    Some(&err_msg),
                    None,
                )?;

                let d = done_count.load(Ordering::SeqCst);
                let f = failed_count.fetch_add(1, Ordering::SeqCst) + 1;
                let _ = store.update_download_progress(task_id, d, f, total_count);
                mgr.throttled_progress(task_id, d, f, total_count, 0, &tracker)
                    .await;

                Ok(())
            });
        }

        while let Some(join_res) = join_set.join_next().await {
            match join_res {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    join_set.abort_all();
                    let _ = self.store.reconcile_download_progress(task.id);
                    return Err(e);
                }
                Err(e) => {
                    join_set.abort_all();
                    let _ = self.store.reconcile_download_progress(task.id);
                    return Err(IpcError::internal(e.to_string()));
                }
            }
        }

        let _ = self.store.reconcile_download_progress(task.id);
        if let Ok(Some(refreshed)) = self.store.get_download(task.id) {
            task.done = refreshed.done;
            task.failed = refreshed.failed;
            task.total = refreshed.total;
        } else {
            task.done = done_count.load(Ordering::SeqCst);
            task.failed = failed_count.load(Ordering::SeqCst);
        }

        if token.is_cancelled() {
            return Ok(false);
        }

        if task.failed == 0 && task.done == task.total {
            self.finalize_completed_task(task, detail_opt.as_ref())
                .await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn throttled_progress(
        &self,
        task_id: i64,
        done: u32,
        failed: u32,
        total: u32,
        bytes_added: u64,
        tracker: &Mutex<ProgressTracker>,
    ) {
        let mut t = tracker.lock().await;
        t.bytes_since_last += bytes_added;
        let elapsed = t.last_emit.elapsed();
        if elapsed >= Duration::from_millis(250) {
            let bytes_per_sec = (t.bytes_since_last as f64 / elapsed.as_secs_f64()).round() as u64;
            t.bytes_since_last = 0;
            t.last_emit = Instant::now();
            drop(t);
            self.emit_progress(DownloadProgress {
                id: task_id,
                done,
                total,
                failed,
                state: DownloadState::Running,
                error: None,
                bytes_per_sec,
            });
        }
    }

    async fn finalize_completed_task(
        &self,
        task: &DownloadTask,
        detail_opt: Option<&eh_client::GalleryDetail>,
    ) -> Result<()> {
        let target_dir = PathBuf::from(&task.dir);

        let mut pages = Vec::new();
        if let Ok(db_pages) = self.store.get_download_pages(task.id) {
            for p in db_pages {
                if let Some(fn_str) = p.file_name {
                    pages.push(fn_str);
                }
            }
        }
        pages.sort_by(|a, b| natord::compare(a, b));

        let fetched_detail = if detail_opt.is_none() {
            let key = GalleryKey {
                gid: task.gid,
                token: task.token.clone(),
            };
            match self.source.gallery_detail(&key).await {
                Ok(d) => Some(d),
                Err(e) => {
                    tracing::warn!(
                        "Failed to fetch gallery detail for task {} during finalization: {}",
                        task.id,
                        e
                    );
                    None
                }
            }
        } else {
            None
        };

        let effective_detail = detail_opt.or(fetched_detail.as_ref());

        let (title_jpn, uploader, posted, tags) = if let Some(d) = effective_detail {
            (
                d.summary.title_jpn.clone(),
                d.summary.uploader.clone(),
                d.summary.posted,
                d.summary.tags.clone(),
            )
        } else {
            (None, None, current_time_secs(), Vec::new())
        };

        let meta = RhentaiMetadata {
            version: 1,
            site: task.site,
            gid: task.gid,
            token: task.token.clone(),
            title: task.title.clone(),
            title_jpn,
            category: task.category,
            uploader,
            posted,
            tags: tags.clone(),
            page_count: pages.len() as u32,
            pages,
            downloaded_at: current_time_secs(),
            original: task.original,
        };

        let json = serde_json::to_string_pretty(&meta)?;
        fs::write(target_dir.join("rhentai.json"), json)?;

        // Upsert library entry
        let canonical = fs::canonicalize(&target_dir).unwrap_or(target_dir);
        let id = compute_local_id(&canonical);
        let mtime = current_time_secs();

        let local_gallery = LocalGallery {
            id,
            title: task.title.clone(),
            path: canonical.to_string_lossy().to_string(),
            kind: LocalKind::Folder,
            page_count: meta.page_count,
            gid: Some(task.gid),
            token: Some(task.token.clone()),
            category: Some(task.category),
            tags,
            downloaded: true,
            added_at: current_time_secs(),
            last_read_page: None,
        };

        self.store.upsert_local_gallery(&local_gallery, mtime)?;
        Ok(())
    }

    pub async fn pause_all_with_error(&self, kind: ErrorKind, msg: &str) {
        let tokens: Vec<_> = self.task_tokens.lock().await.values().cloned().collect();
        for t in tokens {
            t.cancel();
        }

        if let Ok(tasks) = self.store.list_downloads() {
            for t in tasks {
                if t.state == DownloadState::Running || t.state == DownloadState::Queued {
                    let _ =
                        self.store
                            .update_download_state(t.id, DownloadState::Paused, Some(msg));
                    self.emit_progress(DownloadProgress {
                        id: t.id,
                        done: t.done,
                        total: t.total,
                        failed: t.failed,
                        state: DownloadState::Paused,
                        error: Some(format!("{kind:?}: {msg}")),
                        bytes_per_sec: 0,
                    });
                }
            }
        }
        self.emit_event(CoreEvent::DownloadsChanged);
    }

    pub async fn enqueue(&self, req: EnqueueRequest) -> Result<Vec<DownloadTask>> {
        let mut res = Vec::new();
        let default_original = self.settings.get().download_original;
        let original = req.original.unwrap_or(default_original);

        for key in req.galleries {
            if let Some(mut existing) = self.store.get_download_by_gid(key.gid)? {
                let _ = self.store.reconcile_download_progress(existing.id);
                if let Ok(Some(refreshed)) = self.store.get_download(existing.id) {
                    existing = refreshed;
                }
                res.push(existing);
            } else {
                let now = current_time_secs();
                let task = DownloadTask {
                    id: 0,
                    gid: key.gid,
                    token: key.token,
                    site: self.source.site(),
                    title: format!("{}", key.gid),
                    thumb: String::new(),
                    category: Category::Unknown,
                    total: 0,
                    done: 0,
                    failed: 0,
                    state: DownloadState::Queued,
                    error: None,
                    dir: String::new(),
                    original,
                    created_at: now,
                    updated_at: now,
                };
                let id = self.store.insert_download(&task)?;
                let mut created = task;
                created.id = id;
                res.push(created);
            }
        }

        self.emit_event(CoreEvent::DownloadsChanged);
        self.scheduler_notify.notify_one();
        Ok(res)
    }

    pub fn list(&self) -> Result<Vec<DownloadTask>> {
        self.store.list_downloads()
    }

    pub async fn pause(&self, id: i64) -> Result<()> {
        let token = self.task_tokens.lock().await.get(&id).cloned();
        if let Some(token) = token {
            token.cancel();
        }
        let _ = self.store.reconcile_download_progress(id);
        self.store
            .update_download_state(id, DownloadState::Paused, None)?;
        if let Some(t) = self.store.get_download(id)? {
            self.emit_progress(DownloadProgress {
                id,
                done: t.done,
                total: t.total,
                failed: t.failed,
                state: DownloadState::Paused,
                error: None,
                bytes_per_sec: 0,
            });
        }
        self.emit_event(CoreEvent::DownloadsChanged);
        self.scheduler_notify.notify_one();
        Ok(())
    }

    pub async fn resume(&self, id: i64) -> Result<()> {
        self.store.reset_failed_pages_for_download(id)?;
        let _ = self.store.reconcile_download_progress(id);
        self.store
            .update_download_state(id, DownloadState::Queued, None)?;
        if let Some(t) = self.store.get_download(id)? {
            self.emit_progress(DownloadProgress {
                id,
                done: t.done,
                total: t.total,
                failed: t.failed,
                state: DownloadState::Queued,
                error: None,
                bytes_per_sec: 0,
            });
        }
        self.emit_event(CoreEvent::DownloadsChanged);
        self.scheduler_notify.notify_one();
        Ok(())
    }

    pub async fn remove(&self, id: i64, delete_files: bool) -> Result<()> {
        let token = self.task_tokens.lock().await.get(&id).cloned();
        if let Some(token) = token {
            token.cancel();
        }
        if let Some(task) = self.store.remove_download(id)?
            && delete_files
            && !task.dir.is_empty()
        {
            let configured_dir = PathBuf::from(&self.settings.get().download_dir);
            let task_path = PathBuf::from(&task.dir);

            let is_inside = if let (Ok(c_conf), Ok(c_task)) = (
                fs::canonicalize(&configured_dir),
                fs::canonicalize(&task_path),
            ) {
                c_task.starts_with(c_conf)
            } else {
                task_path.starts_with(&configured_dir)
            };

            if is_inside && task_path.is_dir() {
                let _ = fs::remove_dir_all(task_path);
            }
        }
        self.emit_event(CoreEvent::DownloadsChanged);
        self.scheduler_notify.notify_one();
        Ok(())
    }

    pub async fn pause_all(&self) -> Result<()> {
        let tokens: Vec<_> = self.task_tokens.lock().await.values().cloned().collect();
        for t in tokens {
            t.cancel();
        }
        let _ = self.store.reconcile_all_downloads_progress();
        if let Ok(tasks) = self.store.list_downloads() {
            for t in tasks {
                if t.state == DownloadState::Running || t.state == DownloadState::Queued {
                    let _ = self
                        .store
                        .update_download_state(t.id, DownloadState::Paused, None);
                    self.emit_progress(DownloadProgress {
                        id: t.id,
                        done: t.done,
                        total: t.total,
                        failed: t.failed,
                        state: DownloadState::Paused,
                        error: None,
                        bytes_per_sec: 0,
                    });
                }
            }
        }
        self.emit_event(CoreEvent::DownloadsChanged);
        Ok(())
    }

    pub async fn resume_all(&self) -> Result<()> {
        if let Ok(tasks) = self.store.list_downloads() {
            for t in tasks {
                if t.state == DownloadState::Paused || t.state == DownloadState::Failed {
                    let _ = self.store.reset_failed_pages_for_download(t.id);
                    let _ = self.store.reconcile_download_progress(t.id);
                    let _ = self
                        .store
                        .update_download_state(t.id, DownloadState::Queued, None);
                    if let Ok(Some(refreshed)) = self.store.get_download(t.id) {
                        self.emit_progress(DownloadProgress {
                            id: refreshed.id,
                            done: refreshed.done,
                            total: refreshed.total,
                            failed: refreshed.failed,
                            state: DownloadState::Queued,
                            error: None,
                            bytes_per_sec: 0,
                        });
                    }
                }
            }
        }
        self.emit_event(CoreEvent::DownloadsChanged);
        self.scheduler_notify.notify_one();
        Ok(())
    }

    fn emit_progress(&self, progress: DownloadProgress) {
        self.emit_event(CoreEvent::DownloadProgress(progress));
    }

    fn emit_event(&self, event: CoreEvent) {
        self.sink.emit(event.clone());
        let _ = self.broadcast_tx.send(event);
    }
}

fn content_type_to_ext(ct: &str) -> Option<String> {
    match ct.to_lowercase().as_str() {
        "image/jpeg" | "image/jpg" => Some("jpg".to_string()),
        "image/png" => Some("png".to_string()),
        "image/gif" => Some("gif".to_string()),
        "image/webp" => Some("webp".to_string()),
        "image/avif" => Some("avif".to_string()),
        "image/bmp" => Some("bmp".to_string()),
        _ => None,
    }
}
