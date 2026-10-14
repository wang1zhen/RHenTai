use scraper::{Html, Selector};

use crate::error::{EhError, Result};

/// Detect special pages and HTTP error conditions per EH_PROTOCOL §4.
/// Applied to every HTML / API response before parsing.
pub fn detect_special_page(
    status: reqwest::StatusCode,
    headers: &reqwest::header::HeaderMap,
    url: &str,
    body: &str,
) -> Result<()> {
    // 1. Cloudflare challenge
    if (status == reqwest::StatusCode::FORBIDDEN
        || status == reqwest::StatusCode::SERVICE_UNAVAILABLE)
        && (headers
            .get("cf-mitigated")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.contains("challenge"))
            .unwrap_or(false)
            || body.contains("<title>Just a moment...</title>")
            || body.contains("Just a moment..."))
    {
        return Err(EhError::cloudflare(
            "Cloudflare challenge encountered; web login or cookie login recommended",
        ));
    }

    // 2. IP ban
    let trimmed = body.trim();
    if trimmed.starts_with("Your IP address has been temporarily banned")
        || trimmed.starts_with("This IP address has been temporarily banned")
    {
        return Err(EhError::banned(trimmed));
    }
    if let Some(pos) = body.find("Your IP address has been temporarily banned") {
        let msg = extract_ban_message(&body[pos..]);
        return Err(EhError::banned(msg));
    }
    if let Some(pos) = body.find("This IP address has been temporarily banned") {
        let msg = extract_ban_message(&body[pos..]);
        return Err(EhError::banned(msg));
    }

    // 3. Login redirect (302 to bounce_login.php, or landing on bounce_login.php)
    if status.is_redirection()
        && let Some(loc) = headers
            .get(reqwest::header::LOCATION)
            .and_then(|v| v.to_str().ok())
        && loc.contains("bounce_login.php")
    {
        return Err(EhError::login_required("Login required (bounce_login.php)"));
    }
    if url.contains("bounce_login.php") {
        return Err(EhError::login_required("Login required (bounce_login.php)"));
    }

    // 4. Sad panda: exhentai.org 200 with empty body (or tiny image)
    if url.contains("exhentai.org")
        && status.is_success()
        && (trimmed.is_empty()
            || (trimmed.len() < 100
                && (trimmed.contains("sadpanda") || trimmed.starts_with("GIF8"))))
    {
        return Err(EhError::no_ex_access(
            "Sad panda: ExHentai access denied or session expired",
        ));
    }

    // 5. Wrong token
    if body.contains("Key missing, or incorrect key provided.") {
        return Err(EhError::not_found(
            "Key missing, or incorrect key provided.",
        ));
    }

    // 6. Removed or unavailable galleries
    let removed_markers = [
        "This gallery has been removed",
        "This gallery is unavailable due to a copyright claim",
        "Gallery not found.",
        "This gallery is pining for the fjords.",
    ];
    for marker in removed_markers {
        if body.contains(marker) {
            let msg = extract_removed_message(body).unwrap_or_else(|| marker.to_string());
            return Err(EhError::not_found(msg));
        }
    }

    // 7. Image limit
    if body.contains("You have exceeded your image viewing limits") || url.ends_with("/509.gif") {
        return Err(EhError::image_limit(
            "You have exceeded your image viewing limits",
        ));
    }

    // 8. Fatal server error
    if body.contains("Page load has been aborted due to a fatal error") {
        return Err(EhError::http(
            "Page load has been aborted due to a fatal error",
        ));
    }

    // 9. Non-2xx (other)
    if !status.is_success() {
        return Err(EhError::http(format!("HTTP status {status}")));
    }

    Ok(())
}

pub fn is_content_warning(body: &str) -> bool {
    body.contains("<h1>Content Warning</h1>")
}

fn extract_ban_message(s: &str) -> String {
    if let Some(end) = s.find("</") {
        s[..end].trim().to_string()
    } else if let Some(end) = s.find('\n') {
        s[..end].trim().to_string()
    } else {
        s.trim().to_string()
    }
}

fn extract_removed_message(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let selector = Selector::parse(".d p").ok()?;
    document
        .select(&selector)
        .next()
        .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
}
