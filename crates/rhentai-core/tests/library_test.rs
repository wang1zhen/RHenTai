use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use eh_client::Category;
use rhentai_core::{Library, LocalKind, Store};
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

fn create_jpeg_bytes(marker: u8) -> Vec<u8> {
    let mut b = vec![
        0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, b'J', b'F', b'I', b'F', 0x00,
    ];
    b.push(marker);
    b.extend_from_slice(&[0xFF, 0xD9]);
    b
}

fn create_png_bytes(marker: u8) -> Vec<u8> {
    let mut b = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    b.push(marker);
    b
}

fn create_webp_bytes() -> Vec<u8> {
    vec![
        b'R', b'I', b'F', b'F', 0x20, 0x00, 0x00, 0x00, b'W', b'E', b'B', b'P', b'V', b'P', b'8',
        b' ',
    ]
}

#[test]
fn test_library_scan_and_page_bytes() {
    let temp_dir = std::env::temp_dir().join(format!("rhentai_lib_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    // 1. Download-style directory with rhentai.json
    let dl_dir = temp_dir.join("downloaded_manga");
    fs::create_dir_all(&dl_dir).unwrap();
    fs::write(dl_dir.join("0001.jpg"), create_jpeg_bytes(1)).unwrap();
    fs::write(dl_dir.join("0002.png"), create_png_bytes(2)).unwrap();
    let rhentai_json = r#"{
        "version": 1,
        "site": "eh",
        "gid": 99999,
        "token": "tok123",
        "title": "My Downloaded Manga",
        "titleJpn": "僕の漫画",
        "category": "manga",
        "uploader": "testuser",
        "posted": 1700000000,
        "tags": [{"namespace": "artist", "name": "artistname"}],
        "pageCount": 2,
        "pages": ["0001.jpg", "0002.png"],
        "downloadedAt": 1700000100,
        "original": false
    }"#;
    fs::write(dl_dir.join("rhentai.json"), rhentai_json).unwrap();

    // 2. Plain image folder with non-images, hidden files, and .part files
    let plain_dir = temp_dir.join("plain_folder");
    fs::create_dir_all(&plain_dir).unwrap();
    fs::write(plain_dir.join("page1.jpg"), create_jpeg_bytes(10)).unwrap();
    fs::write(plain_dir.join("page2.webp"), create_webp_bytes()).unwrap();
    fs::write(plain_dir.join("notes.txt"), b"some notes").unwrap();
    fs::write(plain_dir.join(".hidden.jpg"), create_jpeg_bytes(99)).unwrap();
    fs::write(
        plain_dir.join("downloading.jpg.part"),
        create_jpeg_bytes(88),
    )
    .unwrap();

    // 3. Valid .cbz archive
    let cbz_path = temp_dir.join("comic.cbz");
    let cbz_file = File::create(&cbz_path).unwrap();
    let mut zip = ZipWriter::new(cbz_file);
    let options = SimpleFileOptions::default();
    zip.start_file("01.jpg", options).unwrap();
    zip.write_all(&create_jpeg_bytes(20)).unwrap();
    zip.start_file("02.png", options).unwrap();
    zip.write_all(&create_png_bytes(21)).unwrap();
    zip.start_file("info.txt", options).unwrap();
    zip.write_all(b"not an image").unwrap();
    zip.finish().unwrap();

    // 4. Corrupt .zip
    let corrupt_path = temp_dir.join("corrupt.zip");
    fs::write(&corrupt_path, b"this is completely invalid zip data!").unwrap();

    // 5. Empty directory
    let empty_dir = temp_dir.join("empty_dir");
    fs::create_dir_all(&empty_dir).unwrap();

    // 6. Non-image files in root
    fs::write(temp_dir.join("readme.txt"), b"hello world").unwrap();
    fs::write(temp_dir.join("document.pdf"), b"%PDF-1.4...").unwrap();

    // Run library scan
    let store = Store::in_memory().unwrap();
    let library = Library::new(store);
    let (scanned, warnings) = library
        .rescan_report(std::slice::from_ref(&temp_dir))
        .unwrap();

    // Exactly 1 warning for corrupt.zip
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].path, corrupt_path);
    assert!(!warnings[0].reason.is_empty());

    // Exactly 3 galleries should be found: downloaded_manga, plain_folder, comic.cbz
    assert_eq!(scanned.len(), 3);

    // Verify downloaded_manga
    let dl_gallery = scanned
        .iter()
        .find(|g| g.title == "My Downloaded Manga")
        .expect("downloaded_manga should be found");
    assert_eq!(dl_gallery.kind, LocalKind::Folder);
    assert_eq!(dl_gallery.gid, Some(99999));
    assert_eq!(dl_gallery.token.as_deref(), Some("tok123"));
    assert_eq!(dl_gallery.category, Some(Category::Manga));
    assert!(dl_gallery.downloaded);
    assert_eq!(dl_gallery.page_count, 2);

    // Verify plain_folder
    let plain_gallery = scanned
        .iter()
        .find(|g| g.title == "plain_folder")
        .expect("plain_folder should be found");
    assert_eq!(plain_gallery.kind, LocalKind::Folder);
    assert_eq!(plain_gallery.gid, None);
    assert!(!plain_gallery.downloaded);
    assert_eq!(plain_gallery.page_count, 2); // only page1.jpg and page2.webp

    // Verify cbz gallery
    let cbz_gallery = scanned
        .iter()
        .find(|g| g.title == "comic")
        .expect("comic.cbz should be found");
    assert_eq!(cbz_gallery.kind, LocalKind::Archive);
    assert_eq!(cbz_gallery.page_count, 2); // only 01.jpg and 02.png
    assert!(!cbz_gallery.downloaded);

    // Corrupt zip and empty dir must not be in scanned
    assert!(!scanned.iter().any(|g| g.title.contains("corrupt")));
    assert!(!scanned.iter().any(|g| g.title.contains("empty")));

    // Test local pages list with natural ordering
    let dl_pages = library.pages(&dl_gallery.id).unwrap();
    assert_eq!(dl_pages.count, 2);
    assert_eq!(dl_pages.names, vec!["0001.jpg", "0002.png"]);

    let cbz_pages = library.pages(&cbz_gallery.id).unwrap();
    assert_eq!(cbz_pages.count, 2);
    assert_eq!(cbz_pages.names, vec!["01.jpg", "02.png"]);

    // Test page bytes from folder
    let (p0_bytes, p0_type) = library.page(&dl_gallery.id, 0).unwrap();
    assert_eq!(p0_type, "image/jpeg");
    assert_eq!(p0_bytes.as_ref(), create_jpeg_bytes(1).as_slice());

    let (p1_bytes, p1_type) = library.page(&dl_gallery.id, 1).unwrap();
    assert_eq!(p1_type, "image/png");
    assert_eq!(p1_bytes.as_ref(), create_png_bytes(2).as_slice());

    // Test page bytes from zip/cbz
    let (cbz_p0_bytes, cbz_p0_type) = library.page(&cbz_gallery.id, 0).unwrap();
    assert_eq!(cbz_p0_type, "image/jpeg");
    assert_eq!(cbz_p0_bytes.as_ref(), create_jpeg_bytes(20).as_slice());

    let (cbz_p1_bytes, cbz_p1_type) = library.page(&cbz_gallery.id, 1).unwrap();
    assert_eq!(cbz_p1_type, "image/png");
    assert_eq!(cbz_p1_bytes.as_ref(), create_png_bytes(21).as_slice());

    // Out of range page index returns error
    assert!(library.page(&cbz_gallery.id, 5).is_err());

    // Test removal outside roots fails, removal inside succeeds
    let outside = Path::new("/some/fake/outside/path");
    assert!(
        library
            .remove(&cbz_gallery.id, true, &[outside.to_path_buf()])
            .is_err()
    );
    assert!(
        library
            .remove(&cbz_gallery.id, true, std::slice::from_ref(&temp_dir))
            .is_ok()
    );
    assert!(!cbz_path.exists());

    let _ = fs::remove_dir_all(&temp_dir);
}
