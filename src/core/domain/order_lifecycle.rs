use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum OrderSide { Buy, Sell }

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum OrderLifecycleEventType {
    Submitted, PartialFill, Filled, Rejected, Cancelled, Replaced, Expired, Error,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OrderLifecycleEvent {
    pub event_id: String,
    pub execution_id: String,
    pub client_order_id: Option<String>,
    pub symbol: String,
    pub event_type: OrderLifecycleEventType,
    pub timestamp: String,
    pub payload: serde_json::Value,
}
