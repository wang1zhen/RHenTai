use regex::Regex;
use scraper::{Html, Selector};
use std::sync::LazyLock;

static PROFILENAME_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"id="profilename"[^>]*>([^<]+)<"#).expect("valid regex"));

/// Check whether an igneous cookie value is valid per EH_PROTOCOL §2.1
pub fn is_valid_igneous(val: &str) -> bool {
    let trimmed = val.trim();
    !trimmed.is_empty() && trimmed != "mystery" && trimmed != "deleted" && trimmed != "null"
}

/// Extract username from forums profile HTML (#profilename)
pub fn parse_username_from_profile(html: &str) -> Option<String> {
    if let Some(caps) = PROFILENAME_REGEX.captures(html) {
        let name = caps[1].trim();
        if !name.is_empty() {
            return Some(name.to_string());
        }
    }

    let document = Html::parse_document(html);
    let sel = Selector::parse("#profilename").ok()?;
    document
        .select(&sel)
        .next()
        .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
        .filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_igneous() {
        assert!(!is_valid_igneous(""));
        assert!(!is_valid_igneous("mystery"));
        assert!(!is_valid_igneous("deleted"));
        assert!(!is_valid_igneous("null"));
        assert!(is_valid_igneous("3a9f0e12345"));
    }

    #[test]
    fn test_parse_username() {
        let html = r#"<div id="profilename">TestUser123</div>"#;
        assert_eq!(
            parse_username_from_profile(html),
            Some("TestUser123".to_string())
        );
    }
}
