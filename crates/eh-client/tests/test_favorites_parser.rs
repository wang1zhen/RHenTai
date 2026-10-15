use eh_client::parse::{parse_favorite_categories, parse_favorite_note, parse_gallery_list};

const FAVORITES: &str = include_str!("fixtures/synthetic_favorites.html");
const FAV_POPUP_NOTE: &str = include_str!("fixtures/synthetic_fav_popup_note.html");
const FAV_POPUP_EMPTY: &str = include_str!("fixtures/synthetic_fav_popup_empty.html");

#[test]
fn test_favorite_categories_parser() {
    let cats = parse_favorite_categories(FAVORITES).expect("should parse favorite categories");
    assert_eq!(
        cats.len(),
        10,
        "should have exactly 10 favorite categories (0..=9)"
    );

    assert_eq!(cats[0].slot, 0);
    assert_eq!(cats[0].count, 12);
    assert_eq!(cats[0].name, "Favorites 0");

    assert_eq!(cats[1].slot, 1);
    assert_eq!(cats[1].count, 5);
    assert_eq!(cats[1].name, "Manga Best");

    assert_eq!(cats[2].slot, 2);
    assert_eq!(cats[2].count, 20);
    assert_eq!(cats[2].name, "Artists");

    assert_eq!(cats[9].slot, 9);
    assert_eq!(cats[9].count, 14);
    assert_eq!(cats[9].name, "To Read");
}

#[test]
fn test_favorite_list_items_parser() {
    let list = parse_gallery_list(FAVORITES).expect("should parse favorite list page");
    assert_eq!(list.items.len(), 2);

    assert_eq!(list.items[0].gid, 123456);
    assert_eq!(list.items[0].favorite_slot, Some(2)); // #fa0 -> slot 2

    assert_eq!(list.items[1].gid, 654321);
    assert_eq!(list.items[1].favorite_slot, Some(9)); // #e8e -> slot 9

    assert_eq!(list.prev, Some("prev:654321".to_string()));
    assert_eq!(list.next, Some("next:123456".to_string()));
}

#[test]
fn test_parse_favorite_note_fixtures() {
    let note = parse_favorite_note(FAV_POPUP_NOTE).expect("should parse favorite note");
    assert_eq!(note, Some("rhentai-qa & more".to_string()));

    let empty = parse_favorite_note(FAV_POPUP_EMPTY).expect("should parse empty favorite note");
    assert_eq!(empty, None);
}
