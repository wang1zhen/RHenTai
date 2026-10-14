use eh_client::client::{ClientConfig, EhClient};
use eh_client::model::{GalleryKey, ListRequest, ListSource, SearchFilter};

fn is_valid_image_bytes(bytes: &[u8]) -> bool {
    if bytes.len() < 12 {
        return false;
    }
    // JPEG
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return true;
    }
    // PNG
    if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        return true;
    }
    // WebP: RIFF....WEBP
    if bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
        return true;
    }
    // GIF
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return true;
    }
    false
}

#[tokio::test]
async fn test_live_pipeline() {
    if std::env::var("RHENTAI_LIVE").as_deref() != Ok("1") {
        eprintln!("Skipping live test because RHENTAI_LIVE is not 1");
        return;
    }

    println!("Running live integration test...");

    let client = EhClient::new(ClientConfig::default()).expect("failed to create EhClient");

    // 1. List front page
    let list_req = ListRequest {
        source: ListSource::Front,
        filter: SearchFilter::default(),
        cursor: None,
    };
    let list = client
        .list(&list_req)
        .await
        .expect("failed to list front page");
    assert!(!list.items.is_empty(), "front page list should have items");
    let first = &list.items[0];
    println!(
        "Front page first item: gid={}, title={}",
        first.gid, first.title
    );

    let key = GalleryKey {
        gid: first.gid,
        token: first.token.clone(),
    };

    // 2. Fetch detail
    let detail = client
        .gallery_detail(&key)
        .await
        .expect("failed to fetch gallery detail");
    println!(
        "Detail fetched: title={}, thumb_pages={}, thumbs={}",
        detail.summary.title,
        detail.thumbnail_page_count,
        detail.thumbnails.pages.len()
    );
    assert!(
        !detail.thumbnails.pages.is_empty(),
        "detail page 0 should have thumbnails"
    );

    // 3. Call page_image for index 0
    let p0 = &detail.thumbnails.pages[0];
    let page_img = client
        .page_image(&key, p0, None)
        .await
        .expect("failed to fetch page image info");
    println!("Page image 0 resolved: url={}", page_img.url);
    assert!(!page_img.url.is_empty(), "image url should not be empty");

    // 4. Fetch image bytes and check magic bytes
    let gallery_url = format!(
        "{}/g/{}/{}/",
        client.site().gallery_root().trim_end_matches('/'),
        key.gid,
        key.token
    );
    let fetched = client
        .fetch_image(&page_img.url, Some(&gallery_url))
        .await
        .expect("failed to fetch image bytes");
    println!(
        "Image fetched: size={} bytes, content_type={:?}",
        fetched.bytes.len(),
        fetched.content_type
    );
    assert!(
        fetched.bytes.len() > 1000,
        "image should be reasonably sized"
    );
    assert!(
        is_valid_image_bytes(&fetched.bytes),
        "fetched bytes must match JPEG/PNG/WebP/GIF magic bytes"
    );
}
