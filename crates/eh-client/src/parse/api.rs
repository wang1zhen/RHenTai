use crate::error::{EhError, Result};
use crate::model::{Category, GallerySummary, Tag};

/// Lenient parser for api.php gdata JSON response
pub fn parse_gdata_response(json_str: &str) -> Result<Vec<GallerySummary>> {
    let value: serde_json::Value =
        serde_json::from_str(json_str).map_err(|e| EhError::parse(format!("Invalid JSON: {e}")))?;

    if let Some(err) = value.get("error").and_then(|v| v.as_str()) {
        return Err(EhError::http(format!("API error: {err}")));
    }

    let gmetadata = match value.get("gmetadata").and_then(|v| v.as_array()) {
        Some(list) => list,
        None => return Ok(Vec::new()),
    };

    let mut summaries = Vec::new();

    for item in gmetadata {
        if item.get("error").is_some() {
            continue;
        }

        let gid = match item.get("gid").and_then(|v| {
            v.as_u64()
                .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        }) {
            Some(g) => g,
            None => continue,
        };

        let token = match item.get("token").and_then(|v| v.as_str()) {
            Some(t) => t.to_string(),
            None => continue,
        };

        let title = item
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        let title_jpn = item
            .get("title_jpn")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        let category = item
            .get("category")
            .and_then(|v| v.as_str())
            .map(Category::from_site)
            .unwrap_or(Category::Unknown);

        let thumb = item
            .get("thumb")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        let uploader = item
            .get("uploader")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        let posted = item
            .get("posted")
            .and_then(|v| {
                v.as_i64()
                    .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            })
            .unwrap_or(0);

        let file_count = item
            .get("filecount")
            .and_then(|v| {
                v.as_u64()
                    .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            })
            .unwrap_or(0) as u32;

        let rating = item
            .get("rating")
            .and_then(|v| {
                v.as_f64()
                    .map(|f| f as f32)
                    .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            })
            .unwrap_or(0.0);

        let expunged = item
            .get("expunged")
            .and_then(|v| {
                v.as_bool()
                    .or_else(|| v.as_u64().map(|n| n != 0))
                    .or_else(|| v.as_str().map(|s| s == "1" || s == "true"))
            })
            .unwrap_or(false);

        let mut tags = Vec::new();
        if let Some(tag_array) = item.get("tags").and_then(|v| v.as_array()) {
            for t_val in tag_array {
                if let Some(tag_str) = t_val.as_str() {
                    let (namespace, name) = if let Some(pos) = tag_str.find(':') {
                        (tag_str[..pos].to_string(), tag_str[pos + 1..].to_string())
                    } else {
                        (String::new(), tag_str.to_string())
                    };
                    tags.push(Tag { namespace, name });
                }
            }
        }

        summaries.push(GallerySummary {
            gid,
            token,
            title,
            title_jpn,
            category,
            thumb,
            uploader,
            posted,
            file_count,
            rating,
            tags,
            expunged,
            favorite_slot: None,
            favorite_note: None,
        });
    }

    Ok(summaries)
}
