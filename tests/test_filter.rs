use tg_snatch::files::catalog::FileEntry;
use tg_snatch::files::filter::{classify_message, FileType};
use tg_snatch::telegram::types::*;

fn make_chat() -> Chat {
    Chat {
        id: -1001234567890,
        title: Some("Test Channel".to_string()),
        chat_type: "channel".to_string(),
    }
}

fn base_message() -> Message {
    Message {
        message_id: 1,
        chat: make_chat(),
        date: 1700000000,
        caption: None,
        document: None,
        photo: None,
        video: None,
        audio: None,
        voice: None,
        animation: None,
    }
}

#[test]
fn classify_pdf_document() {
    let mut msg = base_message();
    msg.document = Some(Document {
        file_id: "file_abc".to_string(),
        file_unique_id: "unique_abc".to_string(),
        file_name: Some("report.pdf".to_string()),
        mime_type: Some("application/pdf".to_string()),
        file_size: Some(1024000),
    });

    let entries = classify_message(&msg);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].file_type, FileType::Pdf);
    assert_eq!(entries[0].file_name.as_deref(), Some("report.pdf"));
    assert_eq!(entries[0].chat_id, -1001234567890);
}

#[test]
fn classify_generic_document() {
    let mut msg = base_message();
    msg.document = Some(Document {
        file_id: "file_doc".to_string(),
        file_unique_id: "unique_doc".to_string(),
        file_name: Some("data.xlsx".to_string()),
        mime_type: Some("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string()),
        file_size: Some(5000),
    });

    let entries = classify_message(&msg);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].file_type, FileType::Document);
}

#[test]
fn classify_photo_picks_largest() {
    let mut msg = base_message();
    msg.photo = Some(vec![
        PhotoSize {
            file_id: "small".to_string(),
            file_unique_id: "small_u".to_string(),
            width: 100,
            height: 100,
            file_size: Some(5000),
        },
        PhotoSize {
            file_id: "medium".to_string(),
            file_unique_id: "medium_u".to_string(),
            width: 800,
            height: 600,
            file_size: Some(50000),
        },
        PhotoSize {
            file_id: "large".to_string(),
            file_unique_id: "large_u".to_string(),
            width: 1920,
            height: 1080,
            file_size: Some(200000),
        },
    ]);

    let entries = classify_message(&msg);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].file_type, FileType::Image);
    assert_eq!(entries[0].file_id, "large"); // Should pick last (largest)
}

#[test]
fn classify_video() {
    let mut msg = base_message();
    msg.video = Some(Video {
        file_id: "vid_1".to_string(),
        file_unique_id: "vid_u1".to_string(),
        file_name: Some("clip.mp4".to_string()),
        mime_type: Some("video/mp4".to_string()),
        file_size: Some(10_000_000),
        duration: 30,
    });

    let entries = classify_message(&msg);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].file_type, FileType::Video);
    assert_eq!(entries[0].file_name.as_deref(), Some("clip.mp4"));
}

#[test]
fn classify_audio_with_title_fallback() {
    let mut msg = base_message();
    msg.audio = Some(Audio {
        file_id: "aud_1".to_string(),
        file_unique_id: "aud_u1".to_string(),
        file_name: None,
        mime_type: Some("audio/mpeg".to_string()),
        file_size: Some(3_000_000),
        duration: 180,
        title: Some("My Song".to_string()),
        performer: Some("Artist".to_string()),
    });

    let entries = classify_message(&msg);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].file_type, FileType::Audio);
    assert_eq!(entries[0].file_name.as_deref(), Some("My Song.mp3"));
}

#[test]
fn classify_voice() {
    let mut msg = base_message();
    msg.voice = Some(Voice {
        file_id: "voice_1".to_string(),
        file_unique_id: "voice_u1".to_string(),
        mime_type: Some("audio/ogg".to_string()),
        file_size: Some(50000),
        duration: 10,
    });

    let entries = classify_message(&msg);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].file_type, FileType::Voice);
    assert_eq!(entries[0].file_name, None);
}

#[test]
fn classify_animation() {
    let mut msg = base_message();
    msg.animation = Some(Animation {
        file_id: "anim_1".to_string(),
        file_unique_id: "anim_u1".to_string(),
        file_name: Some("funny.gif".to_string()),
        mime_type: Some("video/mp4".to_string()),
        file_size: Some(500000),
    });

    let entries = classify_message(&msg);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].file_type, FileType::Animation);
}

#[test]
fn classify_empty_message() {
    let msg = base_message();
    let entries = classify_message(&msg);
    assert!(entries.is_empty());
}

#[test]
fn classify_message_with_multiple_attachments() {
    let mut msg = base_message();
    msg.document = Some(Document {
        file_id: "doc_1".to_string(),
        file_unique_id: "doc_u1".to_string(),
        file_name: Some("file.txt".to_string()),
        mime_type: Some("text/plain".to_string()),
        file_size: Some(100),
    });
    msg.photo = Some(vec![PhotoSize {
        file_id: "photo_1".to_string(),
        file_unique_id: "photo_u1".to_string(),
        width: 800,
        height: 600,
        file_size: Some(50000),
    }]);

    let entries = classify_message(&msg);
    assert_eq!(entries.len(), 2);
}

#[test]
fn classify_preserves_caption() {
    let mut msg = base_message();
    msg.caption = Some("Important document".to_string());
    msg.document = Some(Document {
        file_id: "doc_cap".to_string(),
        file_unique_id: "doc_cap_u".to_string(),
        file_name: Some("doc.pdf".to_string()),
        mime_type: Some("application/pdf".to_string()),
        file_size: Some(1000),
    });

    let entries = classify_message(&msg);
    assert_eq!(entries[0].caption.as_deref(), Some("Important document"));
}
