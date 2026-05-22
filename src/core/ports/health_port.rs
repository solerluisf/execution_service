use async_trait::async_trait;

#[async_trait]
pub trait IHealthReporter: Send + Sync {
    fn is_healthy(&self) -> bool;
    fn get_status(&self) -> String;
}
