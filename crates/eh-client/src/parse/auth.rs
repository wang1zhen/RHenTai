use regex::Regex;
use std::sync::LazyLock;

use crate::error::{EhError, Result};

static ERROR_P_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"<h4>The error returned was:</h4>\s*<p>(.+?)</p>").expect("valid regex")
});

static POSTCOLOR_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<span class="postcolor">(.+?)</span>"#).expect("valid regex"));

static COOKIE_PAIR_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([^=]+)=([^;]+)").expect("valid regex"));

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginSuccess {
    pub member_id: String,
    pub pass_hash: String,
}

/// Parse login response (status, headers, body) per EH_PROTOCOL §2.3
pub fn parse_login_response(
    status: reqwest::StatusCode,
    headers: &reqwest::header::HeaderMap,
    body: &str,
) -> Result<LoginSuccess> {
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

    // 2. Captcha
    if body.contains("The captcha was not entered correctly") {
        return Err(EhError::login_failed("captcha required, use web login"));
    }

    // 3. Error text
    if let Some(caps) = ERROR_P_REGEX.captures(body) {
        let msg = caps[1].trim().to_string();
        return Err(EhError::login_failed(msg));
    }
    if let Some(caps) = POSTCOLOR_REGEX.captures(body) {
        let msg = caps[1].trim().to_string();
        return Err(EhError::login_failed(msg));
    }

    // 4. Success cookies in Set-Cookie headers
    let mut member_id = None;
    let mut pass_hash = None;

    for val in headers.get_all(reqwest::header::SET_COOKIE) {
        if let Ok(s) = val.to_str()
            && let Some(caps) = COOKIE_PAIR_REGEX.captures(s)
        {
            let name = caps[1].trim();
            let value = caps[2].trim().to_string();
            if name == "ipb_member_id" {
                member_id = Some(value);
            } else if name == "ipb_pass_hash" {
                pass_hash = Some(value);
            }
        }
    }

    match (member_id, pass_hash) {
        (Some(id), Some(hash)) => Ok(LoginSuccess {
            member_id: id,
            pass_hash: hash,
        }),
        _ => Err(EhError::login_failed(
            "Login failed: missing session cookies",
        )),
    }
}
