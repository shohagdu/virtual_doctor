//! Thin Claude Messages API client (AI spec §9.2). Called only from the backend;
//! the API key never reaches the browser and is never logged.

pub mod prompts;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use vd_common::config::ClaudeConfig;

const API_VERSION: &str = "2023-06-01";

#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("Claude API key is not configured")]
    NotConfigured,
    #[error("Claude API returned {status}: {message}")]
    Api { status: u16, message: String },
    #[error("the AI answer did not match the expected format")]
    BadOutput,
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
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub stream: bool,
}

#[derive(Debug, Default, Clone, Copy, Deserialize, Serialize)]
pub struct Usage {
    #[serde(default)]
    pub input_tokens: u32,
    #[serde(default)]
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

/// The structured answer of a forced tool call, with what it cost.
#[derive(Debug)]
pub struct ToolAnswer {
    pub input: Value,
    pub model: String,
    pub usage: Usage,
    pub latency_ms: u32,
}

/// Which configured model a task uses (AI spec §8.2).
#[derive(Clone, Copy, Debug)]
pub enum Tier {
    /// Extraction, clinical analysis, patient summary.
    Default,
    /// Many reports or comorbidities.
    Complex,
    /// Short helper tasks: search filters, SMS wording.
    Fast,
}

/// A system prompt plus the single tool Claude must answer through.
pub struct ToolCall<'a> {
    pub tier: Tier,
    pub max_tokens: u32,
    pub system: &'a str,
    pub tool_name: &'a str,
    pub tool_description: &'a str,
    pub schema: &'a Value,
    /// User message content blocks.
    pub content: Value,
}

#[derive(Clone)]
pub struct ClaudeClient {
    http: reqwest::Client,
    config: ClaudeConfig,
}

impl ClaudeClient {
    pub fn new(config: ClaudeConfig) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(180))
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

    pub fn model(&self, tier: Tier) -> &str {
        match tier {
            Tier::Default => &self.config.model_default,
            Tier::Complex => &self.config.model_complex,
            Tier::Fast => &self.config.model_fast,
        }
    }

    fn request<'a>(&'a self, call: &'a ToolCall<'_>, stream: bool) -> MessagesRequest<'a> {
        MessagesRequest {
            model: self.model(call.tier),
            max_tokens: call.max_tokens,
            // The long, fixed instructions are cached across calls (AI spec §8.10).
            system: Some(json!([{
                "type": "text",
                "text": call.system,
                "cache_control": { "type": "ephemeral" }
            }])),
            messages: vec![Message {
                role: "user",
                content: call.content.clone(),
            }],
            tools: Some(json!([{
                "name": call.tool_name,
                "description": call.tool_description,
                "input_schema": call.schema,
            }])),
            tool_choice: Some(json!({ "type": "tool", "name": call.tool_name })),
            stream,
        }
    }

    /// Sends a request, retrying 429 / 5xx with exponential backoff (AI spec §8.11).
    async fn send(&self, req: &MessagesRequest<'_>) -> Result<reqwest::Response, AiError> {
        let key = self
            .config
            .api_key
            .as_deref()
            .ok_or(AiError::NotConfigured)?;
        let url = format!("{}/v1/messages", self.config.base_url);
        let mut delay = Duration::from_millis(800);
        for attempt in 0..4 {
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
                return Ok(res);
            }
            let retryable = status.as_u16() == 429 || status.is_server_error();
            if retryable && attempt < 3 {
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

    /// Sends one Messages API request.
    pub async fn messages(&self, req: &MessagesRequest<'_>) -> Result<MessagesResponse, AiError> {
        Ok(self.send(req).await?.json().await?)
    }

    /// Runs a forced tool call and returns the tool input.
    pub async fn tool(&self, call: &ToolCall<'_>) -> Result<ToolAnswer, AiError> {
        let started = Instant::now();
        let res = self.messages(&self.request(call, false)).await?;
        let input = res
            .content
            .iter()
            .find(|b| b["type"] == "tool_use" && b["name"] == call.tool_name)
            .map(|b| b["input"].clone())
            .ok_or(AiError::BadOutput)?;
        Ok(ToolAnswer {
            input,
            model: res.model,
            usage: res.usage,
            latency_ms: started.elapsed().as_millis() as u32,
        })
    }

    /// Like `tool`, but streams the answer and reports how many characters of it
    /// have arrived, so the screen can show progress while Claude writes (AI spec §9.1).
    pub async fn tool_streamed(
        &self,
        call: &ToolCall<'_>,
        progress: mpsc::Sender<usize>,
    ) -> Result<ToolAnswer, AiError> {
        let started = Instant::now();
        let mut res = self.send(&self.request(call, true)).await?;
        let mut parser = StreamParser::default();
        while let Some(chunk) = res.chunk().await? {
            if parser.push(&chunk)? {
                let _ = progress.try_send(parser.json.len());
            }
        }
        let input: Value = if parser.json.trim().is_empty() {
            json!({})
        } else {
            serde_json::from_str(&parser.json).map_err(|_| AiError::BadOutput)?
        };
        Ok(ToolAnswer {
            input,
            model: parser.model.unwrap_or_else(|| self.model(call.tier).into()),
            usage: parser.usage,
            latency_ms: started.elapsed().as_millis() as u32,
        })
    }
}

/// Reassembles a streamed tool call from Claude's server-sent events.
#[derive(Default)]
struct StreamParser {
    /// Raw bytes not yet decoded; decoding only whole events keeps multi-byte
    /// (Bangla) characters that straddle two chunks intact.
    buf: Vec<u8>,
    json: String,
    model: Option<String>,
    usage: Usage,
}

impl StreamParser {
    /// Feeds raw bytes; returns whether more of the answer arrived.
    fn push(&mut self, bytes: &[u8]) -> Result<bool, AiError> {
        self.buf.extend_from_slice(bytes);
        let mut grew = false;
        while let Some(end) = self.buf.windows(2).position(|w| w == b"\n\n") {
            let raw: Vec<u8> = self.buf.drain(..end + 2).collect();
            let event = String::from_utf8_lossy(&raw);
            let data: String = event
                .lines()
                .filter_map(|l| l.strip_prefix("data:"))
                .map(str::trim_start)
                .collect();
            if data.is_empty() {
                continue;
            }
            let v: Value = serde_json::from_str(&data).map_err(|_| AiError::BadOutput)?;
            match v["type"].as_str().unwrap_or_default() {
                "message_start" => {
                    let m = &v["message"];
                    self.model = m["model"].as_str().map(Into::into);
                    if let Ok(u) = serde_json::from_value::<Usage>(m["usage"].clone()) {
                        self.usage = u;
                    }
                }
                "content_block_delta" if v["delta"]["type"] == "input_json_delta" => {
                    self.json
                        .push_str(v["delta"]["partial_json"].as_str().unwrap_or_default());
                    grew = true;
                }
                "message_delta" => {
                    if let Some(out) = v["usage"]["output_tokens"].as_u64() {
                        self.usage.output_tokens = out as u32;
                    }
                }
                "error" => {
                    return Err(AiError::Api {
                        status: 529,
                        message: v["error"]["message"]
                            .as_str()
                            .unwrap_or("stream error")
                            .into(),
                    });
                }
                _ => {}
            }
        }
        Ok(grew)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_streamed_tool_call_split_across_chunks() {
        let events = concat!(
            "event: message_start\ndata: {\"type\":\"message_start\",\"message\":{\"model\":\"m1\",\"usage\":{\"input_tokens\":120,\"cache_read_input_tokens\":100}}}\n\n",
            "event: content_block_start\ndata: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"tool_use\",\"name\":\"t\",\"input\":{}}}\n\n",
            "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"summary\\\": \\\"ভালো\"}}\n\n",
            "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"\\\"}\"}}\n\n",
            "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"\"}}\n\n",
            "event: message_delta\ndata: {\"type\":\"message_delta\",\"usage\":{\"output_tokens\":42}}\n\n",
            "event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n",
        );
        let mut p = StreamParser::default();
        let bytes = events.as_bytes();
        // Split in an awkward place to mimic network chunks.
        // Split in awkward places, including inside a Bangla character.
        let bn = events.find("ভ").unwrap() + 1;
        p.push(&bytes[..50]).unwrap();
        p.push(&bytes[50..bn]).unwrap();
        p.push(&bytes[bn..]).unwrap();
        let v: Value = serde_json::from_str(&p.json).unwrap();
        assert_eq!(v["summary"], "ভালো");
        assert_eq!(p.model.as_deref(), Some("m1"));
        assert_eq!(p.usage.input_tokens, 120);
        assert_eq!(p.usage.cache_read_input_tokens, 100);
        assert_eq!(p.usage.output_tokens, 42);
    }
}
