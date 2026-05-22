use serde::{Deserialize, Serialize};
use crate::core::domain::operation_mode::OperationMode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub risk_router_endpoint: String,
    pub gateway_router_endpoint: String,
    pub gateway_health_endpoint: String,
    pub lifecycle_pub_endpoint: String,
    pub control_endpoint: String,
    pub ack_endpoint: String,
    pub operation_mode: OperationMode,
    pub symbols: Vec<String>,
    pub execution_config_path: String,
    pub journal_db_path: String,
    pub health_port: u16,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, String> {
        dotenvy::dotenv().ok();

        let risk_router_endpoint = std::env::var("RISK_ROUTER_ENDPOINT")
            .unwrap_or_else(|_| "tcp://127.0.0.1:5559".to_string());
        let gateway_router_endpoint = std::env::var("GATEWAY_ROUTER_ENDPOINT")
            .unwrap_or_else(|_| "tcp://127.0.0.1:5555".to_string());
        let gateway_health_endpoint = std::env::var("GATEWAY_HEALTH_ENDPOINT")
            .unwrap_or_else(|_| "tcp://127.0.0.1:5562".to_string());
        let lifecycle_pub_endpoint = std::env::var("LIFECYCLE_PUB_ENDPOINT")
            .unwrap_or_else(|_| "tcp://127.0.0.1:5570".to_string());
        let control_endpoint = std::env::var("CONTROL_ENDPOINT")
            .unwrap_or_else(|_| "tcp://127.0.0.1:5571".to_string());
        let ack_endpoint = std::env::var("ACK_ENDPOINT")
            .unwrap_or_else(|_| "tcp://127.0.0.1:5572".to_string());
        let operation_mode = std::env::var("OPERATION_MODE")
            .ok()
            .and_then(|s| s.parse::<OperationMode>().ok())
            .unwrap_or(OperationMode::Live);
        let symbols = std::env::var("SYMBOLS")
            .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_else(|_| vec!["AAPL".to_string()]);
        let execution_config_path = std::env::var("EXECUTION_CONFIG_PATH")
            .unwrap_or_else(|_| "./execution.toml".to_string());
        let journal_db_path = std::env::var("JOURNAL_DB_PATH")
            .unwrap_or_else(|_| "execution_journal.db".to_string());
        let health_port = std::env::var("HEALTH_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(9096);

        Ok(Self {
            risk_router_endpoint,
            gateway_router_endpoint,
            gateway_health_endpoint,
            lifecycle_pub_endpoint,
            control_endpoint,
            ack_endpoint,
            operation_mode,
            symbols,
            execution_config_path,
            journal_db_path,
            health_port,
        })
    }
}
