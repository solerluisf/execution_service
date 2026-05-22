use std::sync::Arc;
use tokio::sync::{watch, RwLock};
use execution_service::config::app_config::AppConfig;
use execution_service::core::domain::execution_config::ExecutionConfig;
use execution_service::core::application::kill_switch::KillSwitch;
use execution_service::core::application::mode_controller::ModeController;
use execution_service::core::application::order_tracker::OrderTracker;
use execution_service::core::application::idempotency::IdempotencyStore;
use execution_service::core::application::execution_engine::ExecutionEngine;
use execution_service::core::application::execution_service::ExecutionService;
use execution_service::adapters::messaging::risk_subscriber::RiskSubscriber;
use execution_service::adapters::messaging::gateway_client::GatewayClient;
use execution_service::adapters::messaging::lifecycle_publisher::LifecyclePublisher;
use execution_service::adapters::messaging::heartbeat_publisher::HeartbeatPublisher;
use execution_service::adapters::messaging::orchestration_handler::OrchestrationHandler;
use execution_service::adapters::messaging::kill_switch_subscriber::KillSwitchSubscriber;
use execution_service::adapters::messaging::mode_subscriber::ModeSubscriber;
use execution_service::adapters::metrics::metrics_adapter::MetricsAdapter;
use execution_service::adapters::metrics::health_endpoint::HealthReporter;
use execution_service::infra::http_server::start_http_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load AppConfig from env
    let app_config = AppConfig::from_env().expect("Failed to load app config");

    // 2. Load ExecutionConfig from execution.toml
    let execution_config = ExecutionConfig::from_file(&app_config.execution_config_path)
        .unwrap_or_else(|e| {
            tracing::warn!("Failed to load execution config: {}, using defaults", e);
            ExecutionConfig::default()
        });

    // 3. Init tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("execution_service=info".parse().unwrap())
        )
        .init();

    tracing::info!("Starting Execution Service");

    // 4. Arc<KillSwitch>
    let kill_switch = Arc::new(KillSwitch::new());

    // 5. Arc<ModeController>
    let mode_controller = Arc::new(ModeController::new(app_config.operation_mode));

    // 6. Arc<JournalStorage> (simplified for now)
    // TODO: Wire up journal storage when needed

    // 7. Arc<OrderTracker>
    let order_tracker = Arc::new(OrderTracker::new());

    // 8. Arc<RiskSubscriber> (DEALER -> Risk ROUTER 5559)
    let (risk_subscriber, _risk_thread) = RiskSubscriber::spawn(
        app_config.risk_router_endpoint.clone(),
    );
    let risk_input: Arc<dyn execution_service::core::ports::risk_input_port::IRiskInputPort> =
        Arc::new(risk_subscriber);

    // 9. GatewayClient::spawn() -> Arc<GatewayClient> (DEALER -> Gateway ROUTER 5555)
    let (gateway_client, _gateway_thread) = GatewayClient::spawn(
        app_config.gateway_router_endpoint.clone(),
    );
    let gateway_client: Arc<dyn execution_service::core::ports::gateway_client_port::IGatewayClient> =
        Arc::new(gateway_client);

    // 10. LifecyclePublisher::spawn() -> Arc
    let (lifecycle_publisher, _lifecycle_thread) = LifecyclePublisher::spawn(
        app_config.lifecycle_pub_endpoint.clone(),
    );
    let lifecycle_pub: Arc<dyn execution_service::core::ports::lifecycle_pub_port::IOrderLifecyclePublisher> =
        Arc::new(lifecycle_publisher);

    // 11. Arc<MetricsAdapter>
    let metrics: Arc<dyn execution_service::core::ports::metrics_port::IMetricsPort> =
        Arc::new(MetricsAdapter::new());

    // 12. Arc<HealthReporter>
    let health_reporter = Arc::new(HealthReporter::new());

    // 13. IdempotencyStore
    let idempotency = Arc::new(IdempotencyStore::new());

    // 14. ExecutionEngine
    let engine = Arc::new(ExecutionEngine::new(
        Arc::new(RwLock::new(execution_config)),
        order_tracker,
        idempotency,
        kill_switch.clone(),
        mode_controller.clone(),
        metrics,
    ));

    // 15. ExecutionService
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let (orchestration_handler, _orch_thread) = OrchestrationHandler::spawn(
        app_config.control_endpoint.clone(),
        "orchestrator.control.execution".to_string(),
    );
    let orchestration_input: Arc<dyn execution_service::core::ports::orchestration_port::IOrchestrationReceiver> =
        Arc::new(orchestration_handler);

    let execution_service = ExecutionService::new(
        engine,
        risk_input,
        gateway_client,
        lifecycle_pub,
        orchestration_input,
        shutdown_rx,
    );

    // 16. Read kill-switch state from journal (startup recovery)
    // TODO: Implement journal recovery

    // 17. Read last mode from journal (startup recovery)
    // TODO: Implement journal recovery

    // 18. Start HTTP server - tokio::spawn
    let _http_handle = start_http_server(app_config.health_port, health_reporter).await;

    // 19. Start HeartbeatPublisher - tokio::spawn
    let (_heartbeat, _heartbeat_thread) = HeartbeatPublisher::spawn(
        "tcp://127.0.0.1:5580".to_string(), // TODO: make configurable
    );

    // 20. Start KillSwitchSubscriber - tokio::spawn
    let _kill_switch_thread = KillSwitchSubscriber::spawn(
        "tcp://127.0.0.1:5581".to_string(), // TODO: make configurable
        kill_switch,
    );

    // 21. Start ModeSubscriber - tokio::spawn
    let _mode_thread = ModeSubscriber::spawn(
        "tcp://127.0.0.1:5582".to_string(), // TODO: make configurable
        mode_controller,
    );

    // 22. Start OrchestrationCommandHandler - already started above

    // 23. Start ExecutionService::run() - tokio::spawn
    let service_handle = tokio::spawn(async move {
        execution_service.run().await;
    });

    // 24. Await SIGINT/SIGTERM
    let shutdown_tx_clone = shutdown_tx.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        tracing::info!("Shutdown signal received");
        let _ = shutdown_tx_clone.send(true);
    });

    // 25. Broadcast shutdown via watch::Sender<bool>
    // Done via ctrl_c handler above

    // 26. Await all task JoinHandles
    let _ = service_handle.await;

    tracing::info!("Execution Service shutdown complete");
    Ok(())
}
