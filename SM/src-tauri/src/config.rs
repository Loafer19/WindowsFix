use std::time::Duration;

/// Application configuration loaded from environment variables with sensible defaults.
pub struct AppConfig {
    pub cache_ttl: Duration,
    pub ai_timeout: Duration,
    pub ai_max_tokens: u32,
    pub service_info_ttl: Duration,
}

impl AppConfig {
    pub fn from_env() -> Self {
        AppConfig {
            cache_ttl: Duration::from_secs(
                std::env::var("CACHE_TTL_SECS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(300),
            ),
            ai_timeout: Duration::from_secs(
                std::env::var("GROK_API_TIMEOUT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(15),
            ),
            ai_max_tokens: std::env::var("GROK_MAX_TOKENS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1000),
            service_info_ttl: Duration::from_secs(
                std::env::var("SERVICE_INFO_TTL_SECS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(86400 * 30), // 30 days
            ),
        }
    }
}
