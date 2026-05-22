use async_trait::async_trait;
use crate::core::domain::errors::ExecutionError;
use crate::adapters::messaging::wire_codec::{OrderCmd, CancelCmd, ReplaceCmd, StatusQuery};

pub type ExecutionId = String;

pub struct OrderStatusResponse {
    pub execution_id: String,
    pub status: String,
    pub symbol: String,
}

#[async_trait]
pub trait IGatewayClient: Send + Sync {
    async fn submit_order(&self, cmd: OrderCmd) -> Result<ExecutionId, ExecutionError>;
    async fn cancel_order(&self, cmd: CancelCmd) -> Result<(), ExecutionError>;
    async fn replace_order(&self, cmd: ReplaceCmd) -> Result<(), ExecutionError>;
    async fn query_status(&self, query: StatusQuery) -> Result<OrderStatusResponse, ExecutionError>;
}
