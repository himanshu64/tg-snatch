use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub ok: bool,
    pub result: Option<T>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct User {
    pub id: i64,
    pub is_bot: bool,
    pub first_name: String,
    pub username: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Chat {
    pub id: i64,
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub chat_type: String,
}

#[derive(Debug, Deserialize)]
pub struct Update {
    pub update_id: i64,
    pub message: Option<Message>,
    pub channel_post: Option<Message>,
}

#[derive(Debug, Deserialize)]
pub struct Message {
    pub message_id: i64,
    pub chat: Chat,
    pub date: i64,
    pub caption: Option<String>,
    pub document: Option<Document>,
    pub photo: Option<Vec<PhotoSize>>,
    pub video: Option<Video>,
    pub audio: Option<Audio>,
    pub voice: Option<Voice>,
    pub animation: Option<Animation>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Document {
    pub file_id: String,
    pub file_unique_id: String,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<u64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PhotoSize {
    pub file_id: String,
    pub file_unique_id: String,
    pub width: i32,
    pub height: i32,
    pub file_size: Option<u64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Video {
    pub file_id: String,
    pub file_unique_id: String,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<u64>,
    pub duration: i32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Audio {
    pub file_id: String,
    pub file_unique_id: String,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<u64>,
    pub duration: i32,
    pub title: Option<String>,
    pub performer: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Voice {
    pub file_id: String,
    pub file_unique_id: String,
    pub mime_type: Option<String>,
    pub file_size: Option<u64>,
    pub duration: i32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Animation {
    pub file_id: String,
    pub file_unique_id: String,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct TelegramFile {
    pub file_id: String,
    pub file_unique_id: String,
    pub file_size: Option<u64>,
    pub file_path: Option<String>,
}
