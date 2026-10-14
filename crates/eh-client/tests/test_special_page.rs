use eh_client::error::ErrorKind;
use eh_client::parse::detect_special_page;
use reqwest::StatusCode;
use reqwest::header::{HeaderMap, HeaderValue, LOCATION};

const CLOUDFLARE: &str = include_str!("fixtures/synthetic_cloudflare.html");
const IP_BAN: &str = include_str!("fixtures/synthetic_ip_ban.html");
const IMAGE_LIMIT: &str = include_str!("fixtures/synthetic_image_limit.html");
const REMOVED: &str = include_str!("fixtures/synthetic_removed.html");
const WRONG_TOKEN: &str = include_str!("fixtures/wrong_token.html");

#[test]
fn test_special_page_cloudflare() {
    let mut headers = HeaderMap::new();
    headers.insert("cf-mitigated", HeaderValue::from_static("challenge"));
    let err = detect_special_page(
        StatusCode::FORBIDDEN,
        &headers,
        "https://forums.e-hentai.org/index.php",
        CLOUDFLARE,
    )
    .unwrap_err();
    assert_eq!(err.kind, ErrorKind::Cloudflare);

    // Title match without header
    let empty_headers = HeaderMap::new();
    let err2 = detect_special_page(
        StatusCode::SERVICE_UNAVAILABLE,
        &empty_headers,
        "https://forums.e-hentai.org/index.php",
        CLOUDFLARE,
    )
    .unwrap_err();
    assert_eq!(err2.kind, ErrorKind::Cloudflare);
}

#[test]
fn test_special_page_ip_ban() {
    let headers = HeaderMap::new();
    let err =
        detect_special_page(StatusCode::OK, &headers, "https://e-hentai.org/", IP_BAN).unwrap_err();
    assert_eq!(err.kind, ErrorKind::Banned);
    assert!(err.message.contains("expire in 2 hours and 15 minutes"));
}

#[test]
fn test_special_page_login_redirect() {
    let mut headers = HeaderMap::new();
    headers.insert(
        LOCATION,
        HeaderValue::from_static("https://e-hentai.org/bounce_login.php?b=d&bt=1-1"),
    );
    let err = detect_special_page(
        StatusCode::FOUND,
        &headers,
        "https://e-hentai.org/home.php",
        "",
    )
    .unwrap_err();
    assert_eq!(err.kind, ErrorKind::LoginRequired);
}

#[test]
fn test_special_page_sad_panda() {
    let headers = HeaderMap::new();
    let err =
        detect_special_page(StatusCode::OK, &headers, "https://exhentai.org/", "").unwrap_err();
    assert_eq!(err.kind, ErrorKind::NoExAccess);
}

#[test]
fn test_special_page_wrong_token() {
    let headers = HeaderMap::new();
    let err = detect_special_page(
        StatusCode::OK,
        &headers,
        "https://e-hentai.org/g/123/wrongtoken/",
        WRONG_TOKEN,
    )
    .unwrap_err();
    assert_eq!(err.kind, ErrorKind::NotFound);
    assert_eq!(err.message, "Key missing, or incorrect key provided.");
}

#[test]
fn test_special_page_removed() {
    let headers = HeaderMap::new();
    let err = detect_special_page(
        StatusCode::OK,
        &headers,
        "https://e-hentai.org/g/123/456/",
        REMOVED,
    )
    .unwrap_err();
    assert_eq!(err.kind, ErrorKind::NotFound);
    assert!(err.message.contains("removed due to a copyright claim"));
}

#[test]
fn test_special_page_image_limit() {
    let headers = HeaderMap::new();
    let err = detect_special_page(
        StatusCode::OK,
        &headers,
        "https://e-hentai.org/s/123/456-1",
        IMAGE_LIMIT,
    )
    .unwrap_err();
    assert_eq!(err.kind, ErrorKind::ImageLimit);

    // 509 URL
    let err_509 = detect_special_page(StatusCode::OK, &headers, "https://ehgt.org/g/509.gif", "")
        .unwrap_err();
    assert_eq!(err_509.kind, ErrorKind::ImageLimit);
}

#[test]
fn test_special_page_fatal_error() {
    let headers = HeaderMap::new();
    let err = detect_special_page(
        StatusCode::OK,
        &headers,
        "https://e-hentai.org/",
        "Page load has been aborted due to a fatal error",
    )
    .unwrap_err();
    assert_eq!(err.kind, ErrorKind::Http);
}

#[test]
fn test_special_page_non_2xx() {
    let headers = HeaderMap::new();
    let err = detect_special_page(
        StatusCode::INTERNAL_SERVER_ERROR,
        &headers,
        "https://e-hentai.org/",
        "Internal server error",
    )
    .unwrap_err();
    assert_eq!(err.kind, ErrorKind::Http);
    assert!(err.message.contains("500"));
}

#[test]
fn test_special_page_normal_pass() {
    let headers = HeaderMap::new();
    let res = detect_special_page(
        StatusCode::OK,
        &headers,
        "https://e-hentai.org/",
        "<html><head><title>E-Hentai</title></head><body>Normal</body></html>",
    );
    assert!(res.is_ok());
}
