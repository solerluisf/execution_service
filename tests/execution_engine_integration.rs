#[cfg(test)]
mod execution_engine_integration {
    use execution_service::core::domain::risk_decision::{RiskDecision, ApprovedSize, RejectionReason};
    use execution_service::core::application::order_tracker::OrderTracker;
    use execution_service::core::application::idempotency::IdempotencyStore;
    use execution_service::core::application::kill_switch::KillSwitch;
    use execution_service::core::application::mode_controller::ModeController;
    use execution_service::core::domain::operation_mode::OperationMode;
    use execution_service::core::domain::execution_config::ExecutionConfig;
    use execution_service::adapters::metrics::metrics_adapter::MetricsAdapter;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn make_approved_decision() -> RiskDecision {
        RiskDecision {
            request_id: "req-1".to_string(),
            intent_id: "intent-1".to_string(),
            symbol: "AAPL".to_string(),
            approved: true,
            rejection_reason: None,
            approved_size: Some(ApprovedSize::Units(100.0)),
            risk_score: 0.5,
            timestamp_ns: 0,
            latency_us: 0,
        }
    }

    fn make_rejected_decision() -> RiskDecision {
        RiskDecision {
            request_id: "req-2".to_string(),
            intent_id: "intent-2".to_string(),
            symbol: "AAPL".to_string(),
            approved: false,
            rejection_reason: Some(RejectionReason::KillSwitchActive),
            approved_size: None,
            risk_score: 0.0,
            timestamp_ns: 0,
            latency_us: 0,
        }
    }

    fn create_test_engine() -> (
        execution_service::core::application::execution_engine::ExecutionEngine,
        Arc<KillSwitch>,
    ) {
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
            kill_switch.clone(),
            mode_controller,
            metrics,
        );
        (engine, kill_switch)
    }

    #[tokio::test]
    async fn approved_decision_generates_order_cmd() {
        let (engine, _) = create_test_engine();
        let decision = make_approved_decision();
        let now_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        let result = engine.process_decision(&decision, now_ns);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[tokio::test]
    async fn rejected_decision_returns_none() {
        let (engine, _) = create_test_engine();
        let decision = make_rejected_decision();
        let now_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        let result = engine.process_decision(&decision, now_ns);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn kill_switch_prevents_order_submission() {
        let (engine, kill_switch) = create_test_engine();
        kill_switch.enable();

        let decision = make_approved_decision();
        let now_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        let result = engine.process_decision(&decision, now_ns);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn duplicate_intent_id_prevented_by_idempotency() {
        let (engine, _) = create_test_engine();
        let decision = make_approved_decision();
        let now_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        // First submission should succeed
        let result1 = engine.process_decision(&decision, now_ns);
        assert!(result1.is_ok());
        assert!(result1.unwrap().is_some());

        // Second submission with same intent_id should be deduplicated
        let result2 = engine.process_decision(&decision, now_ns);
        assert!(result2.is_ok());
        assert!(result2.unwrap().is_none());
    }
}
