use async_trait::async_trait;

pub struct RiskDecisionMessage {
    pub payload: Vec<u8>,
    pub received_ns: u64,
}

#[async_trait]
pub trait IRiskInputPort: Send + Sync {
    async fn recv(&self) -> Option<RiskDecisionMessage>;
}
