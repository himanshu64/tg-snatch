use anyhow::{bail, Context, Result};
use reqwest::Client;
use std::time::Duration;

use super::types::{ApiResponse, TelegramFile, Update, User};
use crate::security;

pub struct TelegramClient {
    client: Client,
    token: String,
}

impl TelegramClient {
    pub fn new(token: String) -> Self {
        // Build a client with sensible timeouts and TLS enforcement
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(30))
            .timeout(Duration::from_secs(120))
            .https_only(true) // Refuse HTTP
            .build()
            .unwrap_or_default();

        Self { client, token }
    }

    fn api_url(&self, method: &str) -> String {
        format!("https://api.telegram.org/bot{}/{}", self.token, method)
    }

    pub fn file_url(&self, file_path: &str) -> String {
        format!(
            "https://api.telegram.org/file/bot{}/{}",
            self.token, file_path
        )
    }

    /// Masked token for display in logs/errors — never leak the full token.
    fn masked_token(&self) -> String {
        security::mask_token(&self.token)
    }

    pub async fn get_me(&self) -> Result<User> {
        let resp: ApiResponse<User> = self
            .client
            .get(self.api_url("getMe"))
            .send()
            .await
            .with_context(|| {
                format!(
                    "Failed to connect to Telegram (token: {})",
                    self.masked_token()
                )
            })?
            .json()
            .await?;

        match resp.result {
            Some(user) if resp.ok => Ok(user),
            _ => bail!(
                "Failed to authenticate bot (token: {}): {}",
                self.masked_token(),
                resp.description.unwrap_or_default()
            ),
        }
    }

    pub async fn get_updates(&self, offset: Option<i64>, timeout: u64) -> Result<Vec<Update>> {
        let mut params = vec![
            ("timeout".to_string(), timeout.to_string()),
            (
                "allowed_updates".to_string(),
                "[\"message\",\"channel_post\"]".to_string(),
            ),
        ];
        if let Some(off) = offset {
            params.push(("offset".to_string(), off.to_string()));
        }

        // Use a longer timeout for long-polling
        let long_poll_client = Client::builder()
            .connect_timeout(Duration::from_secs(30))
            .timeout(Duration::from_secs(timeout + 10))
            .https_only(true)
            .build()
            .unwrap_or_default();

        let resp: ApiResponse<Vec<Update>> = long_poll_client
            .get(self.api_url("getUpdates"))
            .query(&params)
            .send()
            .await
            .context("Failed to poll updates")?
            .json()
            .await
            .context("Failed to parse updates response")?;

        match resp.result {
            Some(updates) if resp.ok => Ok(updates),
            _ => bail!(
                "Failed to get updates: {}",
                resp.description.unwrap_or_default()
            ),
        }
    }

    pub async fn get_file(&self, file_id: &str) -> Result<TelegramFile> {
        let resp: ApiResponse<TelegramFile> = self
            .client
            .get(self.api_url("getFile"))
            .query(&[("file_id", file_id)])
            .send()
            .await?
            .json()
            .await?;

        match resp.result {
            Some(file) if resp.ok => Ok(file),
            _ => bail!(
                "Failed to get file: {}",
                resp.description.unwrap_or_default()
            ),
        }
    }
}
