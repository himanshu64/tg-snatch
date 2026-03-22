use tg_snatch::files::catalog::FileEntry;
use tg_snatch::files::filter::FileType;

fn make_entry(file_type: FileType, name: Option<&str>, size: Option<u64>) -> FileEntry {
    FileEntry {
        file_id: "fid_test".to_string(),
        file_unique_id: "uid_test".to_string(),
        file_type,
        file_name: name.map(|s| s.to_string()),
        file_size: size,
        mime_type: None,
        chat_id: -100,
        message_id: 1,
        date: 1700000000,
        caption: None,
        downloaded: false,
    }
}

#[test]
fn display_name_with_filename() {
    let entry = make_entry(FileType::Pdf, Some("report.pdf"), Some(1024));
    assert_eq!(entry.display_name(), "report.pdf");
}

#[test]
fn display_name_fallback_uses_unique_id_and_ext() {
    let entry = make_entry(FileType::Pdf, None, None);
    assert_eq!(entry.display_name(), "uid_test.pdf");

    let entry = make_entry(FileType::Image, None, None);
    assert_eq!(entry.display_name(), "uid_test.jpg");

    let entry = make_entry(FileType::Video, None, None);
    assert_eq!(entry.display_name(), "uid_test.mp4");

    let entry = make_entry(FileType::Audio, None, None);
    assert_eq!(entry.display_name(), "uid_test.mp3");

    let entry = make_entry(FileType::Document, None, None);
    assert_eq!(entry.display_name(), "uid_test.bin");

    let entry = make_entry(FileType::Animation, None, None);
    assert_eq!(entry.display_name(), "uid_test.gif");

    let entry = make_entry(FileType::Voice, None, None);
    assert_eq!(entry.display_name(), "uid_test.ogg");
}

#[test]
fn display_size_formats_bytes() {
    let entry = make_entry(FileType::Pdf, None, Some(500));
    assert_eq!(entry.display_size(), "500 B");
}

#[test]
fn display_size_formats_kilobytes() {
    let entry = make_entry(FileType::Pdf, None, Some(2048));
    assert_eq!(entry.display_size(), "2.0 KB");
}

#[test]
fn display_size_formats_megabytes() {
    let entry = make_entry(FileType::Pdf, None, Some(5 * 1024 * 1024));
    assert_eq!(entry.display_size(), "5.0 MB");
}

#[test]
fn display_size_formats_gigabytes() {
    let entry = make_entry(FileType::Pdf, None, Some(2 * 1024 * 1024 * 1024));
    assert_eq!(entry.display_size(), "2.0 GB");
}

#[test]
fn display_size_unknown_when_none() {
    let entry = make_entry(FileType::Pdf, None, None);
    assert_eq!(entry.display_size(), "unknown");
}
