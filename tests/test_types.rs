use tg_snatch::telegram::types::*;

#[test]
fn deserialize_api_response_success() {
    let json = r#"{"ok": true, "result": {"id": 123, "is_bot": true, "first_name": "TestBot", "username": "test_bot"}}"#;
    let resp: ApiResponse<User> = serde_json::from_str(json).unwrap();
    assert!(resp.ok);
    let user = resp.result.unwrap();
    assert_eq!(user.id, 123);
    assert_eq!(user.first_name, "TestBot");
    assert_eq!(user.username.as_deref(), Some("test_bot"));
}

#[test]
fn deserialize_api_response_error() {
    let json = r#"{"ok": false, "description": "Unauthorized"}"#;
    let resp: ApiResponse<User> = serde_json::from_str(json).unwrap();
    assert!(!resp.ok);
    assert!(resp.result.is_none());
    assert_eq!(resp.description.as_deref(), Some("Unauthorized"));
}

#[test]
fn deserialize_update_with_message() {
    let json = r#"{
        "update_id": 100,
        "message": {
            "message_id": 1,
            "chat": {"id": -100, "type": "group"},
            "date": 1700000000
        }
    }"#;
    let update: Update = serde_json::from_str(json).unwrap();
    assert_eq!(update.update_id, 100);
    assert!(update.message.is_some());
    assert!(update.channel_post.is_none());
}

#[test]
fn deserialize_update_with_channel_post() {
    let json = r#"{
        "update_id": 101,
        "channel_post": {
            "message_id": 5,
            "chat": {"id": -1001234, "title": "My Channel", "type": "channel"},
            "date": 1700000100
        }
    }"#;
    let update: Update = serde_json::from_str(json).unwrap();
    assert_eq!(update.update_id, 101);
    assert!(update.channel_post.is_some());
    let post = update.channel_post.unwrap();
    assert_eq!(post.chat.id, -1001234);
    assert_eq!(post.chat.title.as_deref(), Some("My Channel"));
}

#[test]
fn deserialize_message_with_document() {
    let json = r#"{
        "message_id": 10,
        "chat": {"id": -100, "type": "private"},
        "date": 1700000000,
        "document": {
            "file_id": "BQACAgIAAxkBAAI",
            "file_unique_id": "AgADBQAC",
            "file_name": "report.pdf",
            "mime_type": "application/pdf",
            "file_size": 102400
        },
        "caption": "Q4 Report"
    }"#;
    let msg: Message = serde_json::from_str(json).unwrap();
    assert!(msg.document.is_some());
    let doc = msg.document.unwrap();
    assert_eq!(doc.file_name.as_deref(), Some("report.pdf"));
    assert_eq!(doc.mime_type.as_deref(), Some("application/pdf"));
    assert_eq!(doc.file_size, Some(102400));
    assert_eq!(msg.caption.as_deref(), Some("Q4 Report"));
}

#[test]
fn deserialize_message_with_photo() {
    let json = r#"{
        "message_id": 11,
        "chat": {"id": -100, "type": "private"},
        "date": 1700000000,
        "photo": [
            {"file_id": "small", "file_unique_id": "su", "width": 90, "height": 90, "file_size": 1000},
            {"file_id": "large", "file_unique_id": "lu", "width": 1280, "height": 720, "file_size": 50000}
        ]
    }"#;
    let msg: Message = serde_json::from_str(json).unwrap();
    assert!(msg.photo.is_some());
    let photos = msg.photo.unwrap();
    assert_eq!(photos.len(), 2);
    assert_eq!(photos[1].file_id, "large");
}

#[test]
fn deserialize_message_with_video() {
    let json = r#"{
        "message_id": 12,
        "chat": {"id": -100, "type": "private"},
        "date": 1700000000,
        "video": {
            "file_id": "vid_1",
            "file_unique_id": "vid_u1",
            "file_name": "clip.mp4",
            "mime_type": "video/mp4",
            "file_size": 5000000,
            "duration": 30,
            "width": 1920,
            "height": 1080
        }
    }"#;
    let msg: Message = serde_json::from_str(json).unwrap();
    assert!(msg.video.is_some());
    let vid = msg.video.unwrap();
    assert_eq!(vid.duration, 30);
}

#[test]
fn deserialize_telegram_file() {
    let json = r#"{
        "file_id": "BQACAgIAAxkBAAI",
        "file_unique_id": "AgADBQAC",
        "file_size": 102400,
        "file_path": "documents/file_0.pdf"
    }"#;
    let file: TelegramFile = serde_json::from_str(json).unwrap();
    assert_eq!(file.file_path.as_deref(), Some("documents/file_0.pdf"));
    assert_eq!(file.file_size, Some(102400));
}

#[test]
fn deserialize_message_with_audio() {
    let json = r#"{
        "message_id": 13,
        "chat": {"id": -100, "type": "private"},
        "date": 1700000000,
        "audio": {
            "file_id": "aud_1",
            "file_unique_id": "aud_u1",
            "mime_type": "audio/mpeg",
            "file_size": 3000000,
            "duration": 200,
            "title": "Song Title",
            "performer": "Artist Name"
        }
    }"#;
    let msg: Message = serde_json::from_str(json).unwrap();
    let audio = msg.audio.unwrap();
    assert_eq!(audio.title.as_deref(), Some("Song Title"));
    assert_eq!(audio.performer.as_deref(), Some("Artist Name"));
}

#[test]
fn deserialize_empty_message() {
    let json = r#"{
        "message_id": 99,
        "chat": {"id": -100, "type": "private"},
        "date": 1700000000
    }"#;
    let msg: Message = serde_json::from_str(json).unwrap();
    assert!(msg.document.is_none());
    assert!(msg.photo.is_none());
    assert!(msg.video.is_none());
    assert!(msg.audio.is_none());
    assert!(msg.voice.is_none());
    assert!(msg.animation.is_none());
    assert!(msg.caption.is_none());
}
