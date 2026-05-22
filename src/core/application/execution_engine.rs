use std::sync::Arc;
use tokio::sync::RwLock;
use crate::core::domain::risk_decision::RiskDecision;
use crate::core::domain::execution_config::ExecutionConfig;
use crate::core::domain::order_lifecycle::OrderLifecycleEvent;
use crate::core::domain::errors::ExecutionError;
use crate::core::domain::order_state::OrderState;
use crate::core::application::order_tracker::OrderTracker;
use crate::core::application::idempotency::IdempotencyStore;
use crate::core::application::kill_switch::KillSwitch;
use crate::core::application::mode_controller::ModeController;
use crate::core::ports::metrics_port::IMetricsPort;
use crate::adapters::messaging::wire_codec::OrderCmd;

pub struct ExecutionEngine {
    pub config: Arc<RwLock<ExecutionConfig>>,
    pub tracker: Arc<OrderTracker>,
    pub idempotency: Arc<IdempotencyStore>,
    pub kill_switch: Arc<KillSwitch>,
    pub mode_controller: Arc<ModeController>,
    pub metrics: Arc<dyn IMetricsPort>,
}

impl ExecutionEngine {
    pub fn new(
        config: Arc<RwLock<ExecutionConfig>>,
        tracker: Arc<OrderTracker>,
        idempotency: Arc<IdempotencyStore>,
        kill_switch: Arc<KillSwitch>,
        mode_controller: Arc<ModeController>,
        metrics: Arc<dyn IMetricsPort>,
    ) -> Self {
        Self {
            config,
            tracker,
            idempotency,
            kill_switch,
            mode_controller,
            metrics,
        }
    }

    pub fn process_decision(&self, decision: &RiskDecision, now_ns: u64) -> Result<Option<OrderCmd>, ExecutionError> {
        // 1. Check kill switch
        if self.kill_switch.is_active() {
            tracing::warn!("Kill switch active, rejecting decision {}", decision.intent_id);
            return Ok(None);
        }

        // 2. Check operation mode
        let mode = self.mode_controller.get();
        if !mode.allows_trading() {
            tracing::warn!("Operation mode {:?} does not allow trading", mode);
            return Ok(None);
        }

        // 3. Check if approved
        if !decision.approved {
            tracing::info!("Decision {} not approved, reason: {:?}", decision.intent_id, decision.rejection_reason);
            self.metrics.increment_counter("execution_decisions_received_total", &[
                ("symbol", &decision.symbol),
                ("approved", "false"),
            ]);
            return Ok(None);
        }

        // 4. Check idempotency
        if self.idempotency.is_processed(&decision.intent_id) {
            tracing::warn!("Duplicate intent_id: {}", decision.intent_id);
            return Ok(None);
        }

        // 5. Resolve size
        let size = match &decision.approved_size {
            Some(crate::core::domain::risk_decision::ApprovedSize::Units(qty)) => *qty,
            Some(crate::core::domain::risk_decision::ApprovedSize::Notional(_)) => {
                // For now, treat notional as units (would need price to convert)
                1.0
            }
            None => 1.0,
        };

        // 6. Build OrderCmd
        let config = self.config.try_read().map_err(|_| {
            ExecutionError::ConfigError("Failed to read execution config".to_string())
        })?;

        let execution_id = format!("exec-{}", decision.intent_id);
        let order_cmd = OrderCmd {
            correlation_id: Some(decision.request_id.clone()),
            client_order_id: Some(execution_id.clone()),
            symbol: decision.symbol.clone(),
            qty: size,
            side: "Buy".to_string(), // TODO: derive from intent
            time_in_force: config.default_time_in_force.clone(),
            extended_hours: config.allow_extended_hours,
        };

        // 7. Mark idempotent
        self.idempotency.mark_processed(decision.intent_id.clone(), execution_id.clone());

        // 8. Register order state
        let order_state = OrderState::pending(
            &execution_id,
            &decision.intent_id,
            &decision.request_id,
            &decision.symbol,
            now_ns,
        );
        self.tracker.register(order_state);

        // 9. Update metrics
        self.metrics.increment_counter("execution_decisions_received_total", &[
            ("symbol", &decision.symbol),
            ("approved", "true"),
        ]);
        self.metrics.increment_counter("execution_orders_submitted_total", &[
            ("symbol", &decision.symbol),
        ]);

        Ok(Some(order_cmd))
    }

    pub fn handle_gateway_response(&self, correlation_id: &str, result: &Result<String, String>) {
        match result {
            Ok(execution_id) => {
                tracing::info!("Gateway ACK for correlation_id={}, execution_id={}", correlation_id, execution_id);
                self.metrics.increment_counter("execution_correlation_matches_total", &[]);
            }
            Err(err) => {
                tracing::error!("Gateway error for correlation_id={}: {}", correlation_id, err);
                self.metrics.increment_counter("execution_correlation_mismatches_total", &[]);
            }
        }
    }

    pub fn handle_lifecycle_event(&self, event: &OrderLifecycleEvent) {
        tracing::info!("Lifecycle event: {:?} for execution_id={}", event.event_type, event.execution_id);
        if let Err(e) = self.tracker.update(&event.execution_id, event) {
            tracing::error!("Failed to update order state: {}", e);
        }

        match event.event_type {
            crate::core::domain::order_lifecycle::OrderLifecycleEventType::Filled => {
                self.metrics.increment_counter("execution_orders_filled_total", &[
                    ("symbol", &event.symbol),
                ]);
            }
            crate::core::domain::order_lifecycle::OrderLifecycleEventType::Cancelled => {
                self.metrics.increment_counter("execution_orders_cancelled_total", &[
                    ("symbol", &event.symbol),
                ]);
            }
            crate::core::domain::order_lifecycle::OrderLifecycleEventType::Rejected => {
                self.metrics.increment_counter("execution_orders_rejected_total", &[
                    ("symbol", &event.symbol),
                ]);
            }
            _ => {}
        }
    }
}
