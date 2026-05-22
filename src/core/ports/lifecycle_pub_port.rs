use async_trait::async_trait;
use crate::core::domain::order_lifecycle::OrderLifecycleEvent;

#[async_trait]
pub trait IOrderLifecyclePublisher: Send + Sync {
    async fn publish(&self, event: &OrderLifecycleEvent);
}
