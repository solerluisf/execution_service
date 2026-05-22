#[cfg(test)]
mod correlation_chain_integration {
    use execution_service::core::domain::risk_decision::{RiskDecision, ApprovedSize};
    use execution_service::core::application::order_tracker::OrderTracker;
    use execution_service::core::application::idempotency::IdempotencyStore;
    use execution_service::core::application::kill_switch::KillSwitch;
    use execution_service::core::application::mode_controller::ModeController;
    use execution_service::core::domain::operation_mode::OperationMode;
    use execution_service::core::domain::execution_config::ExecutionConfig;
    use execution_service::adapters::metrics::metrics_adapter::MetricsAdapter;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn make_test_decision(request_id: &str, intent_id: &str) -> RiskDecision {
        RiskDecision {
            request_id: request_id.to_string(),
            intent_id: intent_id.to_string(),
            symbol: "AAPL".to_string(),
            approved: true,
            rejection_reason: None,
            approved_size: Some(ApprovedSize::Units(100.0)),
            risk_score: 0.5,
            timestamp_ns: 0,
            latency_us: 0,
        }
    }

    #[tokio::test]
    async fn process_decision_propagates_correlation_id() {
        let config = Arc::new(RwLock::new(ExecutionConfig::default()));
        let tracker = Arc::new(OrderTracker::new());
        let idempotency = Arc::new(IdempotencyStore::new());
        let kill_switch = Arc::new(KillSwitch::new());
        let mode_controller = Arc::new(ModeController::new(OperationMode::Live));
        let metrics = Arc::new(MetricsAdapter::new());

        let engine = execution_service::core::application::execution_engine::ExecutionEngine::new(
            config,
            tracker,
            idempotency,
            kill_switch,
            mode_controller,
            metrics,
        );

        let decision = make_test_decision("test-corr-001", "test-intent-001");
        let now_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        let result = engine.process_decision(&decision, now_ns);
        assert!(result.is_ok());
        let order_cmd = result.unwrap().unwrap();

        // Verify correlation_id is propagated
        assert_eq!(order_cmd.correlation_id, Some("test-corr-001".to_string()));
        assert_eq!(order_cmd.client_order_id, Some("exec-test-intent-001".to_string()));
    }
}
