use eh_client::parse::parse_image_page;

const IMAGE_PAGE: &str = include_str!("fixtures/image_page.html");

#[test]
fn test_image_page_parser() {
    let img = parse_image_page(IMAGE_PAGE, 0).expect("should parse image page");
    assert_eq!(img.index, 0);
    assert!(img.url.contains("hath.network"));
    assert!(img.url.contains("0001.webp"));
    assert_eq!(img.width, Some(1280));
    assert_eq!(img.height, Some(1785));
    assert_eq!(img.reload_key, Some("52952-497769".to_string()));
    assert_eq!(img.file_name, Some("0001.jpg".to_string()));
}
