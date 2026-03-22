use crate::files::catalog::FileEntry;
use crate::telegram::types::Message;
use chrono::NaiveDate;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    Pdf,
    Image,
    Video,
    Audio,
    Document,
    Animation,
    Voice,
}

impl FileType {
    pub fn as_str(&self) -> &'static str {
        match self {
            FileType::Pdf => "pdf",
            FileType::Image => "image",
            FileType::Video => "video",
            FileType::Audio => "audio",
            FileType::Document => "document",
            FileType::Animation => "animation",
            FileType::Voice => "voice",
        }
    }

    pub fn from_db(s: &str) -> Option<FileType> {
        match s {
            "pdf" => Some(FileType::Pdf),
            "image" => Some(FileType::Image),
            "video" => Some(FileType::Video),
            "audio" => Some(FileType::Audio),
            "document" => Some(FileType::Document),
            "animation" => Some(FileType::Animation),
            "voice" => Some(FileType::Voice),
            _ => None,
        }
    }

    pub fn subdir(&self) -> &'static str {
        match self {
            FileType::Pdf => "pdfs",
            FileType::Image => "images",
            FileType::Video => "videos",
            FileType::Audio => "audio",
            FileType::Document => "documents",
            FileType::Animation => "animations",
            FileType::Voice => "voice",
        }
    }
}

impl fmt::Display for FileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str().to_uppercase())
    }
}

pub fn classify_message(msg: &Message) -> Vec<FileEntry> {
    let mut entries = Vec::new();
    let chat_id = msg.chat.id;
    let message_id = msg.message_id;
    let date = msg.date;
    let caption = msg.caption.clone();

    // Document (includes PDFs)
    if let Some(doc) = &msg.document {
        let file_type = if doc
            .mime_type
            .as_deref()
            .is_some_and(|m| m == "application/pdf")
        {
            FileType::Pdf
        } else {
            FileType::Document
        };

        entries.push(FileEntry {
            file_id: doc.file_id.clone(),
            file_unique_id: doc.file_unique_id.clone(),
            file_type,
            file_name: doc.file_name.clone(),
            file_size: doc.file_size,
            mime_type: doc.mime_type.clone(),
            chat_id,
            message_id,
            date,
            caption: caption.clone(),
            downloaded: false,
        });
    }

    // Photo — pick the largest size
    if let Some(photos) = &msg.photo {
        if let Some(photo) = photos.last() {
            entries.push(FileEntry {
                file_id: photo.file_id.clone(),
                file_unique_id: photo.file_unique_id.clone(),
                file_type: FileType::Image,
                file_name: None,
                file_size: photo.file_size,
                mime_type: Some("image/jpeg".to_string()),
                chat_id,
                message_id,
                date,
                caption: caption.clone(),
                downloaded: false,
            });
        }
    }

    // Video
    if let Some(video) = &msg.video {
        entries.push(FileEntry {
            file_id: video.file_id.clone(),
            file_unique_id: video.file_unique_id.clone(),
            file_type: FileType::Video,
            file_name: video.file_name.clone(),
            file_size: video.file_size,
            mime_type: video.mime_type.clone(),
            chat_id,
            message_id,
            date,
            caption: caption.clone(),
            downloaded: false,
        });
    }

    // Audio
    if let Some(audio) = &msg.audio {
        let name = audio
            .file_name
            .clone()
            .or_else(|| audio.title.clone().map(|t| format!("{}.mp3", t)));
        entries.push(FileEntry {
            file_id: audio.file_id.clone(),
            file_unique_id: audio.file_unique_id.clone(),
            file_type: FileType::Audio,
            file_name: name,
            file_size: audio.file_size,
            mime_type: audio.mime_type.clone(),
            chat_id,
            message_id,
            date,
            caption: caption.clone(),
            downloaded: false,
        });
    }

    // Voice
    if let Some(voice) = &msg.voice {
        entries.push(FileEntry {
            file_id: voice.file_id.clone(),
            file_unique_id: voice.file_unique_id.clone(),
            file_type: FileType::Voice,
            file_name: None,
            file_size: voice.file_size,
            mime_type: voice.mime_type.clone(),
            chat_id,
            message_id,
            date,
            caption: caption.clone(),
            downloaded: false,
        });
    }

    // Animation (GIF)
    if let Some(anim) = &msg.animation {
        entries.push(FileEntry {
            file_id: anim.file_id.clone(),
            file_unique_id: anim.file_unique_id.clone(),
            file_type: FileType::Animation,
            file_name: anim.file_name.clone(),
            file_size: anim.file_size,
            mime_type: anim.mime_type.clone(),
            chat_id,
            message_id,
            date,
            caption,
            downloaded: false,
        });
    }

    entries
}

/// Parse a YYYY-MM-DD string to a Unix timestamp (start of day UTC).
pub fn parse_date_to_timestamp(date_str: &str) -> anyhow::Result<i64> {
    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|_| anyhow::anyhow!("Invalid date format '{}'. Use YYYY-MM-DD", date_str))?;
    let datetime = date
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| anyhow::anyhow!("Invalid date"))?;
    Ok(datetime.and_utc().timestamp())
}

/// Filter files by date range.
pub fn filter_by_date_range(
    files: Vec<FileEntry>,
    after: Option<&str>,
    before: Option<&str>,
) -> anyhow::Result<Vec<FileEntry>> {
    let after_ts = after.map(parse_date_to_timestamp).transpose()?;
    let before_ts = before
        .map(|d| {
            // "before" means before end of that day
            parse_date_to_timestamp(d).map(|ts| ts + 86400)
        })
        .transpose()?;

    Ok(files
        .into_iter()
        .filter(|f| {
            if let Some(after) = after_ts {
                if f.date < after {
                    return false;
                }
            }
            if let Some(before) = before_ts {
                if f.date >= before {
                    return false;
                }
            }
            true
        })
        .collect())
}
