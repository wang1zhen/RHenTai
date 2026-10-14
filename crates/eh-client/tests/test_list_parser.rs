use eh_client::model::Category;
use eh_client::parse::parse_gallery_list;

const FRONT_COMPACT: &str = include_str!("fixtures/front_compact.html");
const POPULAR: &str = include_str!("fixtures/popular.html");
const TOPLIST_DAY: &str = include_str!("fixtures/toplist_day.html");
const EXTENDED: &str = include_str!("fixtures/synthetic_list_extended.html");
const MINIMAL: &str = include_str!("fixtures/synthetic_list_minimal.html");
const THUMBNAIL: &str = include_str!("fixtures/synthetic_list_thumbnail.html");
const EMPTY: &str = include_str!("fixtures/synthetic_list_empty.html");

#[test]
fn test_front_compact_parser() {
    let list = parse_gallery_list(FRONT_COMPACT).expect("should parse front page");
    assert!(!list.items.is_empty(), "items should not be empty");
    assert!(
        list.items.len() >= 20,
        "front page should have at least 20 items"
    );

    let first = &list.items[0];
    assert!(first.gid > 0);
    assert_eq!(first.token.len(), 10);
    assert!(!first.title.is_empty());
    assert_ne!(first.category, Category::Unknown);
    assert!(first.posted > 0);
    assert!(!first.thumb.is_empty());

    // Cursors
    assert!(list.next.is_some(), "should have next cursor on front page");
    assert!(list.next.as_ref().unwrap().starts_with("next:"));
    assert!(list.prev.is_none(), "first page should have no prev cursor");
}

#[test]
fn test_popular_parser() {
    let list = parse_gallery_list(POPULAR).expect("should parse popular page");
    assert!(!list.items.is_empty(), "popular page should have items");
    // Popular page has no cursors
    assert!(list.next.is_none());
    assert!(list.prev.is_none());
}

#[test]
fn test_toplist_day_parser() {
    let list = parse_gallery_list(TOPLIST_DAY).expect("should parse toplist page");
    assert!(!list.items.is_empty(), "toplist should have items");
    assert_eq!(list.next, Some("page:1".to_string()));
    assert_eq!(list.prev, None);
}

#[test]
fn test_extended_list_parser() {
    let list = parse_gallery_list(EXTENDED).expect("should parse extended list");
    assert_eq!(list.items.len(), 2);
    assert_eq!(list.items[0].gid, 200001);
    assert_eq!(list.items[0].token, "a1b2c3d4e5");
    assert_eq!(list.items[0].category, Category::Doujinshi);
    assert_eq!(list.items[0].title, "Extended Mode Gallery Title 1");
    assert!((list.items[0].rating - 4.0).abs() < 0.01);
    assert_eq!(list.next, Some("next:200002".to_string()));
}

#[test]
fn test_minimal_list_parser() {
    let list = parse_gallery_list(MINIMAL).expect("should parse minimal list");
    assert_eq!(list.items.len(), 2);
    assert_eq!(list.items[0].gid, 300001);
    assert_eq!(list.items[0].token, "12345abcde");
    assert_eq!(list.items[0].category, Category::ArtistCg);
    assert_eq!(list.items[0].title, "Minimal Mode Gallery Title 1");
    assert_eq!(list.prev, Some("prev:300001".to_string()));
}

#[test]
fn test_thumbnail_list_parser() {
    let list = parse_gallery_list(THUMBNAIL).expect("should parse thumbnail grid");
    assert_eq!(list.items.len(), 2);
    assert_eq!(list.items[0].gid, 400001);
    assert_eq!(list.items[0].token, "a9b8c7d6e5");
    assert_eq!(list.items[0].category, Category::Doujinshi);
    assert_eq!(list.items[0].title, "Thumbnail Grid Gallery Title 1");
    assert!((list.items[0].rating - 3.0).abs() < 0.01);
    assert_eq!(list.next, Some("next:400002".to_string()));
}

#[test]
fn test_empty_list_parser() {
    let list = parse_gallery_list(EMPTY).expect("should parse empty result");
    assert!(list.items.is_empty());
    assert!(list.next.is_none());
    assert!(list.prev.is_none());
}
