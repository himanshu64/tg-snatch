use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::sync::Mutex;

use crate::cli::FileTypeFilter;
use crate::files::catalog::FileEntry;
use crate::files::filter::FileType;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// Open an in-memory database (for testing)
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().context("Failed to open in-memory database")?;
        Self::init(conn)
    }

    pub fn open() -> Result<Self> {
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("tg-snatch");

        std::fs::create_dir_all(&data_dir)?;

        let db_path = data_dir.join("index.db");
        let conn = Connection::open(db_path).context("Failed to open database")?;
        Self::init(conn)
    }

    fn init(conn: Connection) -> Result<Self> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS files (
                file_unique_id TEXT PRIMARY KEY,
                file_id TEXT NOT NULL,
                file_type TEXT NOT NULL,
                file_name TEXT,
                file_size INTEGER,
                mime_type TEXT,
                chat_id INTEGER NOT NULL,
                message_id INTEGER NOT NULL,
                date INTEGER NOT NULL,
                caption TEXT,
                downloaded INTEGER DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS state (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );",
        )
        .context("Failed to create tables")?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Insert a file entry. Returns true if it was newly inserted.
    pub fn insert_file(&self, entry: &FileEntry) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let changed = conn.execute(
            "INSERT OR IGNORE INTO files
                (file_unique_id, file_id, file_type, file_name, file_size,
                 mime_type, chat_id, message_id, date, caption)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                entry.file_unique_id,
                entry.file_id,
                entry.file_type.as_str(),
                entry.file_name,
                entry.file_size.map(|s| s as i64),
                entry.mime_type,
                entry.chat_id,
                entry.message_id,
                entry.date,
                entry.caption,
            ],
        )?;
        Ok(changed > 0)
    }

    pub fn query_files(
        &self,
        chat_id: Option<i64>,
        file_type: Option<&FileTypeFilter>,
        limit: usize,
        only_not_downloaded: bool,
    ) -> Result<Vec<FileEntry>> {
        let conn = self.conn.lock().unwrap();

        let mut sql = String::from("SELECT * FROM files WHERE 1=1");
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(cid) = chat_id {
            sql.push_str(" AND chat_id = ?");
            param_values.push(Box::new(cid));
        }

        if let Some(ft) = file_type {
            match ft {
                FileTypeFilter::All => {}
                _ => {
                    let type_str = match ft {
                        FileTypeFilter::Pdf => "pdf",
                        FileTypeFilter::Image => "image",
                        FileTypeFilter::Video => "video",
                        FileTypeFilter::Audio => "audio",
                        FileTypeFilter::Document => "document",
                        FileTypeFilter::Animation => "animation",
                        FileTypeFilter::Voice => "voice",
                        FileTypeFilter::All => unreachable!(),
                    };
                    sql.push_str(" AND file_type = ?");
                    param_values.push(Box::new(type_str.to_string()));
                }
            }
        }

        if only_not_downloaded {
            sql.push_str(" AND downloaded = 0");
        }

        sql.push_str(" ORDER BY date DESC LIMIT ?");
        param_values.push(Box::new(limit as i64));

        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();

        let mut stmt = conn.prepare(&sql)?;
        let entries = stmt
            .query_map(params_refs.as_slice(), |row| {
                Ok(FileEntry {
                    file_unique_id: row.get("file_unique_id")?,
                    file_id: row.get("file_id")?,
                    file_type: FileType::from_db(&row.get::<_, String>("file_type")?)
                        .unwrap_or(FileType::Document),
                    file_name: row.get("file_name")?,
                    file_size: row.get::<_, Option<i64>>("file_size")?.map(|s| s as u64),
                    mime_type: row.get("mime_type")?,
                    chat_id: row.get("chat_id")?,
                    message_id: row.get("message_id")?,
                    date: row.get("date")?,
                    caption: row.get("caption")?,
                    downloaded: row.get::<_, i32>("downloaded")? != 0,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    pub fn mark_downloaded(&self, file_unique_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE files SET downloaded = 1 WHERE file_unique_id = ?",
            params![file_unique_id],
        )?;
        Ok(())
    }

    pub fn get_offset(&self) -> Result<Option<i64>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM state WHERE key = 'poll_offset'")?;
        let result = stmt.query_map([], |row| row.get::<_, String>(0))?.next();

        match result {
            Some(Ok(val)) => Ok(val.parse().ok()),
            _ => Ok(None),
        }
    }

    pub fn set_offset(&self, offset: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO state (key, value) VALUES ('poll_offset', ?)",
            params![offset.to_string()],
        )?;
        Ok(())
    }
}
