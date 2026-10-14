use regex::Regex;
use scraper::{Html, Selector};
use std::sync::LazyLock;

use crate::error::{EhError, Result};
use crate::model::PageImage;

static NL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"nl\('([^']+)'\)").expect("valid regex"));

static WIDTH_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"width:\s*(\d+)px").expect("valid regex"));

static HEIGHT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"height:\s*(\d+)px").expect("valid regex"));

/// Parse /s/ image page HTML
pub fn parse_image_page(html: &str, index: u32) -> Result<PageImage> {
    let document = Html::parse_document(html);

    // #img
    let img_sel =
        Selector::parse("#img").map_err(|e| EhError::parse(format!("Selector error: {e}")))?;
    let img = document
        .select(&img_sel)
        .next()
        .ok_or_else(|| EhError::parse("Missing #img element on image page"))?;

    let url = img
        .value()
        .attr("src")
        .ok_or_else(|| EhError::parse("Missing src on #img element"))?
        .to_string();

    // Size from style or attributes
    let style = img.value().attr("style").unwrap_or_default();
    let width = WIDTH_REGEX
        .captures(style)
        .and_then(|c| c[1].parse().ok())
        .or_else(|| img.value().attr("width").and_then(|w| w.parse().ok()));
    let height = HEIGHT_REGEX
        .captures(style)
        .and_then(|c| c[1].parse().ok())
        .or_else(|| img.value().attr("height").and_then(|h| h.parse().ok()));

    // Reload key: nl('...')
    let mut reload_key = None;
    if let Some(onerror) = img.value().attr("onerror")
        && let Some(caps) = NL_REGEX.captures(onerror)
    {
        reload_key = Some(caps[1].to_string());
    }
    if reload_key.is_none() {
        let loadfail_sel = Selector::parse("#loadfail").ok();
        if let Some(ref sel) = loadfail_sel
            && let Some(lf) = document.select(sel).next()
            && let Some(onclick) = lf.value().attr("onclick")
            && let Some(caps) = NL_REGEX.captures(onclick)
        {
            reload_key = Some(caps[1].to_string());
        }
    }
    if reload_key.is_none()
        && let Some(caps) = NL_REGEX.captures(html)
    {
        reload_key = Some(caps[1].to_string());
    }

    // Original URL: #i6 a[href*="fullimg"]
    let original_sel = Selector::parse("#i6 a[href*='fullimg'], a[href*='fullimg.php']").ok();
    let original_url = original_sel.and_then(|sel| {
        document
            .select(&sel)
            .next()
            .and_then(|a| a.value().attr("href"))
            .map(|s| s.to_string())
    });

    // File name: #i4 > div text before " :: "
    let i4_sel = Selector::parse("#i4 > div, #i4").ok();
    let file_name = i4_sel.and_then(|sel| {
        document.select(&sel).next().and_then(|el| {
            let text = el.text().collect::<Vec<_>>().join(" ");
            let trimmed = text.trim();
            if let Some(pos) = trimmed.find(" :: ") {
                Some(trimmed[..pos].trim().to_string())
            } else if !trimmed.is_empty() {
                Some(trimmed.to_string())
            } else {
                None
            }
        })
    });

    Ok(PageImage {
        index,
        url,
        width,
        height,
        reload_key,
        original_url,
        file_name,
    })
}
