use async_trait::async_trait;

pub struct OrchestrationCommand {
    pub command: String,
    pub payload: Vec<u8>,
    pub received_ns: u64,
}

#[async_trait]
pub trait IOrchestrationReceiver: Send + Sync {
    async fn recv(&self) -> Option<OrchestrationCommand>;
}
