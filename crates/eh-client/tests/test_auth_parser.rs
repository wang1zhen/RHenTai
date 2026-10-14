use eh_client::error::ErrorKind;
use eh_client::parse::parse_login_response;
use reqwest::StatusCode;
use reqwest::header::{HeaderMap, HeaderValue, SET_COOKIE};

const LOGIN_ERROR: &str = include_str!("fixtures/synthetic_login_error.html");
const LOGIN_CAPTCHA: &str = include_str!("fixtures/synthetic_login_captcha.html");
const CLOUDFLARE: &str = include_str!("fixtures/synthetic_cloudflare.html");

#[test]
fn test_login_success_parser() {
    let mut headers = HeaderMap::new();
    headers.append(
        SET_COOKIE,
        HeaderValue::from_static("ipb_member_id=12345; path=/; domain=.e-hentai.org"),
    );
    headers.append(
        SET_COOKIE,
        HeaderValue::from_static(
            "ipb_pass_hash=0123456789abcdef0123456789abcdef; path=/; domain=.e-hentai.org",
        ),
    );

    let res = parse_login_response(StatusCode::OK, &headers, "<html>Success</html>")
        .expect("should parse login success");
    assert_eq!(res.member_id, "12345");
    assert_eq!(res.pass_hash, "0123456789abcdef0123456789abcdef");
}

#[test]
fn test_login_error_parser() {
    let headers = HeaderMap::new();
    let err = parse_login_response(StatusCode::OK, &headers, LOGIN_ERROR)
        .expect_err("should return error");
    assert_eq!(err.kind, ErrorKind::LoginFailed);
    assert!(err.message.contains("Username or password incorrect"));
}

#[test]
fn test_login_captcha_parser() {
    let headers = HeaderMap::new();
    let err = parse_login_response(StatusCode::OK, &headers, LOGIN_CAPTCHA)
        .expect_err("should return error");
    assert_eq!(err.kind, ErrorKind::LoginFailed);
    assert!(err.message.contains("captcha required"));
}

#[test]
fn test_login_cloudflare_parser() {
    let mut headers = HeaderMap::new();
    headers.insert("cf-mitigated", HeaderValue::from_static("challenge"));
    let err = parse_login_response(StatusCode::FORBIDDEN, &headers, CLOUDFLARE)
        .expect_err("should return error");
    assert_eq!(err.kind, ErrorKind::Cloudflare);
}
