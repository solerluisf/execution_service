use std::collections::HashMap;
use std::sync::RwLock;
use crate::core::domain::order_state::{OrderState, OrderStatus, OrderStateMachine};
use crate::core::domain::order_lifecycle::OrderLifecycleEvent;
use crate::core::domain::errors::ExecutionError;

pub struct OrderTracker {
    orders: RwLock<HashMap<String, OrderState>>,
}

impl OrderTracker {
    pub fn new() -> Self {
        Self {
            orders: RwLock::new(HashMap::new()),
        }
    }

    pub fn register(&self, state: OrderState) {
        let mut orders = self.orders.write().unwrap();
        orders.insert(state.execution_id.clone(), state);
    }

    pub fn update(&self, execution_id: &str, event: &OrderLifecycleEvent) -> Result<(), ExecutionError> {
        let mut orders = self.orders.write().unwrap();
        if let Some(state) = orders.get(execution_id) {
            let new_state = OrderStateMachine::transition(state, event)?;
            orders.insert(execution_id.to_string(), new_state);
            Ok(())
        } else {
            Err(ExecutionError::OrderNotFound(execution_id.to_string()))
        }
    }

    pub fn get(&self, execution_id: &str) -> Option<OrderState> {
        let orders = self.orders.read().unwrap();
        orders.get(execution_id).cloned()
    }

    pub fn get_all(&self) -> Vec<OrderState> {
        let orders = self.orders.read().unwrap();
        orders.values().cloned().collect()
    }

    pub fn count_by_status(&self) -> HashMap<OrderStatus, usize> {
        let orders = self.orders.read().unwrap();
        let mut counts = HashMap::new();
        for state in orders.values() {
            *counts.entry(state.status.clone()).or_insert(0) += 1;
        }
        counts
    }
}

impl Default for OrderTracker {
    fn default() -> Self {
        Self::new()
    }
}
