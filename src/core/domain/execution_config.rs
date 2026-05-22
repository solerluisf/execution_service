use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    pub default_time_in_force: String,
    pub allow_extended_hours: bool,
    pub max_order_retries: u32,
    pub retry_backoff_ms: u64,
    pub order_staleness_ms: u64,
    pub idempotency_window_ms: u64,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            default_time_in_force: "day".to_string(),
            allow_extended_hours: false,
            max_order_retries: 3,
            retry_backoff_ms: 1000,
            order_staleness_ms: 30_000,
            idempotency_window_ms: 60_000,
        }
    }
}

impl ExecutionConfig {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file {}: {}", path, e))?;
        let config: ExecutionConfig = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config file {}: {}", path, e))?;
        Ok(config)
    }
}
