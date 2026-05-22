#[cfg(test)]
mod order_state_machine_unit {
    use execution_service::core::domain::order_state::{OrderState, OrderStatus, OrderStateMachine};
    use execution_service::core::domain::order_lifecycle::{OrderLifecycleEvent, OrderLifecycleEventType};
    use execution_service::core::domain::errors::ExecutionError;

    fn make_event(event_type: OrderLifecycleEventType) -> OrderLifecycleEvent {
        OrderLifecycleEvent {
            event_id: "test-event".to_string(),
            execution_id: "test-exec".to_string(),
            client_order_id: Some("test-client".to_string()),
            symbol: "AAPL".to_string(),
            event_type,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            payload: serde_json::json!({}),
        }
    }

    #[test]
    fn pending_plus_submitted_event_becomes_submitted() {
        let state = OrderState::pending("test-exec", "test-intent", "test-corr", "AAPL", 0);
        let event = make_event(OrderLifecycleEventType::Submitted);
        let new_state = OrderStateMachine::transition(&state, &event).unwrap();
        assert_eq!(new_state.status, OrderStatus::Submitted);
    }

    #[test]
    fn submitted_plus_filled_event_becomes_filled() {
        let mut state = OrderState::pending("test-exec", "test-intent", "test-corr", "AAPL", 0);
        state.status = OrderStatus::Submitted;
        let event = make_event(OrderLifecycleEventType::Filled);
        let new_state = OrderStateMachine::transition(&state, &event).unwrap();
        assert_eq!(new_state.status, OrderStatus::Filled);
    }

    #[test]
    fn submitted_plus_partial_fill_event_becomes_partial_fill() {
        let mut state = OrderState::pending("test-exec", "test-intent", "test-corr", "AAPL", 0);
        state.status = OrderStatus::Submitted;
        let event = make_event(OrderLifecycleEventType::PartialFill);
        let new_state = OrderStateMachine::transition(&state, &event).unwrap();
        assert!(matches!(new_state.status, OrderStatus::PartialFill { .. }));
    }

    #[test]
    fn partial_fill_plus_filled_event_becomes_filled() {
        let mut state = OrderState::pending("test-exec", "test-intent", "test-corr", "AAPL", 0);
        state.status = OrderStatus::PartialFill { filled_qty: 50.0 };
        let event = make_event(OrderLifecycleEventType::Filled);
        let new_state = OrderStateMachine::transition(&state, &event).unwrap();
        assert_eq!(new_state.status, OrderStatus::Filled);
    }

    #[test]
    fn submitted_plus_cancelled_event_becomes_cancelled() {
        let mut state = OrderState::pending("test-exec", "test-intent", "test-corr", "AAPL", 0);
        state.status = OrderStatus::Submitted;
        let event = make_event(OrderLifecycleEventType::Cancelled);
        let new_state = OrderStateMachine::transition(&state, &event).unwrap();
        assert_eq!(new_state.status, OrderStatus::Cancelled);
    }

    #[test]
    fn submitted_plus_rejected_event_becomes_rejected() {
        let mut state = OrderState::pending("test-exec", "test-intent", "test-corr", "AAPL", 0);
        state.status = OrderStatus::Submitted;
        let event = make_event(OrderLifecycleEventType::Rejected);
        let new_state = OrderStateMachine::transition(&state, &event).unwrap();
        assert_eq!(new_state.status, OrderStatus::Rejected);
    }

    #[test]
    fn invalid_transition_filled_to_submitted_returns_error() {
        let mut state = OrderState::pending("test-exec", "test-intent", "test-corr", "AAPL", 0);
        state.status = OrderStatus::Filled;
        let event = make_event(OrderLifecycleEventType::Submitted);
        let result = OrderStateMachine::transition(&state, &event);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ExecutionError::InvalidTransition(_, _)));
    }
}
