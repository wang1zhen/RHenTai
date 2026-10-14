use regex::Regex;
use scraper::{Html, Selector};
use std::sync::LazyLock;

use crate::error::{EhError, Result};
use crate::model::{Category, Comment, GalleryDetail, GalleryKey, GallerySummary, Tag, TagGroup};
use crate::parse::date::parse_eh_date;
use crate::parse::favorites::parse_favorite_slot_from_position;
use crate::parse::rating::parse_rating_label;
use crate::parse::thumbnails::parse_thumbnails_from_html;

static URL_BG_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"url\(([^)]+)\)").expect("valid regex"));

static PARENT_URL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"/g/(\d+)/([0-9a-f]{10})/").expect("valid regex"));

static LENGTH_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(\d+)\s*pages?").expect("valid regex"));

static FAVORITED_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(\d+)\s*times?").expect("valid regex"));

/// Parse gallery detail page HTML
pub fn parse_gallery_detail(html: &str, key: &GalleryKey) -> Result<GalleryDetail> {
    let document = Html::parse_document(html);

    // Title #gn
    let gn_sel =
        Selector::parse("#gn").map_err(|e| EhError::parse(format!("Selector error: {e}")))?;
    let title = document
        .select(&gn_sel)
        .next()
        .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
        .unwrap_or_default();

    // Japanese Title #gj
    let gj_sel =
        Selector::parse("#gj").map_err(|e| EhError::parse(format!("Selector error: {e}")))?;
    let title_jpn = document
        .select(&gj_sel)
        .next()
        .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
        .filter(|s| !s.is_empty());

    // Category #gdc
    let gdc_sel = Selector::parse("#gdc .cs, #gdc div")
        .map_err(|e| EhError::parse(format!("Selector error: {e}")))?;
    let category = document
        .select(&gdc_sel)
        .next()
        .map(|el| {
            let text = el.text().collect::<Vec<_>>().join(" ");
            Category::from_site(&text)
        })
        .unwrap_or(Category::Unknown);

    // Uploader #gdn
    let gdn_sel = Selector::parse("#gdn a, #gdn")
        .map_err(|e| EhError::parse(format!("Selector error: {e}")))?;
    let uploader = document
        .select(&gdn_sel)
        .next()
        .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
        .filter(|s| !s.is_empty());

    // Cover #gd1 > div
    let gd1_sel =
        Selector::parse("#gd1").map_err(|e| EhError::parse(format!("Selector error: {e}")))?;
    let div_sel =
        Selector::parse("div").map_err(|e| EhError::parse(format!("Selector error: {e}")))?;
    let img_sel =
        Selector::parse("img").map_err(|e| EhError::parse(format!("Selector error: {e}")))?;
    let thumb = document
        .select(&gd1_sel)
        .next()
        .and_then(|gd1| {
            gd1.select(&div_sel)
                .next()
                .and_then(|el| el.value().attr("style"))
                .and_then(|s| {
                    URL_BG_REGEX
                        .captures(s)
                        .map(|c| c[1].trim_matches('\'').trim_matches('"').to_string())
                })
                .or_else(|| {
                    gd1.select(&img_sel)
                        .next()
                        .and_then(|el| el.value().attr("src"))
                        .map(|s| s.to_string())
                })
        })
        .unwrap_or_default();

    // Key-value pairs in #gdd tr (.gdt1 -> .gdt2)
    let gdd_tr_sel =
        Selector::parse("#gdd tr").map_err(|e| EhError::parse(format!("Selector error: {e}")))?;
    let gdt1_sel =
        Selector::parse(".gdt1").map_err(|e| EhError::parse(format!("Selector error: {e}")))?;
    let gdt2_sel =
        Selector::parse(".gdt2").map_err(|e| EhError::parse(format!("Selector error: {e}")))?;
    let a_sel = Selector::parse("a").map_err(|e| EhError::parse(format!("Selector error: {e}")))?;

    let mut posted = 0;
    let mut parent = None;
    let mut language = None;
    let mut file_size = None;
    let mut file_count = 0;
    let mut favorited_count = None;

    for tr in document.select(&gdd_tr_sel) {
        let label = tr
            .select(&gdt1_sel)
            .next()
            .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
            .unwrap_or_default();
        let value_el = tr.select(&gdt2_sel).next();
        let value = value_el
            .as_ref()
            .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
            .unwrap_or_default();

        if label.starts_with("Posted:") {
            posted = parse_eh_date(&value).unwrap_or(0);
        } else if label.starts_with("Parent:") {
            if let Some(val_el) = value_el
                && let Some(a) = val_el.select(&a_sel).next()
                && let Some(href) = a.value().attr("href")
                && let Some(caps) = PARENT_URL_REGEX.captures(href)
                && let (Ok(p_gid), Some(p_tok)) = (caps[1].parse::<u64>(), caps.get(2))
            {
                parent = Some(GalleryKey {
                    gid: p_gid,
                    token: p_tok.as_str().to_string(),
                });
            }
        } else if label.starts_with("Language:") {
            // Strip trailing " TR" or "&nbsp; TR"
            let lang = value
                .replace("&nbsp;", " ")
                .split_whitespace()
                .filter(|w| *w != "TR")
                .collect::<Vec<_>>()
                .join(" ");
            language = if lang.is_empty() { None } else { Some(lang) };
        } else if label.starts_with("File Size:") {
            file_size = if value.is_empty() { None } else { Some(value) };
        } else if label.starts_with("Length:") {
            if let Some(caps) = LENGTH_REGEX.captures(&value) {
                file_count = caps[1].parse().unwrap_or(0);
            }
        } else if label.starts_with("Favorited:") {
            if value.eq_ignore_ascii_case("never") {
                favorited_count = Some(0);
            } else if value.eq_ignore_ascii_case("once") {
                favorited_count = Some(1);
            } else if let Some(caps) = FAVORITED_REGEX.captures(&value) {
                favorited_count = caps[1].parse().ok();
            }
        }
    }

    // Rating: #rating_label and #rating_count
    let rating_label_sel = Selector::parse("#rating_label")
        .map_err(|e| EhError::parse(format!("Selector error: {e}")))?;
    let rating = document
        .select(&rating_label_sel)
        .next()
        .map(|el| {
            let text = el.text().collect::<Vec<_>>().join(" ");
            parse_rating_label(&text)
        })
        .unwrap_or(0.0);

    let rating_count_sel = Selector::parse("#rating_count")
        .map_err(|e| EhError::parse(format!("Selector error: {e}")))?;
    let rating_count = document
        .select(&rating_count_sel)
        .next()
        .and_then(|el| el.text().collect::<Vec<_>>().join(" ").trim().parse().ok());

    // Tags: #taglist tr
    let tag_tr_sel = Selector::parse("#taglist tr")
        .map_err(|e| EhError::parse(format!("Selector error: {e}")))?;
    let tc_sel =
        Selector::parse("td.tc").map_err(|e| EhError::parse(format!("Selector error: {e}")))?;
    let tag_a_sel = Selector::parse("td:nth-child(2) a")
        .map_err(|e| EhError::parse(format!("Selector error: {e}")))?;

    let mut tag_groups = Vec::new();
    let mut flat_tags = Vec::new();

    for tr in document.select(&tag_tr_sel) {
        let namespace = tr
            .select(&tc_sel)
            .next()
            .map(|el| {
                el.text()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .trim()
                    .trim_end_matches(':')
                    .to_string()
            })
            .unwrap_or_default();

        let tags: Vec<String> = tr
            .select(&tag_a_sel)
            .map(|el| {
                let text = el.text().collect::<Vec<_>>().join(" ");
                let raw = text.trim();
                if let Some(pos) = raw.find(" | ") {
                    raw[..pos].trim().to_string()
                } else {
                    raw.to_string()
                }
            })
            .filter(|s| !s.is_empty())
            .collect();

        for t in &tags {
            flat_tags.push(Tag {
                namespace: namespace.clone(),
                name: t.clone(),
            });
        }

        tag_groups.push(TagGroup { namespace, tags });
    }

    // Favorite: #favoritelink and #fav .i
    let fav_link_sel = Selector::parse("#favoritelink")
        .map_err(|e| EhError::parse(format!("Selector error: {e}")))?;
    let fav_link_text = document
        .select(&fav_link_sel)
        .next()
        .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string());

    let fav_i_sel =
        Selector::parse("#fav .i").map_err(|e| EhError::parse(format!("Selector error: {e}")))?;
    let (favorite_slot, favorite_note) = if let Some(fav_i) = document.select(&fav_i_sel).next() {
        let slot = fav_i
            .value()
            .attr("style")
            .and_then(parse_favorite_slot_from_position);
        let note = fav_i
            .value()
            .attr("title")
            .map(|s| s.trim().to_string())
            .or_else(|| {
                fav_link_text
                    .as_ref()
                    .filter(|s| s.as_str() != "Add to Favorites")
                    .cloned()
            });
        (slot, note)
    } else {
        (None, None)
    };

    // Thumbnails: p=0
    let thumbnails = parse_thumbnails_from_html(html, 0)?;
    let thumbs_per_page = if thumbnails.pages.is_empty() {
        20
    } else {
        thumbnails.pages.len() as u32
    };

    // Page count from .ptt td a
    let ptt_a_sel =
        Selector::parse(".ptt td a").map_err(|e| EhError::parse(format!("Selector error: {e}")))?;
    let mut thumbnail_page_count = 1;
    for a in document.select(&ptt_a_sel) {
        let text = a.text().collect::<Vec<_>>().join(" ").trim().to_string();
        if let Ok(n) = text.parse::<u32>()
            && n > thumbnail_page_count
        {
            thumbnail_page_count = n;
        }
    }

    // Comments: #cdiv .c1
    let c1_sel =
        Selector::parse("#cdiv .c1").map_err(|e| EhError::parse(format!("Selector error: {e}")))?;
    let c3_a_sel =
        Selector::parse(".c3 a").map_err(|e| EhError::parse(format!("Selector error: {e}")))?;
    let c3_sel =
        Selector::parse(".c3").map_err(|e| EhError::parse(format!("Selector error: {e}")))?;
    let c5_span_sel =
        Selector::parse(".c5 span").map_err(|e| EhError::parse(format!("Selector error: {e}")))?;
    let c6_sel =
        Selector::parse(".c6").map_err(|e| EhError::parse(format!("Selector error: {e}")))?;

    let mut comments = Vec::new();
    for c1 in document.select(&c1_sel) {
        let author = c1
            .select(&c3_a_sel)
            .next()
            .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
            .unwrap_or_else(|| {
                c1.select(&c3_sel)
                    .next()
                    .map(|el| {
                        let text = el.text().collect::<Vec<_>>().join(" ");
                        if let Some(pos) = text.find("by:") {
                            text[pos + 3..].trim().to_string()
                        } else {
                            text.trim().to_string()
                        }
                    })
                    .unwrap_or_default()
            });

        let posted_str = c1
            .select(&c3_sel)
            .next()
            .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
            .unwrap_or_default();

        let score = c1
            .select(&c5_span_sel)
            .next()
            .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
            .filter(|s| !s.is_empty());

        let body = c1
            .select(&c6_sel)
            .next()
            .map(|el| {
                // Preserving linebreaks: inner_html with <br> -> \n
                let inner = el.inner_html();
                inner
                    .replace("<br>", "\n")
                    .replace("<br/>", "\n")
                    .replace("<br />", "\n")
            })
            .map(|html_text| {
                // Strip any remaining html tags
                Html::parse_fragment(&html_text)
                    .root_element()
                    .text()
                    .collect::<Vec<_>>()
                    .join("")
                    .trim()
                    .to_string()
            })
            .unwrap_or_default();

        let is_uploader = c1.value().id() == Some("comment_0");

        comments.push(Comment {
            author,
            posted: posted_str,
            score,
            body,
            is_uploader,
        });
    }

    let summary = GallerySummary {
        gid: key.gid,
        token: key.token.clone(),
        title,
        title_jpn,
        category,
        thumb,
        uploader,
        posted,
        file_count,
        rating,
        tags: flat_tags,
        expunged: false,
        favorite_slot,
        favorite_note,
    };

    Ok(GalleryDetail {
        summary,
        language,
        file_size,
        favorited_count,
        rating_count,
        parent,
        tag_groups,
        thumbnails,
        thumbnail_page_count,
        thumbs_per_page,
        comments,
    })
}
