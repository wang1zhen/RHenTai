use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use eh_client::{
    Category, FetchedImage, GalleryDetail, GalleryKey, GallerySummary, PageImage, PageRef, Site,
    ThumbSprite, ThumbnailPage,
};
use futures::future::BoxFuture;
use rhentai_core::{ImageService, PageSource, Result, Store};
use tokio::sync::Mutex;

#[derive(Clone)]
struct FakePageSource {
    total_pages: u32,
    thumbs_per_page: u32,
    fetch_counts: Arc<Mutex<HashMap<u32, usize>>>,
}

impl FakePageSource {
    fn new(total_pages: u32, thumbs_per_page: u32) -> Self {
        Self {
            total_pages,
            thumbs_per_page,
            fetch_counts: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl PageSource for FakePageSource {
    fn site(&self) -> Site {
        Site::Eh
    }

    fn gallery_detail<'a>(&'a self, key: &'a GalleryKey) -> BoxFuture<'a, Result<GalleryDetail>> {
        Box::pin(async move {
            let first_page_count = self.thumbs_per_page.min(self.total_pages);
            let mut pages = Vec::with_capacity(first_page_count as usize);
            for i in 0..first_page_count {
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

            let th_page_count = self.total_pages.div_ceil(self.thumbs_per_page);

            Ok(GalleryDetail {
                summary: GallerySummary {
                    gid: key.gid,
                    token: key.token.clone(),
                    title: format!("Gallery {}", key.gid),
                    title_jpn: None,
                    category: Category::Manga,
                    thumb: "https://example.com/thumb.jpg".to_string(),
                    uploader: Some("tester".to_string()),
                    posted: 1700000000,
                    file_count: self.total_pages,
                    rating: 4.5,
                    tags: Vec::new(),
                    expunged: false,
                    favorite_slot: None,
                    favorite_note: None,
                },
                language: Some("English".to_string()),
                file_size: Some("10 MB".to_string()),
                favorited_count: Some(10),
                rating_count: Some(5),
                parent: None,
                tag_groups: Vec::new(),
                thumbnails: ThumbnailPage { page: 0, pages },
                thumbnail_page_count: th_page_count,
                thumbs_per_page: self.thumbs_per_page,
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
            let start = page * self.thumbs_per_page;
            let end = (start + self.thumbs_per_page).min(self.total_pages);
            let mut pages = Vec::new();
            if start < self.total_pages {
                for i in start..end {
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
            }
            Ok(ThumbnailPage { page, pages })
        })
    }

    fn page_image<'a>(
        &'a self,
        _key: &'a GalleryKey,
        page: &'a PageRef,
        _reload_key: Option<&'a str>,
    ) -> BoxFuture<'a, Result<PageImage>> {
        Box::pin(async move {
            Ok(PageImage {
                index: page.index,
                url: format!("https://example.com/image/{}.jpg", page.index),
                width: Some(1000),
                height: Some(1500),
                reload_key: Some(format!("reload_{}", page.index)),
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

            let bytes = Bytes::from_static(
                b"\xFF\xD8\xFF\xE0\x00\x10JFIF\x00\x01\x01\x01\x00\x48\x00\x48\x00\x00\xFF\xDB\x00\x43\x00fake_jpeg_content",
            );
            Ok(FetchedImage {
                bytes,
                content_type: Some("image/jpeg".to_string()),
            })
        })
    }
}

struct TestDir(PathBuf);

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn setup_test_service(source: Arc<FakePageSource>, test_name: &str) -> (TestDir, ImageService) {
    let temp_dir = std::env::temp_dir().join(format!(
        "rhentai_img_test_{test_name}_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();
    let store = Store::in_memory().unwrap();
    let service = ImageService::new(&temp_dir, source, store);
    (TestDir(temp_dir), service)
}

#[tokio::test]
async fn test_page_fresh_gallery_returns_within_timeout() {
    let fake = Arc::new(FakePageSource::new(40, 20));
    let (_tmp, service) = setup_test_service(fake.clone(), "fresh");

    let res = tokio::time::timeout(Duration::from_secs(5), service.page(12345, "abcdef1234", 0))
        .await
        .expect("test timed out: deadlock in ImageService::page for fresh gallery")
        .expect("page fetch failed");

    assert!(!res.0.is_empty());
    assert_eq!(res.1, "image/jpeg");
}

#[tokio::test]
async fn test_page_second_thumbnail_page_resolves() {
    let fake = Arc::new(FakePageSource::new(40, 20));
    let (_tmp, service) = setup_test_service(fake.clone(), "second_thumb");

    let res = tokio::time::timeout(
        Duration::from_secs(5),
        service.page(12345, "abcdef1234", 25),
    )
    .await
    .expect("test timed out: deadlock or hang for second thumbnail page")
    .expect("page fetch failed");

    assert!(!res.0.is_empty());
    assert_eq!(res.1, "image/jpeg");
    assert_eq!(*fake.fetch_counts.lock().await.get(&25).unwrap_or(&0), 1);
}

#[tokio::test]
async fn test_page_cached_served_without_extra_fetch() {
    let fake = Arc::new(FakePageSource::new(40, 20));
    let (_tmp, service) = setup_test_service(fake.clone(), "cached");

    let first = tokio::time::timeout(Duration::from_secs(5), service.page(12345, "abcdef1234", 0))
        .await
        .expect("first call timed out")
        .expect("first call failed");

    assert_eq!(*fake.fetch_counts.lock().await.get(&0).unwrap_or(&0), 1);

    let second = tokio::time::timeout(Duration::from_secs(5), service.page(12345, "abcdef1234", 0))
        .await
        .expect("second call timed out")
        .expect("second call failed");

    assert_eq!(first.0, second.0);
    assert_eq!(*fake.fetch_counts.lock().await.get(&0).unwrap_or(&0), 1);
}
