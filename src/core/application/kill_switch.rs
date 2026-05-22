use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{self, Receiver, Sender};
use crate::core::infrastructure::mutex_ext::MutexExt;

pub type ExecutionId = String;

type CancelCallback = Arc<dyn Fn(ExecutionId) + Send + Sync>;

#[derive(Clone)]
pub struct CancelChannel(Sender<ExecutionId>);

#[derive(Clone)]
pub struct KillSwitch {
    inner: Arc<Mutex<KillSwitchInner>>,
}

struct KillSwitchInner {
    enabled: bool,
    open_orders: HashSet<String>,
    cancel_callback: Option<CancelCallback>,
    cancel_channel: Option<CancelChannel>,
}

impl Default for KillSwitch {
    fn default() -> Self {
        Self::new()
    }
}

impl CancelChannel {
    pub fn send(&self, exec_id: ExecutionId) {
        if let Err(e) = self.0.try_send(exec_id) {
            tracing::error!("Failed to send cancel command: {}", e);
        }
    }
}

impl KillSwitch {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(KillSwitchInner {
                enabled: false,
                open_orders: HashSet::new(),
                cancel_callback: None,
                cancel_channel: None,
            })),
        }
    }

    pub fn with_channel() -> (Self, Receiver<ExecutionId>) {
        let (tx, rx) = mpsc::channel::<ExecutionId>(100);
        let kill_switch = Self {
            inner: Arc::new(Mutex::new(KillSwitchInner {
                enabled: false,
                open_orders: HashSet::new(),
                cancel_callback: None,
                cancel_channel: Some(CancelChannel(tx)),
            })),
        };
        (kill_switch, rx)
    }

    pub fn is_active(&self) -> bool {
        self.inner.safe_lock().enabled
    }

    pub fn enable(&self) -> usize {
        let (order_count, orders_to_cancel, cancel_callback, cancel_channel) = {
            let mut inner = self.inner.safe_lock();

            if inner.enabled {
                tracing::warn!("Kill switch already enabled, {} open orders remain", inner.open_orders.len());
                return inner.open_orders.len();
            }

            inner.enabled = true;
            let order_count = inner.open_orders.len();
            let orders_to_cancel: Vec<String> = inner.open_orders.iter().cloned().collect();
            let cancel_callback = inner.cancel_callback.clone();
            let cancel_channel = inner.cancel_channel.clone();

            (order_count, orders_to_cancel, cancel_callback, cancel_channel)
        };

        if order_count > 0 {
            tracing::error!("KILL SWITCH ACTIVATED - Cancelling {} open orders", order_count);

            if let Some(channel) = cancel_channel {
                for execution_id in &orders_to_cancel {
                    tracing::error!("Kill switch sending cancel for order: {}", execution_id);
                    channel.send(execution_id.clone());
                }
            } else if let Some(callback) = cancel_callback {
                for execution_id in &orders_to_cancel {
                    tracing::error!("Kill switch cancelling order: {}", execution_id);
                    callback(execution_id.clone());
                }
            } else {
                tracing::warn!("Kill switch has no cancel mechanism registered");
            }
        } else {
            tracing::error!("KILL SWITCH ACTIVATED - No open orders to cancel");
        }

        order_count
    }

    pub fn disable(&self) {
        let mut inner = self.inner.safe_lock();
        inner.enabled = false;
        tracing::info!("Kill switch deactivated - new orders will be accepted");
    }

    pub fn register_cancel_callback<F>(&self, callback: F)
    where
        F: Fn(ExecutionId) + Send + Sync + 'static,
    {
        let mut inner = self.inner.safe_lock();
        inner.cancel_callback = Some(Arc::new(callback));
        tracing::debug!("Kill switch cancel callback registered");
    }

    pub fn register_cancel_channel(&self) -> Receiver<ExecutionId> {
        let (tx, rx) = mpsc::channel::<ExecutionId>(100);
        let mut inner = self.inner.safe_lock();
        inner.cancel_channel = Some(CancelChannel(tx));
        tracing::debug!("Kill switch cancel channel registered");
        rx
    }

    pub fn track_open_order(&self, execution_id: &ExecutionId) -> bool {
        let mut inner = self.inner.safe_lock();
        let added = inner.open_orders.insert(execution_id.clone());
        if added {
            tracing::debug!("Tracking open order: {}", execution_id);
        }
        added
    }

    pub fn remove_open_order(&self, execution_id: &ExecutionId) -> bool {
        let mut inner = self.inner.safe_lock();
        let removed = inner.open_orders.remove(execution_id);
        if removed {
            tracing::debug!("Order no longer tracked: {}", execution_id);
        }
        removed
    }

    pub fn open_order_count(&self) -> usize {
        self.inner.safe_lock().open_orders.len()
    }

    pub fn get_open_orders(&self) -> Vec<ExecutionId> {
        self.inner.safe_lock()
            .open_orders
            .iter()
            .cloned()
            .collect()
    }

    pub fn is_order_tracked(&self, execution_id: &ExecutionId) -> bool {
        self.inner.safe_lock().open_orders.contains(execution_id)
    }
}
