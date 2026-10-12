//! Thin Claude Messages API client (AI spec §9.2). Called only from the backend;
//! the API key never reaches the browser and is never logged.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use vd_common::config::ClaudeConfig;

const API_VERSION: &str = "2023-06-01";

#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("Claude API key is not configured")]
    NotConfigured,
    #[error("Claude API returned {status}: {message}")]
    Api { status: u16, message: String },
    #[error(transparent)]
    Http(#[from] reqwest::Error),
}

#[derive(Debug, Serialize)]
pub struct Message {
    pub role: &'static str,
    pub content: Value,
}

#[derive(Debug, Serialize)]
pub struct MessagesRequest<'a> {
    pub model: &'a str,
    pub max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<Value>,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    #[serde(default)]
    pub cache_read_input_tokens: u32,
}

#[derive(Debug, Deserialize)]
pub struct MessagesResponse {
    pub model: String,
    pub content: Vec<Value>,
    pub stop_reason: Option<String>,
    pub usage: Usage,
}

#[derive(Clone)]
pub struct ClaudeClient {
    http: reqwest::Client,
    config: ClaudeConfig,
}

impl ClaudeClient {
    pub fn new(config: ClaudeConfig) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("reqwest client");
        Self { http, config }
    }

    pub fn is_configured(&self) -> bool {
        self.config.api_key.is_some()
    }

    pub fn config(&self) -> &ClaudeConfig {
        &self.config
    }

    /// Sends one Messages API request. Retries on 429 / 5xx with exponential backoff.
    pub async fn messages(&self, req: &MessagesRequest<'_>) -> Result<MessagesResponse, AiError> {
        let key = self
            .config
            .api_key
            .as_deref()
            .ok_or(AiError::NotConfigured)?;
        let url = format!("{}/v1/messages", self.config.base_url);
        let mut delay = Duration::from_millis(500);

        for attempt in 0..3 {
            let res = self
                .http
                .post(&url)
                .header("x-api-key", key)
                .header("anthropic-version", API_VERSION)
                .json(req)
                .send()
                .await?;
            let status = res.status();
            if status.is_success() {
                return Ok(res.json().await?);
            }
            let retryable = status.as_u16() == 429 || status.is_server_error();
            if retryable && attempt < 2 {
                tracing::warn!(status = status.as_u16(), attempt, "Claude API retry");
                tokio::time::sleep(delay).await;
                delay *= 2;
                continue;
            }
            let body: Value = res.json().await.unwrap_or_default();
            return Err(AiError::Api {
                status: status.as_u16(),
                message: body["error"]["message"]
                    .as_str()
                    .unwrap_or("unknown error")
                    .to_string(),
            });
        }
        unreachable!("loop always returns")
    }
}
