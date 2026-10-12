use std::env;

/// Runtime configuration, read from environment variables (see `.env.example`).
#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub bind_addr: String,
    pub jwt_secret: String,
    pub jwt_ttl_minutes: i64,
    pub cors_origin: Option<String>,
    pub storage_dir: String,
    pub claude: ClaudeConfig,
    pub sms: SmsConfig,
}

/// SMS gateway (PM-17). `provider` is "log" (development: nothing is sent, messages
/// are only logged) or "bulksmsbd".
#[derive(Clone)]
pub struct SmsConfig {
    pub provider: String,
    pub api_key: Option<String>,
    pub sender_id: Option<String>,
    pub base_url: String,
}

// Never print the API key.
impl std::fmt::Debug for SmsConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SmsConfig")
            .field("provider", &self.provider)
            .field("api_key", &self.api_key.as_ref().map(|_| "***"))
            .field("sender_id", &self.sender_id)
            .field("base_url", &self.base_url)
            .finish()
    }
}

/// Claude API settings. Model names stay configurable (AI spec §8.2).
#[derive(Clone)]
pub struct ClaudeConfig {
    pub api_key: Option<String>,
    pub base_url: String,
    pub model_default: String,
    pub model_complex: String,
    pub model_fast: String,
}

// Never print the API key.
impl std::fmt::Debug for ClaudeConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClaudeConfig")
            .field("api_key", &self.api_key.as_ref().map(|_| "***"))
            .field("base_url", &self.base_url)
            .field("model_default", &self.model_default)
            .field("model_complex", &self.model_complex)
            .field("model_fast", &self.model_fast)
            .finish()
    }
}

#[derive(Debug, thiserror::Error)]
#[error("missing required environment variable {0}")]
pub struct ConfigError(&'static str);

fn required(key: &'static str) -> Result<String, ConfigError> {
    env::var(key).map_err(|_| ConfigError(key))
}

fn or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            database_url: required("DATABASE_URL")?,
            bind_addr: or("BIND_ADDR", "0.0.0.0:8080"),
            jwt_secret: required("JWT_SECRET")?,
            jwt_ttl_minutes: or("JWT_TTL_MINUTES", "720").parse().unwrap_or(720),
            cors_origin: env::var("CORS_ORIGIN").ok().filter(|s| !s.is_empty()),
            storage_dir: or("STORAGE_DIR", "./storage"),
            claude: ClaudeConfig {
                api_key: env::var("ANTHROPIC_API_KEY").ok().filter(|s| !s.is_empty()),
                base_url: or("ANTHROPIC_BASE_URL", "https://api.anthropic.com"),
                model_default: or("CLAUDE_MODEL_DEFAULT", "claude-sonnet-5-5"),
                model_complex: or("CLAUDE_MODEL_COMPLEX", "claude-opus-5-5"),
                model_fast: or("CLAUDE_MODEL_FAST", "claude-haiku-5-5"),
            },
            sms: SmsConfig {
                provider: or("SMS_PROVIDER", "log"),
                api_key: env::var("SMS_API_KEY").ok().filter(|s| !s.is_empty()),
                sender_id: env::var("SMS_SENDER_ID").ok().filter(|s| !s.is_empty()),
                base_url: or("SMS_BASE_URL", "https://bulksmsbd.net"),
            },
        })
    }
}
