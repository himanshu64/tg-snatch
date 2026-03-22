use tg_snatch::cli::FileTypeFilter;
use tg_snatch::files::catalog::FileEntry;
use tg_snatch::files::filter::FileType;
use tg_snatch::storage::db::Database;

fn make_entry(unique_id: &str, file_type: FileType, chat_id: i64) -> FileEntry {
    FileEntry {
        file_id: format!("fid_{}", unique_id),
        file_unique_id: unique_id.to_string(),
        file_type,
        file_name: Some(format!("{}.pdf", unique_id)),
        file_size: Some(1024),
        mime_type: Some("application/pdf".to_string()),
        chat_id,
        message_id: 1,
        date: 1700000000,
        caption: None,
        downloaded: false,
    }
}

#[test]
fn insert_and_query_file() {
    let db = Database::open_in_memory().unwrap();
    let entry = make_entry("abc123", FileType::Pdf, -100);

    let inserted = db.insert_file(&entry).unwrap();
    assert!(inserted);

    let files = db.query_files(None, None, 100, false).unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].file_unique_id, "abc123");
    assert_eq!(files[0].file_type, FileType::Pdf);
}

#[test]
fn insert_duplicate_is_ignored() {
    let db = Database::open_in_memory().unwrap();
    let entry = make_entry("dup1", FileType::Pdf, -100);

    assert!(db.insert_file(&entry).unwrap());
    assert!(!db.insert_file(&entry).unwrap()); // duplicate

    let files = db.query_files(None, None, 100, false).unwrap();
    assert_eq!(files.len(), 1);
}

#[test]
fn query_filter_by_chat_id() {
    let db = Database::open_in_memory().unwrap();
    db.insert_file(&make_entry("a", FileType::Pdf, -100))
        .unwrap();
    db.insert_file(&make_entry("b", FileType::Pdf, -200))
        .unwrap();
    db.insert_file(&make_entry("c", FileType::Image, -100))
        .unwrap();

    let files = db.query_files(Some(-100), None, 100, false).unwrap();
    assert_eq!(files.len(), 2);

    let files = db.query_files(Some(-200), None, 100, false).unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].file_unique_id, "b");
}

#[test]
fn query_filter_by_type() {
    let db = Database::open_in_memory().unwrap();
    db.insert_file(&make_entry("pdf1", FileType::Pdf, -100))
        .unwrap();
    db.insert_file(&make_entry("img1", FileType::Image, -100))
        .unwrap();
    db.insert_file(&make_entry("vid1", FileType::Video, -100))
        .unwrap();

    let files = db
        .query_files(None, Some(&FileTypeFilter::Pdf), 100, false)
        .unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].file_type, FileType::Pdf);

    let files = db
        .query_files(None, Some(&FileTypeFilter::Image), 100, false)
        .unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].file_type, FileType::Image);

    let files = db
        .query_files(None, Some(&FileTypeFilter::All), 100, false)
        .unwrap();
    assert_eq!(files.len(), 3);
}

#[test]
fn query_limit() {
    let db = Database::open_in_memory().unwrap();
    for i in 0..10 {
        let mut entry = make_entry(&format!("file_{}", i), FileType::Document, -100);
        entry.date = 1700000000 + i;
        db.insert_file(&entry).unwrap();
    }

    let files = db.query_files(None, None, 5, false).unwrap();
    assert_eq!(files.len(), 5);
}

#[test]
fn mark_downloaded() {
    let db = Database::open_in_memory().unwrap();
    db.insert_file(&make_entry("dl1", FileType::Pdf, -100))
        .unwrap();

    // Before marking
    let files = db.query_files(None, None, 100, true).unwrap();
    assert_eq!(files.len(), 1);

    db.mark_downloaded("dl1").unwrap();

    // After marking — only_not_downloaded should exclude it
    let files = db.query_files(None, None, 100, true).unwrap();
    assert_eq!(files.len(), 0);

    // But it should still appear when not filtering
    let files = db.query_files(None, None, 100, false).unwrap();
    assert_eq!(files.len(), 1);
    assert!(files[0].downloaded);
}

#[test]
fn offset_persistence() {
    let db = Database::open_in_memory().unwrap();

    assert_eq!(db.get_offset().unwrap(), None);

    db.set_offset(42).unwrap();
    assert_eq!(db.get_offset().unwrap(), Some(42));

    db.set_offset(100).unwrap();
    assert_eq!(db.get_offset().unwrap(), Some(100));
}

#[test]
fn query_combined_filters() {
    let db = Database::open_in_memory().unwrap();
    db.insert_file(&make_entry("p1", FileType::Pdf, -100))
        .unwrap();
    db.insert_file(&make_entry("p2", FileType::Pdf, -200))
        .unwrap();
    db.insert_file(&make_entry("i1", FileType::Image, -100))
        .unwrap();

    // Chat -100 + PDF type
    let files = db
        .query_files(Some(-100), Some(&FileTypeFilter::Pdf), 100, false)
        .unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].file_unique_id, "p1");
}

#[test]
fn insert_file_with_no_optional_fields() {
    let db = Database::open_in_memory().unwrap();
    let entry = FileEntry {
        file_id: "fid_minimal".to_string(),
        file_unique_id: "minimal".to_string(),
        file_type: FileType::Voice,
        file_name: None,
        file_size: None,
        mime_type: None,
        chat_id: -100,
        message_id: 1,
        date: 1700000000,
        caption: None,
        downloaded: false,
    };

    assert!(db.insert_file(&entry).unwrap());
    let files = db.query_files(None, None, 100, false).unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].file_name, None);
    assert_eq!(files[0].file_size, None);
}
