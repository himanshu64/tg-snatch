use crate::files::filter::FileType;
use crate::security;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub file_id: String,
    pub file_unique_id: String,
    pub file_type: FileType,
    pub file_name: Option<String>,
    pub file_size: Option<u64>,
    pub mime_type: Option<String>,
    pub chat_id: i64,
    pub message_id: i64,
    pub date: i64,
    pub caption: Option<String>,
    pub downloaded: bool,
}

impl FileEntry {
    pub fn display_name(&self) -> String {
        let raw = self
            .file_name
            .clone()
            .unwrap_or_else(|| format!("{}.{}", self.file_unique_id, self.default_ext()));
        security::sanitize_filename(&raw)
    }

    pub fn display_size(&self) -> String {
        match self.file_size {
            Some(size) => format_size(size),
            None => "unknown".to_string(),
        }
    }

    fn default_ext(&self) -> &str {
        match self.file_type {
            FileType::Pdf => "pdf",
            FileType::Image => "jpg",
            FileType::Video => "mp4",
            FileType::Audio => "mp3",
            FileType::Document => "bin",
            FileType::Animation => "gif",
            FileType::Voice => "ogg",
        }
    }
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
