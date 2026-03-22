use anyhow::Result;
use console::style;
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;

use super::client::TelegramClient;
use crate::files::filter::classify_message;
use crate::storage::db::Database;
use crate::ui::progress::{self, SpinnerType};

pub async fn watch(
    client: &TelegramClient,
    db: Arc<Database>,
    chat_id: i64,
    continuous: bool,
    duration: Option<u64>,
) -> Result<()> {
    let mut offset = db.get_offset()?;
    let mut total_indexed = 0u64;

    let spinner = progress::spinner(SpinnerType::Watch, "Waiting for files...");

    let deadline = duration.map(|d| tokio::time::Instant::now() + Duration::from_secs(d));

    loop {
        // Check deadline
        if let Some(dl) = deadline {
            if tokio::time::Instant::now() >= dl {
                spinner.finish_with_message(format!(
                    "Polling complete. {} files indexed.",
                    total_indexed
                ));
                return Ok(());
            }
        }

        // Poll with cancellation support
        let updates = tokio::select! {
            result = client.get_updates(offset, 30) => result?,
            _ = signal::ctrl_c() => {
                spinner.finish_with_message(format!(
                    "Stopped. {} files indexed.",
                    total_indexed
                ));
                return Ok(());
            }
        };

        for update in &updates {
            offset = Some(update.update_id + 1);

            let msg = update.message.as_ref().or(update.channel_post.as_ref());
            let Some(msg) = msg else { continue };

            if msg.chat.id != chat_id {
                continue;
            }

            let entries = classify_message(msg);
            for entry in entries {
                if db.insert_file(&entry)? {
                    total_indexed += 1;
                    spinner.set_message(format!(
                        "{} files indexed | latest: {}",
                        total_indexed,
                        style(
                            entry
                                .file_name
                                .as_deref()
                                .unwrap_or(entry.file_type.as_str())
                        )
                        .cyan()
                    ));
                }
            }
        }

        // Save offset
        if let Some(off) = offset {
            db.set_offset(off)?;
        }

        if !continuous && !updates.is_empty() && duration.is_none() {
            spinner.finish_with_message(format!("Done. {} files indexed.", total_indexed));
            return Ok(());
        }
    }
}
