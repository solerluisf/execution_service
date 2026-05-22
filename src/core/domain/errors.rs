#[derive(Debug)]
pub enum ExecutionError {
    GatewayError(String),
    OrderNotFound(String),
    InvalidTransition(String, String),
    KillSwitchActive,
    OperationModeBlocked(String),
    IdempotencyConflict(String),
    SerializationError(String),
    MessagingError(String),
    ConfigError(String),
    JournalError(String),
}

impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionError::GatewayError(e) => write!(f, "Gateway error: {}", e),
            ExecutionError::OrderNotFound(id) => write!(f, "Order not found: {}", id),
            ExecutionError::InvalidTransition(from, to) => write!(f, "Invalid state transition: {} -> {}", from, to),
            ExecutionError::KillSwitchActive => write!(f, "Kill switch active"),
            ExecutionError::OperationModeBlocked(mode) => write!(f, "Operation mode blocked: {}", mode),
            ExecutionError::IdempotencyConflict(key) => write!(f, "Idempotency conflict: {}", key),
            ExecutionError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            ExecutionError::MessagingError(e) => write!(f, "Messaging error: {}", e),
            ExecutionError::ConfigError(e) => write!(f, "Configuration error: {}", e),
            ExecutionError::JournalError(e) => write!(f, "Journal error: {}", e),
        }
    }
}

impl std::error::Error for ExecutionError {}
