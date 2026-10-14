use eh_client::parse::parse_thumbnail_page;

const DETAIL_P1: &str = include_str!("fixtures/gallery_detail_p1.html");

#[test]
fn test_thumbnails_page_parser() {
    let thumb_page = parse_thumbnail_page(DETAIL_P1, 1).expect("should parse thumbnail page 1");
    assert_eq!(thumb_page.page, 1);
    assert!(!thumb_page.pages.is_empty());
    // In our 37 page gallery, page 1 has 17 thumbs (indices 20 to 36)
    assert_eq!(thumb_page.pages.len(), 17);

    let first = &thumb_page.pages[0];
    assert_eq!(first.index, 20);
    assert_eq!(first.file_name, Some("0021.jpg".to_string()));
    assert!(!first.page_token.is_empty());
    assert!(first.thumb.width > 0);
    assert!(first.thumb.url.contains("hath.network"));
}
