use tg_snatch::files::filter::FileType;

#[test]
fn file_type_as_str_roundtrip() {
    let types = vec![
        FileType::Pdf,
        FileType::Image,
        FileType::Video,
        FileType::Audio,
        FileType::Document,
        FileType::Animation,
        FileType::Voice,
    ];

    for ft in types {
        let s = ft.as_str();
        let recovered = FileType::from_db(s).unwrap();
        assert_eq!(ft, recovered);
    }
}

#[test]
fn file_type_from_db_unknown_returns_none() {
    assert_eq!(FileType::from_db("unknown"), None);
    assert_eq!(FileType::from_db(""), None);
    assert_eq!(FileType::from_db("PDF"), None); // case-sensitive
}

#[test]
fn file_type_display() {
    assert_eq!(format!("{}", FileType::Pdf), "PDF");
    assert_eq!(format!("{}", FileType::Image), "IMAGE");
    assert_eq!(format!("{}", FileType::Video), "VIDEO");
    assert_eq!(format!("{}", FileType::Audio), "AUDIO");
    assert_eq!(format!("{}", FileType::Document), "DOCUMENT");
    assert_eq!(format!("{}", FileType::Animation), "ANIMATION");
    assert_eq!(format!("{}", FileType::Voice), "VOICE");
}

#[test]
fn file_type_subdirectories() {
    assert_eq!(FileType::Pdf.subdir(), "pdfs");
    assert_eq!(FileType::Image.subdir(), "images");
    assert_eq!(FileType::Video.subdir(), "videos");
    assert_eq!(FileType::Audio.subdir(), "audio");
    assert_eq!(FileType::Document.subdir(), "documents");
    assert_eq!(FileType::Animation.subdir(), "animations");
    assert_eq!(FileType::Voice.subdir(), "voice");
}
