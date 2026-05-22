use serde::{Deserialize, Serialize};
use crate::core::domain::errors::ExecutionError;
use crate::core::domain::order_lifecycle::OrderLifecycleEvent;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    Submitted,
    PartialFill { filled_qty: f64 },
    Filled,
    Cancelled,
    Rejected,
    Expired,
}

impl std::hash::Hash for OrderStatus {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
    }
}

impl Eq for OrderStatus {}

impl std::fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderStatus::Pending => write!(f, "Pending"),
            OrderStatus::Submitted => write!(f, "Submitted"),
            OrderStatus::PartialFill { .. } => write!(f, "PartialFill"),
            OrderStatus::Filled => write!(f, "Filled"),
            OrderStatus::Cancelled => write!(f, "Cancelled"),
            OrderStatus::Rejected => write!(f, "Rejected"),
            OrderStatus::Expired => write!(f, "Expired"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderState {
    pub execution_id: String,
    pub intent_id: String,
    pub correlation_id: String,
    pub symbol: String,
    pub status: OrderStatus,
    pub submitted_ns: u64,
    pub last_updated_ns: u64,
}

impl OrderState {
    pub fn pending(execution_id: &str, intent_id: &str, correlation_id: &str, symbol: &str, now_ns: u64) -> Self {
        Self {
            execution_id: execution_id.to_string(),
            intent_id: intent_id.to_string(),
            correlation_id: correlation_id.to_string(),
            symbol: symbol.to_string(),
            status: OrderStatus::Pending,
            submitted_ns: now_ns,
            last_updated_ns: now_ns,
        }
    }
}

pub struct OrderStateMachine;

impl OrderStateMachine {
    pub fn transition(state: &OrderState, event: &OrderLifecycleEvent) -> Result<OrderState, ExecutionError> {
        let new_status = match (&state.status, &event.event_type) {
            (OrderStatus::Pending, crate::core::domain::order_lifecycle::OrderLifecycleEventType::Submitted) => {
                OrderStatus::Submitted
            }
            (OrderStatus::Submitted, crate::core::domain::order_lifecycle::OrderLifecycleEventType::Filled) => {
                OrderStatus::Filled
            }
            (OrderStatus::Submitted, crate::core::domain::order_lifecycle::OrderLifecycleEventType::PartialFill) => {
                let filled_qty = event.payload.get("filled_qty")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                OrderStatus::PartialFill { filled_qty }
            }
            (OrderStatus::PartialFill { .. }, crate::core::domain::order_lifecycle::OrderLifecycleEventType::Filled) => {
                OrderStatus::Filled
            }
            (OrderStatus::Submitted, crate::core::domain::order_lifecycle::OrderLifecycleEventType::Cancelled) => {
                OrderStatus::Cancelled
            }
            (OrderStatus::Submitted, crate::core::domain::order_lifecycle::OrderLifecycleEventType::Rejected) => {
                OrderStatus::Rejected
            }
            (OrderStatus::Submitted, crate::core::domain::order_lifecycle::OrderLifecycleEventType::Expired) => {
                OrderStatus::Expired
            }
            _ => {
                return Err(ExecutionError::InvalidTransition(
                    state.status.to_string(),
                    format!("{:?}", event.event_type),
                ));
            }
        };

        Ok(OrderState {
            status: new_status,
            last_updated_ns: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
            ..state.clone()
        })
    }
}
