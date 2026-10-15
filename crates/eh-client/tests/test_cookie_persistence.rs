use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use eh_client::client::{ClientConfig, EhClient};
use eh_client::error::ErrorKind;
use eh_client::model::{CookieLogin, RawCookie};

#[tokio::test]
async fn test_offline_cookie_persistence() {
    let temp_dir = std::env::temp_dir().join(format!(
        "rhentai_cookie_test_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp_dir).unwrap();
    let cookie_path = temp_dir.join("cookies.json");

    // Client A creates and imports cookies
    let client_a = EhClient::new(ClientConfig {
        cookie_path: Some(cookie_path.clone()),
        ..Default::default()
    })
    .expect("failed to create client A");

    client_a
        .import_cookies(&[
            RawCookie {
                name: "ipb_member_id".into(),
                value: "998877".into(),
                domain: ".e-hentai.org".into(),
                path: "/".into(),
            },
            RawCookie {
                name: "ipb_pass_hash".into(),
                value: "0123456789abcdef0123456789abcdef".into(),
                domain: ".e-hentai.org".into(),
                path: "/".into(),
            },
            RawCookie {
                name: "igneous".into(),
                value: "ignval123".into(),
                domain: ".exhentai.org".into(),
                path: "/".into(),
            },
        ])
        .expect("import_cookies failed on client A");

    // Drop client A
    drop(client_a);

    // Verify cookies.json on disk
    assert!(cookie_path.exists(), "cookies.json must exist on disk");
    let content = std::fs::read_to_string(&cookie_path).unwrap();
    let json: serde_json::Value =
        serde_json::from_str(&content).expect("cookies.json must be valid JSON");
    let cookies_arr = json
        .as_array()
        .expect("cookies.json must be an array of cookies");

    let raw_cookies: Vec<String> = cookies_arr
        .iter()
        .filter_map(|c| {
            c.get("raw_cookie")
                .and_then(|r| r.as_str())
                .map(ToString::to_string)
        })
        .collect();

    // Verify nw cookies are present for both domains
    assert!(
        raw_cookies
            .iter()
            .any(|c| c.contains("nw=1") && c.contains("e-hentai.org")),
        "cookies.json must contain nw for e-hentai.org"
    );
    assert!(
        raw_cookies
            .iter()
            .any(|c| c.contains("nw=1") && c.contains("exhentai.org")),
        "cookies.json must contain nw for exhentai.org"
    );

    // Verify credential cookies were saved with Max-Age
    assert!(
        raw_cookies
            .iter()
            .any(|c| c.contains("ipb_member_id=998877") && c.contains("Max-Age=")),
        "cookies.json must contain persistent ipb_member_id"
    );
    assert!(
        raw_cookies
            .iter()
            .any(|c| c.contains("ipb_pass_hash=0123456789abcdef0123456789abcdef")),
        "cookies.json must contain persistent ipb_pass_hash"
    );

    // Client B opens the same cookie_path
    let client_b = EhClient::new(ClientConfig {
        cookie_path: Some(cookie_path.clone()),
        ..Default::default()
    })
    .expect("failed to create client B");

    let status = client_b
        .auth_status(false)
        .await
        .expect("client B auth_status failed");

    assert!(
        status.logged_in,
        "client B must report logged_in = true from persisted cookies"
    );
    assert_eq!(
        status.member_id.as_deref(),
        Some("998877"),
        "client B must report member_id = Some(998877)"
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_failed_login_cookie_removal() {
    let temp_dir = std::env::temp_dir().join(format!(
        "rhentai_cookie_removal_test_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp_dir).unwrap();
    let cookie_path = temp_dir.join("cookies.json");

    let client = EhClient::new(ClientConfig {
        cookie_path: Some(cookie_path.clone()),
        ..Default::default()
    })
    .expect("failed to create client");

    // Import credentials
    client
        .import_cookies(&[
            RawCookie {
                name: "ipb_member_id".into(),
                value: "112233".into(),
                domain: ".e-hentai.org".into(),
                path: "/".into(),
            },
            RawCookie {
                name: "ipb_pass_hash".into(),
                value: "abcdef1234567890abcdef1234567890".into(),
                domain: ".e-hentai.org".into(),
                path: "/".into(),
            },
            RawCookie {
                name: "igneous".into(),
                value: "ignval456".into(),
                domain: ".exhentai.org".into(),
                path: "/".into(),
            },
        ])
        .unwrap();

    let status_before = client.auth_status(false).await.unwrap();
    assert!(status_before.logged_in);

    // Call removal helper
    client
        .remove_credential_cookies()
        .expect("remove_credential_cookies failed");

    let status_after = client.auth_status(false).await.unwrap();
    assert!(
        !status_after.logged_in,
        "client must report logged_in = false after removal"
    );
    assert_eq!(
        status_after.member_id, None,
        "member_id must be None after removal"
    );

    // Verify file on disk has NO credentials and still has nw cookies
    let content = std::fs::read_to_string(&cookie_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    let cookies_arr = json.as_array().unwrap();
    let raw_cookies: Vec<String> = cookies_arr
        .iter()
        .filter_map(|c| {
            c.get("raw_cookie")
                .and_then(|r| r.as_str())
                .map(ToString::to_string)
        })
        .collect();

    assert!(
        !raw_cookies.iter().any(|c| c.contains("ipb_member_id")),
        "cookies.json must not contain ipb_member_id after removal"
    );
    assert!(
        !raw_cookies.iter().any(|c| c.contains("ipb_pass_hash")),
        "cookies.json must not contain ipb_pass_hash after removal"
    );
    assert!(
        !raw_cookies.iter().any(|c| c.contains("igneous")),
        "cookies.json must not contain igneous after removal"
    );
    assert!(
        raw_cookies
            .iter()
            .any(|c| c.contains("nw=1") && c.contains("e-hentai.org")),
        "cookies.json must still contain nw for e-hentai.org"
    );
    assert!(
        raw_cookies
            .iter()
            .any(|c| c.contains("nw=1") && c.contains("exhentai.org")),
        "cookies.json must still contain nw for exhentai.org"
    );

    drop(client);

    // New client instance reading the file
    let client_b = EhClient::new(ClientConfig {
        cookie_path: Some(cookie_path.clone()),
        ..Default::default()
    })
    .unwrap();
    let status_b = client_b.auth_status(false).await.unwrap();
    assert!(!status_b.logged_in);
    assert_eq!(status_b.member_id, None);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_content_warning_retry_limit() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let request_count = Arc::new(AtomicUsize::new(0));
    let req_count_clone = request_count.clone();

    tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            req_count_clone.fetch_add(1, Ordering::SeqCst);
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf).await;

            let body = "<html><body><h1>Content Warning</h1>This gallery has content warning</body></html>";
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes()).await;
            let _ = stream.shutdown().await;
        }
    });

    let client = EhClient::new(ClientConfig {
        min_request_interval_ms: 0,
        ..Default::default()
    })
    .unwrap();

    let url = format!("http://127.0.0.1:{}/g/123/abc/", addr.port());
    let err = client.request_html(&url, None).await.unwrap_err();

    assert_eq!(
        err.kind,
        ErrorKind::Parse,
        "Persistent content warning must return ErrorKind::Parse"
    );
    assert!(
        err.message.contains("Content warning bypassed"),
        "Error message should mention content warning: {}",
        err.message
    );

    // Exactly 2 requests: initial + 1 retry with nw=session
    assert_eq!(
        request_count.load(Ordering::SeqCst),
        2,
        "request_html should retry at most once"
    );
}

#[tokio::test]
async fn test_cookie_login_network_error_leaves_no_unverified_credentials() {
    let temp_dir = std::env::temp_dir().join(format!(
        "rhentai_cookie_network_err_test_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp_dir).unwrap();
    let cookie_path = temp_dir.join("cookies.json");

    let client = EhClient::new(ClientConfig {
        cookie_path: Some(cookie_path.clone()),
        proxy: Some("http://127.0.0.1:9".to_string()),
        min_request_interval_ms: 0,
        ..Default::default()
    })
    .expect("failed to create client");

    let res = client
        .login_cookies(&CookieLogin {
            member_id: "1".into(),
            pass_hash: "00000000000000000000000000000000".into(),
            igneous: None,
        })
        .await;

    assert!(res.is_err(), "login_cookies via refused proxy must fail");
    assert_eq!(res.unwrap_err().kind, ErrorKind::Network);

    drop(client);

    let client_new = EhClient::new(ClientConfig {
        cookie_path: Some(cookie_path.clone()),
        ..Default::default()
    })
    .expect("failed to create client_new");

    let status = client_new.auth_status(false).await.unwrap();
    assert!(
        !status.logged_in,
        "new client must report logged_in == false after failed login"
    );
    assert_eq!(status.member_id, None);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_cookie_login_failure_preserves_prior_session() {
    let temp_dir = std::env::temp_dir().join(format!(
        "rhentai_cookie_preserve_test_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp_dir).unwrap();
    let cookie_path = temp_dir.join("cookies.json");

    let client = EhClient::new(ClientConfig {
        cookie_path: Some(cookie_path.clone()),
        proxy: Some("http://127.0.0.1:9".to_string()),
        min_request_interval_ms: 0,
        ..Default::default()
    })
    .expect("failed to create client");

    // Establish prior session with member_id "5"
    client
        .import_cookies(&[
            RawCookie {
                name: "ipb_member_id".into(),
                value: "5".into(),
                domain: ".e-hentai.org".into(),
                path: "/".into(),
            },
            RawCookie {
                name: "ipb_pass_hash".into(),
                value: "55555555555555555555555555555555".into(),
                domain: ".e-hentai.org".into(),
                path: "/".into(),
            },
        ])
        .unwrap();

    let status_before = client.auth_status(false).await.unwrap();
    assert!(status_before.logged_in);
    assert_eq!(status_before.member_id.as_deref(), Some("5"));

    // Attempt failed login with member_id "1"
    let res = client
        .login_cookies(&CookieLogin {
            member_id: "1".into(),
            pass_hash: "11111111111111111111111111111111".into(),
            igneous: None,
        })
        .await;

    assert!(res.is_err());

    drop(client);

    // Verify prior session stays intact
    let client_new = EhClient::new(ClientConfig {
        cookie_path: Some(cookie_path.clone()),
        ..Default::default()
    })
    .unwrap();

    let status_after = client_new.auth_status(false).await.unwrap();
    assert!(
        status_after.logged_in,
        "prior session must remain logged in after failed login attempt"
    );
    assert_eq!(
        status_after.member_id.as_deref(),
        Some("5"),
        "member_id 5 must stay intact"
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[cfg(unix)]
#[tokio::test]
async fn test_cookie_file_mode_enforced_on_existing_file() {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = std::env::temp_dir().join(format!(
        "rhentai_cookie_mode_test_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp_dir).unwrap();
    let cookie_path = temp_dir.join("cookies.json");

    // Pre-create file with mode 0644
    std::fs::write(&cookie_path, b"[]").unwrap();
    std::fs::set_permissions(&cookie_path, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert_eq!(
        std::fs::metadata(&cookie_path)
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o644
    );

    let client = EhClient::new(ClientConfig {
        cookie_path: Some(cookie_path.clone()),
        ..Default::default()
    })
    .unwrap();

    // Trigger save by logging out (which clears and saves)
    client.logout().unwrap();

    let mode = std::fs::metadata(&cookie_path)
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(
        mode, 0o600,
        "Cookie file mode must be tightened to 0600 on existing files"
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}
