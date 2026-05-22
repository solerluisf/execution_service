use async_trait::async_trait;

#[async_trait]
pub trait IJournalPort: Send + Sync {
    async fn append_order_state(&self, execution_id: &str, state: &str) -> Result<(), String>;
    async fn append_control_event(&self, event_type: &str, payload: &str) -> Result<(), String>;
    async fn get_latest_kill_switch_state(&self) -> Result<Option<bool>, String>;
    async fn get_latest_operation_mode(&self) -> Result<Option<String>, String>;
}
