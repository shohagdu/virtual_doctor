//! SMS gateways (PM-17). The gateway is chosen by `SMS_PROVIDER`.

use std::time::Duration;

use serde::Deserialize;
use vd_common::config::SmsConfig;

pub enum Gateway {
    /// Development: nothing leaves the server; the send is only logged.
    Log,
    BulkSmsBd {
        client: reqwest::Client,
        url: String,
        api_key: String,
        sender_id: String,
    },
}

impl Gateway {
    pub fn from_config(c: &SmsConfig) -> Self {
        match c.provider.as_str() {
            "bulksmsbd" => match (&c.api_key, &c.sender_id) {
                (Some(api_key), Some(sender_id)) => Self::BulkSmsBd {
                    client: reqwest::Client::builder()
                        .timeout(Duration::from_secs(20))
                        .build()
                        .unwrap_or_default(),
                    url: format!("{}/api/smsapi", c.base_url.trim_end_matches('/')),
                    api_key: api_key.clone(),
                    sender_id: sender_id.clone(),
                },
                _ => {
                    tracing::error!(
                        "SMS_PROVIDER=bulksmsbd needs SMS_API_KEY and SMS_SENDER_ID; SMS will only be logged"
                    );
                    Self::Log
                }
            },
            "log" => Self::Log,
            other => {
                tracing::error!(
                    provider = other,
                    "unknown SMS_PROVIDER; SMS will only be logged"
                );
                Self::Log
            }
        }
    }

    /// Sends one message to `to` (8801XXXXXXXXX). Returns the gateway's reference.
    pub async fn send(&self, to: &str, body: &str) -> Result<String, String> {
        match self {
            Self::Log => {
                // The text is not logged: it carries the patient's name.
                let masked = format!(
                    "{}*****{}",
                    &to[..to.len().min(5)],
                    &to[to.len().saturating_sub(3)..]
                );
                tracing::info!(to = %masked, chars = body.chars().count(), "SMS (log gateway, not sent)");
                Ok("log".into())
            }
            Self::BulkSmsBd {
                client,
                url,
                api_key,
                sender_id,
            } => {
                #[derive(Deserialize)]
                struct Reply {
                    response_code: i64,
                    #[serde(default)]
                    message_id: Option<serde_json::Value>,
                    #[serde(default)]
                    error_message: Option<String>,
                }
                let res = client
                    .post(url)
                    .form(&[
                        ("api_key", api_key.as_str()),
                        ("type", "text"),
                        ("number", to),
                        ("senderid", sender_id.as_str()),
                        ("message", body),
                    ])
                    .send()
                    .await
                    .map_err(|e| format!("gateway unreachable: {e}"))?;
                let status = res.status();
                let reply: Reply = res
                    .json()
                    .await
                    .map_err(|e| format!("gateway reply unreadable (HTTP {status}): {e}"))?;
                // 202 = "SMS submitted successfully".
                if reply.response_code == 202 {
                    Ok(reply
                        .message_id
                        .map(|v| v.to_string().trim_matches('"').to_string())
                        .unwrap_or_default())
                } else {
                    Err(format!(
                        "gateway error {}: {}",
                        reply.response_code,
                        reply.error_message.unwrap_or_default()
                    ))
                }
            }
        }
    }
}
