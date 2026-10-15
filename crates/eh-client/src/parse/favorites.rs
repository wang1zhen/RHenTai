use regex::Regex;
use scraper::{Html, Selector};
use std::sync::LazyLock;

use crate::error::{EhError, Result};
use crate::model::FavoriteCategory;

static BORDER_COLOR_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"border-color:\s*(#[0-9a-fA-F]{3,6}|rgb\([^)]+\))").expect("valid regex")
});

static FAV_BG_POS_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"background-position:\s*0px\s+(-?\d+)px").expect("valid regex"));

/// Map a border color from style to favorite slot 0..=9
pub fn parse_favorite_slot_from_border(style: &str) -> Option<u8> {
    let caps = BORDER_COLOR_REGEX.captures(style)?;
    let color = caps[1].to_lowercase();
    match color.as_str() {
        "#000" | "#000000" | "rgb(0,0,0)" | "rgb(0, 0, 0)" => Some(0),
        "#f00" | "#ff0000" | "rgb(255,0,0)" | "rgb(255, 0, 0)" => Some(1),
        "#fa0" | "#ffaa00" | "rgb(255,170,0)" | "rgb(255, 170, 0)" => Some(2),
        "#dd0" | "#dddd00" | "rgb(221,221,0)" | "rgb(221, 221, 0)" => Some(3),
        "#080" | "#008800" | "rgb(0,136,0)" | "rgb(0, 136, 0)" => Some(4),
        "#9f4" | "#99ff44" | "rgb(153,255,68)" | "rgb(153, 255, 68)" => Some(5),
        "#4bf" | "#44bbff" | "rgb(68,187,255)" | "rgb(68, 187, 255)" => Some(6),
        "#00f" | "#0000ff" | "rgb(0,0,255)" | "rgb(0, 0, 255)" => Some(7),
        "#508" | "#550088" | "rgb(85,0,136)" | "rgb(85, 0, 136)" => Some(8),
        "#e8e" | "#ee88ee" | "rgb(238,136,238)" | "rgb(238, 136, 238)" => Some(9),
        _ => None,
    }
}

/// Map detail page #fav .i background-position:0px -Ypx to slot (Y - 2) / 19
pub fn parse_favorite_slot_from_position(style: &str) -> Option<u8> {
    let caps = FAV_BG_POS_REGEX.captures(style)?;
    let y: i32 = caps[1].parse().ok()?;
    let y_pos = (-y).max(0);
    if y_pos >= 2 {
        let slot = ((y_pos - 2) / 19) as u8;
        if slot <= 9 {
            return Some(slot);
        }
    }
    None
}

/// Parse the 10 favorite categories from favorites.php HTML
pub fn parse_favorite_categories(html: &str) -> Result<Vec<FavoriteCategory>> {
    let document = Html::parse_document(html);
    let fp_sel = Selector::parse(".nosel .fp, .ido .fp")
        .map_err(|e| EhError::parse(format!("Selector error: {e}")))?;
    let div_sel =
        Selector::parse("div").map_err(|e| EhError::parse(format!("Selector error: {e}")))?;

    let mut categories = Vec::new();

    for el in document.select(&fp_sel) {
        // Skip "All Favorites" which has class "fps"
        let class_attr = el.value().attr("class").unwrap_or("");
        if class_attr.split_whitespace().any(|c| c == "fps") {
            continue;
        }

        let child_divs: Vec<_> = el.select(&div_sel).collect();
        if child_divs.is_empty() {
            continue;
        }

        let count_str = child_divs[0].text().collect::<Vec<_>>().join(" ");
        let count = count_str.trim().parse::<u32>().unwrap_or(0);

        let name = if child_divs.len() > 1 {
            child_divs[child_divs.len() - 1]
                .text()
                .collect::<Vec<_>>()
                .join(" ")
                .trim()
                .to_string()
        } else {
            format!("Favorites {}", categories.len())
        };

        let slot = categories.len() as u8;
        if slot < 10 {
            categories.push(FavoriteCategory { slot, name, count });
        }
    }

    Ok(categories)
}

/// Parse favorite note from gallery popup HTML (gallerypopups.php?act=addfav).
/// Returns HTML-decoded textarea[name=favnote] text, trimmed; empty string becomes None.
pub fn parse_favorite_note(html: &str) -> Result<Option<String>> {
    let document = Html::parse_document(html);
    let textarea_sel = Selector::parse("textarea[name=favnote]")
        .map_err(|e| EhError::parse(format!("Selector error: {e}")))?;

    let el = document
        .select(&textarea_sel)
        .next()
        .ok_or_else(|| EhError::parse("Missing textarea[name=favnote] in favorite popup"))?;

    let text: String = el.text().collect();
    let trimmed = text.trim();
    if trimmed.is_empty() {
        Ok(None)
    } else {
        Ok(Some(trimmed.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_border_palette() {
        assert_eq!(
            parse_favorite_slot_from_border("border-color:#000"),
            Some(0)
        );
        assert_eq!(
            parse_favorite_slot_from_border("border-color:#f00"),
            Some(1)
        );
        assert_eq!(
            parse_favorite_slot_from_border("border-color:#fa0"),
            Some(2)
        );
        assert_eq!(
            parse_favorite_slot_from_border("border-color:#dd0"),
            Some(3)
        );
        assert_eq!(
            parse_favorite_slot_from_border("border-color:#080"),
            Some(4)
        );
        assert_eq!(
            parse_favorite_slot_from_border("border-color:#9f4"),
            Some(5)
        );
        assert_eq!(
            parse_favorite_slot_from_border("border-color:#4bf"),
            Some(6)
        );
        assert_eq!(
            parse_favorite_slot_from_border("border-color:#00f"),
            Some(7)
        );
        assert_eq!(
            parse_favorite_slot_from_border("border-color:#508"),
            Some(8)
        );
        assert_eq!(
            parse_favorite_slot_from_border("border-color:#e8e"),
            Some(9)
        );
    }

    #[test]
    fn test_fav_position() {
        assert_eq!(
            parse_favorite_slot_from_position("background-position:0px -2px"),
            Some(0)
        );
        assert_eq!(
            parse_favorite_slot_from_position("background-position:0px -21px"),
            Some(1)
        );
        assert_eq!(
            parse_favorite_slot_from_position("background-position:0px -40px"),
            Some(2)
        );
        assert_eq!(
            parse_favorite_slot_from_position("background-position:0px -173px"),
            Some(9)
        );
    }

    #[test]
    fn test_parse_favorite_note_basic() {
        let html = r#"<form id="galpop"><textarea name="favnote">hello world</textarea></form>"#;
        assert_eq!(
            parse_favorite_note(html).unwrap(),
            Some("hello world".to_string())
        );
    }

    #[test]
    fn test_parse_favorite_note_html_entities() {
        let html =
            r#"<form id="galpop"><textarea name="favnote">rhentai-qa &amp; more</textarea></form>"#;
        assert_eq!(
            parse_favorite_note(html).unwrap(),
            Some("rhentai-qa & more".to_string())
        );
    }

    #[test]
    fn test_parse_favorite_note_empty_and_whitespace() {
        let html_empty = r#"<form id="galpop"><textarea name="favnote"></textarea></form>"#;
        assert_eq!(parse_favorite_note(html_empty).unwrap(), None);

        let html_whitespace =
            "<form id=\"galpop\"><textarea name=\"favnote\">   \n\t  </textarea></form>";
        assert_eq!(parse_favorite_note(html_whitespace).unwrap(), None);
    }

    #[test]
    fn test_parse_favorite_note_missing_textarea() {
        let html = r#"<form id="galpop"><p>No textarea</p></form>"#;
        assert!(parse_favorite_note(html).is_err());
    }
}
