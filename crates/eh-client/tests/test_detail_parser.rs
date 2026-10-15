use eh_client::model::{Category, GalleryKey};
use eh_client::parse::parse_gallery_detail;

const DETAIL: &str = include_str!("fixtures/gallery_detail.html");

#[test]
fn test_gallery_detail_parser() {
    let key = GalleryKey {
        gid: 4253617,
        token: "97ec92c515".to_string(),
    };

    let detail = parse_gallery_detail(DETAIL, &key).expect("should parse gallery detail");

    // Title and metadata
    assert_eq!(detail.summary.gid, 4253617);
    assert_eq!(detail.summary.token, "97ec92c515");
    assert_eq!(detail.summary.title, "[北川時] 遠距離恋愛中の二人");
    assert_eq!(detail.summary.category, Category::Manga);
    assert!(detail.summary.thumb.contains("ehgt.org"));
    assert_eq!(detail.summary.uploader, Some("KPs14".to_string()));
    assert_eq!(detail.summary.file_count, 37);

    // Language and file size
    assert_eq!(detail.language, Some("Japanese".to_string()));
    assert_eq!(detail.file_size, Some("17.57 MiB".to_string()));
    assert_eq!(detail.rating_count, Some(3));
    assert_eq!(detail.thumbnail_page_count, 2);
    assert_eq!(detail.thumbs_per_page, 20);

    // Tags
    assert!(!detail.tag_groups.is_empty());
    assert_eq!(detail.tag_groups[0].namespace, "language");
    assert_eq!(detail.tag_groups[0].tags, vec!["japanese"]);

    // Thumbnails on page 0
    assert_eq!(detail.thumbnails.pages.len(), 20);
    let p0 = &detail.thumbnails.pages[0];
    assert_eq!(p0.index, 0);
    assert_eq!(p0.page_token, "3ceb43d06b");
    assert_eq!(p0.file_name, Some("0001.jpg".to_string()));
    assert_eq!(p0.thumb.width, 200);
    assert_eq!(p0.thumb.offset_x, 0);
    assert_eq!(p0.thumb.offset_y, 0);

    let p1 = &detail.thumbnails.pages[1];
    assert_eq!(p1.index, 1);
    assert_eq!(p1.page_token, "085d1326b5");
    assert_eq!(p1.file_name, Some("0002.jpg".to_string()));
    assert_eq!(p1.thumb.offset_x, -200);

    // Unfavorited gallery detail has both None
    assert_eq!(detail.summary.favorite_slot, None);
    assert_eq!(detail.summary.favorite_note, None);
}

#[test]
fn test_gallery_detail_favorite_note_is_none_with_fav_present() {
    let key = GalleryKey {
        gid: 123456,
        token: "abcdef1234".to_string(),
    };
    let html = r#"
    <div id="gd2"><h1 id="gn">Test Gallery</h1></div>
    <div id="gdf">
      <div id="fav"><div class="i" title="Favorites 9" style="background-position:0px -173px"></div></div>
      <a id="favoritelink">Favorites 9</a>
    </div>
    "#;
    let detail = parse_gallery_detail(html, &key).expect("should parse gallery detail");
    assert_eq!(
        detail.summary.favorite_slot,
        Some(9),
        "favorite_slot should be parsed from background-position"
    );
    assert_eq!(
        detail.summary.favorite_note, None,
        "favorite_note must be None even when #fav .i title and category name are present"
    );
}
