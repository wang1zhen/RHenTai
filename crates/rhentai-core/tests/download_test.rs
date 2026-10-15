use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use bytes::Bytes;
use eh_client::{
    Category, FetchedImage, GalleryDetail, GalleryKey, GallerySummary, PageImage, PageRef, Site,
    ThumbSprite, ThumbnailPage,
};
use futures::future::BoxFuture;
use rhentai_core::{
    CoreEvent, DownloadManager, DownloadProgress, DownloadState, EnqueueRequest, IpcError,
    NoopEventSink, PageSource, Result, SettingsStore, Store,
};
use tokio::sync::{Mutex, Notify, Semaphore, broadcast};

#[allow(clippy::type_complexity)]
#[derive(Clone)]
struct FakePageSource {
    page_count: u32,
    site: Site,
    fetch_counts: Arc<Mutex<HashMap<u32, usize>>>,
    reload_keys_received: Arc<Mutex<Vec<(u32, Option<String>)>>>,
    fail_509_on_index: Arc<Mutex<Option<u32>>>,
    fail_502_on_index: Arc<Mutex<HashMap<u32, usize>>>,
    transient_failures_left: Arc<Mutex<HashMap<u32, usize>>>,
    gate_semaphore: Arc<Semaphore>,
    gate_arrived: Arc<Notify>,
    gate_active: Arc<AtomicBool>,
    gate_min_index: Arc<Mutex<Option<u32>>>,
    tags: Arc<Mutex<Vec<eh_client::Tag>>>,
}

impl FakePageSource {
    fn new(page_count: u32) -> Self {
        Self::with_site(page_count, Site::Eh)
    }

    fn with_site(page_count: u32, site: Site) -> Self {
        Self {
            page_count,
            site,
            fetch_counts: Arc::new(Mutex::new(HashMap::new())),
            reload_keys_received: Arc::new(Mutex::new(Vec::new())),
            fail_509_on_index: Arc::new(Mutex::new(None)),
            fail_502_on_index: Arc::new(Mutex::new(HashMap::new())),
            transient_failures_left: Arc::new(Mutex::new(HashMap::new())),
            gate_semaphore: Arc::new(Semaphore::new(0)),
            gate_arrived: Arc::new(Notify::new()),
            gate_active: Arc::new(AtomicBool::new(false)),
            gate_min_index: Arc::new(Mutex::new(None)),
            tags: Arc::new(Mutex::new(vec![eh_client::Tag {
                namespace: "artist".to_string(),
                name: "test_artist".to_string(),
            }])),
        }
    }
}

impl PageSource for FakePageSource {
    fn site(&self) -> Site {
        self.site
    }

    fn gallery_detail<'a>(&'a self, key: &'a GalleryKey) -> BoxFuture<'a, Result<GalleryDetail>> {
        Box::pin(async move {
            let mut pages = Vec::new();
            for i in 0..self.page_count {
                pages.push(PageRef {
                    index: i,
                    page_token: format!("token_{i}"),
                    thumb: ThumbSprite {
                        url: String::new(),
                        width: 100,
                        height: 100,
                        offset_x: 0,
                        offset_y: 0,
                    },
                    file_name: Some(format!("img_{i}.jpg")),
                });
            }

            let tags = self.tags.lock().await.clone();
            Ok(GalleryDetail {
                summary: GallerySummary {
                    gid: key.gid,
                    token: key.token.clone(),
                    title: format!("Test Gallery {}", key.gid),
                    title_jpn: None,
                    category: Category::Manga,
                    thumb: "https://example.com/thumb.jpg".to_string(),
                    uploader: Some("uploader1".to_string()),
                    posted: 1700000000,
                    file_count: self.page_count,
                    rating: 4.5,
                    tags,
                    expunged: false,
                    favorite_slot: None,
                    favorite_note: None,
                },
                language: Some("English".to_string()),
                file_size: Some("10 MB".to_string()),
                favorited_count: Some(100),
                rating_count: Some(50),
                parent: None,
                tag_groups: Vec::new(),
                thumbnails: ThumbnailPage {
                    page: 0,
                    pages: pages.clone(),
                },
                thumbnail_page_count: 1,
                thumbs_per_page: self.page_count,
                comments: Vec::new(),
            })
        })
    }

    fn thumbnails<'a>(
        &'a self,
        _key: &'a GalleryKey,
        page: u32,
    ) -> BoxFuture<'a, Result<ThumbnailPage>> {
        Box::pin(async move {
            Ok(ThumbnailPage {
                page,
                pages: Vec::new(),
            })
        })
    }

    fn page_image<'a>(
        &'a self,
        _key: &'a GalleryKey,
        page: &'a PageRef,
        reload_key: Option<&'a str>,
    ) -> BoxFuture<'a, Result<PageImage>> {
        Box::pin(async move {
            self.reload_keys_received
                .lock()
                .await
                .push((page.index, reload_key.map(|s| s.to_string())));

            if let Some(fail_idx) = *self.fail_509_on_index.lock().await
                && page.index == fail_idx
            {
                return Err(IpcError::new(
                    eh_client::ErrorKind::ImageLimit,
                    "Image viewing limit exceeded (509)",
                ));
            }

            Ok(PageImage {
                index: page.index,
                url: format!("https://example.com/image/{}.jpg", page.index),
                width: Some(1000),
                height: Some(1500),
                reload_key: Some(format!("reload_key_{}", page.index)),
                original_url: None,
                file_name: Some(format!("img_{}.jpg", page.index)),
            })
        })
    }

    fn fetch_image<'a>(
        &'a self,
        url: &'a str,
        _referer: Option<&'a str>,
    ) -> BoxFuture<'a, Result<FetchedImage>> {
        Box::pin(async move {
            let idx = url
                .split('/')
                .next_back()
                .and_then(|s| s.split('.').next())
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(0);

            let mut counts = self.fetch_counts.lock().await;
            *counts.entry(idx).or_insert(0) += 1;
            drop(counts);

            if self.gate_active.load(Ordering::SeqCst) {
                let should_gate = match *self.gate_min_index.lock().await {
                    Some(min_idx) => idx >= min_idx,
                    None => true,
                };
                if should_gate {
                    self.gate_arrived.notify_waiters();
                    let _permit = self
                        .gate_semaphore
                        .acquire()
                        .await
                        .map_err(|_| IpcError::internal("Gate closed"))?;
                }
            }

            let mut fails_502 = self.fail_502_on_index.lock().await;
            if let Some(count) = fails_502.get_mut(&idx)
                && *count > 0
            {
                *count -= 1;
                return Err(IpcError::new(
                    eh_client::ErrorKind::Http,
                    "HTTP 502 Bad Gateway: dead H@H node",
                ));
            }

            let mut fails = self.transient_failures_left.lock().await;
            if let Some(count) = fails.get_mut(&idx)
                && *count > 0
            {
                *count -= 1;
                return Err(IpcError::new(
                    eh_client::ErrorKind::Http,
                    "HTTP 403 Forbidden: dead H@H node",
                ));
            }

            // Valid JPEG image bytes
            let bytes = vec![0xFF, 0xD8, 0xFF, 0xE0, idx as u8, 0xFF, 0xD9];
            Ok(FetchedImage {
                bytes: Bytes::from(bytes),
                content_type: Some("image/jpeg".to_string()),
            })
        })
    }
}

fn setup_test_env(name: &str) -> (PathBuf, SettingsStore, Store) {
    let temp_dir = std::env::temp_dir().join(format!("rhentai_dl_{name}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    let config_dir = temp_dir.join("config");
    let download_dir = temp_dir.join("downloads");
    let settings = SettingsStore::open(&config_dir, &download_dir).unwrap();
    let store = Store::in_memory().unwrap();
    (temp_dir, settings, store)
}

async fn wait_for_state(
    rx: &mut broadcast::Receiver<CoreEvent>,
    task_id: i64,
    target_state: DownloadState,
) -> DownloadProgress {
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            match rx.recv().await {
                Ok(CoreEvent::DownloadProgress(p)) => {
                    if p.id == task_id && p.state == target_state {
                        return p;
                    }
                }
                Ok(_) => {}
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(e) => panic!("Event receiver error: {e:?}"),
            }
        }
    })
    .await
    .unwrap_or_else(|_| panic!("Timed out waiting for task {task_id} to reach {target_state:?}"))
}

async fn wait_for_all_states(
    rx: &mut broadcast::Receiver<CoreEvent>,
    task_ids: &[i64],
    target_state: DownloadState,
) {
    let mut remaining: std::collections::HashSet<i64> = task_ids.iter().copied().collect();
    tokio::time::timeout(Duration::from_secs(10), async {
        while !remaining.is_empty() {
            match rx.recv().await {
                Ok(CoreEvent::DownloadProgress(p)) => {
                    if remaining.contains(&p.id) && p.state == target_state {
                        remaining.remove(&p.id);
                    }
                }
                Ok(_) => {}
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(e) => panic!("Event receiver error: {e:?}"),
            }
        }
    })
    .await
    .unwrap_or_else(|_| {
        panic!("Timed out waiting for tasks {task_ids:?} to reach {target_state:?}")
    });
}

#[tokio::test]
async fn test_download_completes_with_rhentai_json() {
    let (temp_dir, settings, store) = setup_test_env("complete");
    let fake = FakePageSource::new(3);
    let (tx, _) = broadcast::channel(100);
    let manager = DownloadManager::new(
        store.clone(),
        Arc::new(fake.clone()),
        settings.clone(),
        Arc::new(NoopEventSink),
        tx.clone(),
    );
    let mut rx = tx.subscribe();
    manager.start();

    let tasks = manager
        .enqueue(EnqueueRequest {
            galleries: vec![GalleryKey {
                gid: 111,
                token: "token1".to_string(),
            }],
            original: Some(false),
        })
        .await
        .unwrap();

    let task_id = tasks[0].id;
    wait_for_state(&mut rx, task_id, DownloadState::Completed).await;

    let task = store.get_download(task_id).unwrap().unwrap();
    assert_eq!(task.done, 3);
    assert_eq!(task.total, 3);
    assert_eq!(task.failed, 0);

    let dir = PathBuf::from(&task.dir);
    assert!(dir.join("0001.jpg").exists());
    assert!(dir.join("0002.jpg").exists());
    assert!(dir.join("0003.jpg").exists());

    let rhentai_json_path = dir.join("rhentai.json");
    assert!(rhentai_json_path.exists());
    let json_content = fs::read_to_string(rhentai_json_path).unwrap();
    assert!(json_content.contains("\"gid\": 111"));
    assert!(json_content.contains("\"pageCount\": 3"));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_download_resumes_only_missing_pages() {
    let (temp_dir, settings, store) = setup_test_env("resume");
    let fake = FakePageSource::new(4);

    let dl_dir = PathBuf::from(&settings.get().download_dir).join("222 - Test Gallery 222");
    fs::create_dir_all(&dl_dir).unwrap();

    // Pages 0 and 1 already done on disk
    fs::write(dl_dir.join("0001.jpg"), b"page1").unwrap();
    fs::write(dl_dir.join("0002.jpg"), b"page2").unwrap();

    // Insert task in DB as queued with 2 pages done
    let now = rhentai_core::store::current_time_secs();
    let task_id = store
        .insert_download(&rhentai_core::DownloadTask {
            id: 0,
            gid: 222,
            token: "tok222".to_string(),
            site: Site::Eh,
            title: "Test Gallery 222".to_string(),
            thumb: "".to_string(),
            category: Category::Manga,
            total: 4,
            done: 2,
            failed: 0,
            state: DownloadState::Queued,
            error: None,
            dir: dl_dir.to_string_lossy().to_string(),
            original: false,
            created_at: now,
            updated_at: now,
        })
        .unwrap();

    let pages = vec![
        PageRef {
            index: 0,
            page_token: "t0".to_string(),
            thumb: ThumbSprite {
                url: "".to_string(),
                width: 0,
                height: 0,
                offset_x: 0,
                offset_y: 0,
            },
            file_name: Some("0001.jpg".to_string()),
        },
        PageRef {
            index: 1,
            page_token: "t1".to_string(),
            thumb: ThumbSprite {
                url: "".to_string(),
                width: 0,
                height: 0,
                offset_x: 0,
                offset_y: 0,
            },
            file_name: Some("0002.jpg".to_string()),
        },
        PageRef {
            index: 2,
            page_token: "t2".to_string(),
            thumb: ThumbSprite {
                url: "".to_string(),
                width: 0,
                height: 0,
                offset_x: 0,
                offset_y: 0,
            },
            file_name: None,
        },
        PageRef {
            index: 3,
            page_token: "t3".to_string(),
            thumb: ThumbSprite {
                url: "".to_string(),
                width: 0,
                height: 0,
                offset_x: 0,
                offset_y: 0,
            },
            file_name: None,
        },
    ];
    store
        .insert_download_pages_if_empty(task_id, &pages)
        .unwrap();
    store
        .update_download_page_state(task_id, 0, "done", Some("0001.jpg"), None, Some(5))
        .unwrap();
    store
        .update_download_page_state(task_id, 1, "done", Some("0002.jpg"), None, Some(5))
        .unwrap();

    let (tx, _) = broadcast::channel(100);
    let manager = DownloadManager::new(
        store.clone(),
        Arc::new(fake.clone()),
        settings.clone(),
        Arc::new(NoopEventSink),
        tx.clone(),
    );
    let mut rx = tx.subscribe();
    manager.start();

    wait_for_state(&mut rx, task_id, DownloadState::Completed).await;

    // Assert fetch counts: pages 0 and 1 were NEVER fetched!
    let counts = fake.fetch_counts.lock().await;
    assert_eq!(counts.get(&0), None);
    assert_eq!(counts.get(&1), None);
    assert_eq!(counts.get(&2), Some(&1));
    assert_eq!(counts.get(&3), Some(&1));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_download_509_pauses_everything() {
    let (temp_dir, settings, store) = setup_test_env("limit509");
    let fake = FakePageSource::new(4);
    *fake.fail_509_on_index.lock().await = Some(0);

    let (tx, _) = broadcast::channel(100);
    let manager = DownloadManager::new(
        store.clone(),
        Arc::new(fake.clone()),
        settings.clone(),
        Arc::new(NoopEventSink),
        tx.clone(),
    );
    let mut rx = tx.subscribe();
    manager.start();

    let tasks = manager
        .enqueue(EnqueueRequest {
            galleries: vec![
                GalleryKey {
                    gid: 331,
                    token: "tok331".to_string(),
                },
                GalleryKey {
                    gid: 332,
                    token: "tok332".to_string(),
                },
            ],
            original: Some(false),
        })
        .await
        .unwrap();

    let task_ids: Vec<i64> = tasks.iter().map(|t| t.id).collect();
    wait_for_all_states(&mut rx, &task_ids, DownloadState::Paused).await;

    let t1 = store.get_download(tasks[0].id).unwrap().unwrap();
    let t2 = store.get_download(tasks[1].id).unwrap().unwrap();
    assert_eq!(t1.state, DownloadState::Paused);
    assert_eq!(t2.state, DownloadState::Paused);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_download_retries_transient_failures_with_reload_key() {
    let (temp_dir, settings, store) = setup_test_env("retry");
    let fake = FakePageSource::new(2);
    // Page 0 fails once with HTTP 403, then succeeds
    fake.transient_failures_left.lock().await.insert(0, 1);

    let (tx, _) = broadcast::channel(100);
    let manager = DownloadManager::new(
        store.clone(),
        Arc::new(fake.clone()),
        settings.clone(),
        Arc::new(NoopEventSink),
        tx.clone(),
    );
    let mut rx = tx.subscribe();
    manager.start();

    let tasks = manager
        .enqueue(EnqueueRequest {
            galleries: vec![GalleryKey {
                gid: 444,
                token: "tok444".to_string(),
            }],
            original: Some(false),
        })
        .await
        .unwrap();

    let task_id = tasks[0].id;
    wait_for_state(&mut rx, task_id, DownloadState::Completed).await;

    // Verify page_image was called with the reload key on the second attempt
    let rks = fake.reload_keys_received.lock().await;
    let page0_calls: Vec<_> = rks.iter().filter(|(idx, _)| *idx == 0).collect();
    assert!(page0_calls.len() >= 2);
    assert_eq!(page0_calls[0].1, None); // initial call without reload key
    assert_eq!(page0_calls[1].1.as_deref(), Some("reload_key_0")); // retry with reload key

    let _ = fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_download_pause_and_resume() {
    let (temp_dir, settings, store) = setup_test_env("pauseresume");
    let fake = FakePageSource::new(6);
    fake.gate_active.store(true, Ordering::SeqCst);

    let (tx, _) = broadcast::channel(100);
    let manager = DownloadManager::new(
        store.clone(),
        Arc::new(fake.clone()),
        settings.clone(),
        Arc::new(NoopEventSink),
        tx.clone(),
    );
    let mut rx = tx.subscribe();
    manager.start();

    let tasks = manager
        .enqueue(EnqueueRequest {
            galleries: vec![GalleryKey {
                gid: 555,
                token: "tok555".to_string(),
            }],
            original: Some(false),
        })
        .await
        .unwrap();

    let task_id = tasks[0].id;

    // Await fetch arriving at gate
    tokio::time::timeout(Duration::from_secs(10), fake.gate_arrived.notified())
        .await
        .expect("Fetch should arrive at gate");

    // Pause task while held at gate
    manager.pause(task_id).await.unwrap();
    wait_for_state(&mut rx, task_id, DownloadState::Paused).await;

    let paused_task = store.get_download(task_id).unwrap().unwrap();
    assert_eq!(paused_task.state, DownloadState::Paused);

    // Release gate and resume
    fake.gate_active.store(false, Ordering::SeqCst);
    fake.gate_semaphore.add_permits(100);
    manager.resume(task_id).await.unwrap();

    // Await completion
    wait_for_state(&mut rx, task_id, DownloadState::Completed).await;

    let completed_task = store.get_download(task_id).unwrap().unwrap();
    assert_eq!(completed_task.state, DownloadState::Completed);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_download_override_dir() {
    let (temp_dir, settings, store) = setup_test_env("override_dir");
    let custom_dir = temp_dir.join("custom_downloads");
    fs::create_dir_all(&custom_dir).unwrap();

    let fake = FakePageSource::new(2);
    let (tx, _) = broadcast::channel(100);
    let manager = DownloadManager::new(
        store.clone(),
        Arc::new(fake.clone()),
        settings.clone(),
        Arc::new(NoopEventSink),
        tx.clone(),
    );
    manager.set_run_overrides(None, Some(custom_dir.clone()));
    manager.start();

    let mut rx = tx.subscribe();
    let tasks = manager
        .enqueue(EnqueueRequest {
            galleries: vec![GalleryKey {
                gid: 666,
                token: "tok666".to_string(),
            }],
            original: Some(false),
        })
        .await
        .unwrap();

    let task_id = tasks[0].id;
    wait_for_state(&mut rx, task_id, DownloadState::Completed).await;

    let task = store.get_download(task_id).unwrap().unwrap();
    assert!(
        PathBuf::from(&task.dir).starts_with(&custom_dir),
        "Task dir {} should be under override dir {}",
        task.dir,
        custom_dir.display()
    );
    assert!(PathBuf::from(&task.dir).join("0001.jpg").exists());
    assert!(PathBuf::from(&task.dir).join("0002.jpg").exists());
    assert!(PathBuf::from(&task.dir).join("rhentai.json").exists());

    let _ = fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_download_site_recorded_on_task_and_metadata() {
    let (temp_dir, settings, store) = setup_test_env("site_ex");
    let fake = FakePageSource::with_site(2, Site::Ex);
    let (tx, _) = broadcast::channel(100);
    let manager = DownloadManager::new(
        store.clone(),
        Arc::new(fake.clone()),
        settings.clone(),
        Arc::new(NoopEventSink),
        tx.clone(),
    );
    manager.start();

    let mut rx = tx.subscribe();
    let tasks = manager
        .enqueue(EnqueueRequest {
            galleries: vec![GalleryKey {
                gid: 777,
                token: "tok777".to_string(),
            }],
            original: Some(false),
        })
        .await
        .unwrap();

    let task_id = tasks[0].id;
    assert_eq!(tasks[0].site, Site::Ex);

    wait_for_state(&mut rx, task_id, DownloadState::Completed).await;

    let task = store.get_download(task_id).unwrap().unwrap();
    assert_eq!(task.site, Site::Ex);

    let rhentai_json_path = PathBuf::from(&task.dir).join("rhentai.json");
    let content = fs::read_to_string(rhentai_json_path).unwrap();
    assert!(content.contains("\"site\": \"ex\""));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_download_resume_fetches_detail_tags() {
    let (temp_dir, settings, store) = setup_test_env("resume_tags");
    let fake = FakePageSource::new(2);

    let dl_dir = PathBuf::from(&settings.get().download_dir).join("888 - Test Gallery 888");
    fs::create_dir_all(&dl_dir).unwrap();

    // Insert task with pages already in DB so detail_opt is None when execute_task runs
    let now = rhentai_core::store::current_time_secs();
    let task_id = store
        .insert_download(&rhentai_core::DownloadTask {
            id: 0,
            gid: 888,
            token: "tok888".to_string(),
            site: Site::Eh,
            title: "Test Gallery 888".to_string(),
            thumb: "".to_string(),
            category: Category::Manga,
            total: 2,
            done: 0,
            failed: 0,
            state: DownloadState::Queued,
            error: None,
            dir: dl_dir.to_string_lossy().to_string(),
            original: false,
            created_at: now,
            updated_at: now,
        })
        .unwrap();

    let pages = vec![
        PageRef {
            index: 0,
            page_token: "t0".to_string(),
            thumb: ThumbSprite {
                url: "".to_string(),
                width: 0,
                height: 0,
                offset_x: 0,
                offset_y: 0,
            },
            file_name: Some("0001.jpg".to_string()),
        },
        PageRef {
            index: 1,
            page_token: "t1".to_string(),
            thumb: ThumbSprite {
                url: "".to_string(),
                width: 0,
                height: 0,
                offset_x: 0,
                offset_y: 0,
            },
            file_name: Some("0002.jpg".to_string()),
        },
    ];
    store
        .insert_download_pages_if_empty(task_id, &pages)
        .unwrap();

    let (tx, _) = broadcast::channel(100);
    let manager = DownloadManager::new(
        store.clone(),
        Arc::new(fake.clone()),
        settings.clone(),
        Arc::new(NoopEventSink),
        tx.clone(),
    );

    let mut rx = tx.subscribe();
    manager.start();

    wait_for_state(&mut rx, task_id, DownloadState::Completed).await;

    let rhentai_json_path = dl_dir.join("rhentai.json");
    assert!(rhentai_json_path.exists());
    let content = fs::read_to_string(rhentai_json_path).unwrap();
    // Tags from fake detail must appear in rhentai.json!
    assert!(
        content.contains("\"test_artist\""),
        "Tags should appear in rhentai.json on resume: {content}"
    );

    let _ = fs::remove_dir_all(&temp_dir);
}

async fn wait_for_state_timeout(
    rx: &mut broadcast::Receiver<CoreEvent>,
    task_id: i64,
    target_state: DownloadState,
    timeout: Duration,
) -> std::result::Result<DownloadProgress, ()> {
    tokio::time::timeout(timeout, async {
        loop {
            match rx.recv().await {
                Ok(CoreEvent::DownloadProgress(p)) => {
                    if p.id == task_id && p.state == target_state {
                        return p;
                    }
                }
                Ok(_) => {}
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(e) => panic!("Event receiver error: {e:?}"),
            }
        }
    })
    .await
    .map_err(|_| ())
}

#[tokio::test]
async fn test_download_exhausted_retries_becomes_terminal_failed() {
    let (temp_dir, settings, store) = setup_test_env("exhausted_retries");
    let fake = FakePageSource::new(1);
    // Exhaust all 5 attempts with a non-limit error (HTTP 502)
    fake.fail_502_on_index.lock().await.insert(0, 5);

    let (tx, _) = broadcast::channel(100);
    let manager = DownloadManager::new(
        store.clone(),
        Arc::new(fake.clone()),
        settings.clone(),
        Arc::new(NoopEventSink),
        tx.clone(),
    );
    let mut rx = tx.subscribe();
    manager.start();

    let tasks = manager
        .enqueue(EnqueueRequest {
            galleries: vec![GalleryKey {
                gid: 901,
                token: "tok901".to_string(),
            }],
            original: Some(false),
        })
        .await
        .unwrap();

    let task_id = tasks[0].id;

    // Event-driven wait for Failed state
    let progress = wait_for_state_timeout(
        &mut rx,
        task_id,
        DownloadState::Failed,
        Duration::from_secs(2),
    )
    .await
    .expect("Timed out waiting for Failed state");
    assert_eq!(progress.state, DownloadState::Failed);
    assert_eq!(progress.failed, 1);
    assert!(progress.error.is_some(), "Progress must report error");

    // Assert 5 fetch attempts for that page
    let counts = fake.fetch_counts.lock().await;
    assert_eq!(counts.get(&0), Some(&5), "Must have made 5 fetch attempts");

    // Assert the task row in store is Failed with an error summary
    let task = store.get_download(task_id).unwrap().unwrap();
    assert_eq!(task.state, DownloadState::Failed);
    assert_eq!(task.failed, 1);
    let err_msg = task.error.expect("Task row must have error summary");
    assert!(
        err_msg.contains("1 page(s) failed"),
        "Error message must summarize failed pages: {err_msg}"
    );

    let _ = fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_download_non403_failure_must_reresolve_with_nl() {
    let (temp_dir, settings, store) = setup_test_env("non403_reresolve");
    let fake = FakePageSource::new(1);
    // Page 0 fails once with HTTP 502, then succeeds
    fake.fail_502_on_index.lock().await.insert(0, 1);

    let (tx, _) = broadcast::channel(100);
    let manager = DownloadManager::new(
        store.clone(),
        Arc::new(fake.clone()),
        settings.clone(),
        Arc::new(NoopEventSink),
        tx.clone(),
    );
    let mut rx = tx.subscribe();
    manager.start();

    let tasks = manager
        .enqueue(EnqueueRequest {
            galleries: vec![GalleryKey {
                gid: 902,
                token: "tok902".to_string(),
            }],
            original: Some(false),
        })
        .await
        .unwrap();

    let task_id = tasks[0].id;
    wait_for_state(&mut rx, task_id, DownloadState::Completed).await;

    let calls = fake.reload_keys_received.lock().await.clone();
    assert_eq!(
        calls,
        vec![(0, None), (0, Some("reload_key_0".to_string()))],
        "Retryable non-403 failure must re-resolve with retained reload key"
    );

    let _ = fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_download_paused_reconciles_done_count_with_gated_fake() {
    let (temp_dir, settings, store) = setup_test_env("paused_reconcile");
    let k = 2;
    let fake = FakePageSource::new(4);
    // Gate pages starting from index 2 (k = 2)
    *fake.gate_min_index.lock().await = Some(k);
    fake.gate_active.store(true, Ordering::SeqCst);

    let (tx, _) = broadcast::channel(100);
    let manager = DownloadManager::new(
        store.clone(),
        Arc::new(fake.clone()),
        settings.clone(),
        Arc::new(NoopEventSink),
        tx.clone(),
    );
    let mut rx = tx.subscribe();
    manager.start();

    let tasks = manager
        .enqueue(EnqueueRequest {
            galleries: vec![GalleryKey {
                gid: 903,
                token: "tok903".to_string(),
            }],
            original: Some(false),
        })
        .await
        .unwrap();

    let task_id = tasks[0].id;

    // Await fetch arriving at gate for the gated pages
    tokio::time::timeout(Duration::from_secs(5), fake.gate_arrived.notified())
        .await
        .expect("Fetch should arrive at gate");

    // Pause task while held at gate
    manager.pause(task_id).await.unwrap();

    let paused_progress = wait_for_state(&mut rx, task_id, DownloadState::Paused).await;
    assert_eq!(
        paused_progress.done, k,
        "Paused progress event must report done == k"
    );

    let task = store.get_download(task_id).unwrap().unwrap();
    assert_eq!(task.done, k, "Task row in store must report done == k");

    // Release gate
    fake.gate_active.store(false, Ordering::SeqCst);
    fake.gate_semaphore.add_permits(100);

    let _ = fs::remove_dir_all(&temp_dir);
}
