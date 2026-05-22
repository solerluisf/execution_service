use std::sync::Arc;
use tokio::sync::watch;
use crate::core::application::execution_engine::ExecutionEngine;
use crate::core::ports::risk_input_port::IRiskInputPort;
use crate::core::ports::gateway_client_port::IGatewayClient;
use crate::core::ports::lifecycle_pub_port::IOrderLifecyclePublisher;
use crate::core::ports::orchestration_port::IOrchestrationReceiver;
use crate::adapters::messaging::wire_codec::decode_risk_decision;

pub struct ExecutionService {
    engine: Arc<ExecutionEngine>,
    risk_input: Arc<dyn IRiskInputPort>,
    gateway_client: Arc<dyn IGatewayClient>,
    lifecycle_publisher: Arc<dyn IOrderLifecyclePublisher>,
    orchestration_input: Arc<dyn IOrchestrationReceiver>,
    shutdown: watch::Receiver<bool>,
}

impl ExecutionService {
    pub fn new(
        engine: Arc<ExecutionEngine>,
        risk_input: Arc<dyn IRiskInputPort>,
        gateway_client: Arc<dyn IGatewayClient>,
        lifecycle_publisher: Arc<dyn IOrderLifecyclePublisher>,
        orchestration_input: Arc<dyn IOrchestrationReceiver>,
        shutdown: watch::Receiver<bool>,
    ) -> Self {
        Self {
            engine,
            risk_input,
            gateway_client,
            lifecycle_publisher,
            orchestration_input,
            shutdown,
        }
    }

    pub async fn run(self) {
        let engine = self.engine;
        let shutdown = self.shutdown;

        // Task 1: Risk decision intake
        let engine_1 = engine.clone();
        let risk_input = self.risk_input;
        let gateway_client = self.gateway_client;
        let mut shutdown_1 = shutdown.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_1.changed() => {
                        if *shutdown_1.borrow() {
                            tracing::info!("Risk intake task shutting down");
                            break;
                        }
                    }
                    msg = risk_input.recv() => {
                        if let Some(msg) = msg {
                            if let Ok((decision, _)) = decode_risk_decision(&msg.payload) {
                                let now_ns = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_nanos() as u64;
                                match engine_1.process_decision(&decision, now_ns) {
                                    Ok(Some(order_cmd)) => {
                                        let result = gateway_client.submit_order(order_cmd).await;
                                        engine_1.handle_gateway_response(
                                            &decision.request_id,
                                            &result.map_err(|e| e.to_string()),
                                        );
                                    }
                                    Ok(None) => {}
                                    Err(e) => {
                                        tracing::error!("Error processing decision: {}", e);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        // Task 2: Order lifecycle events
        // Note: This would need a lifecycle input port - simplified for now
        tracing::info!("Lifecycle event task started (placeholder)");

        // Task 3: Orchestrator control commands
        let mut shutdown_3 = shutdown.clone();
        let orchestration_input = self.orchestration_input;
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_3.changed() => {
                        if *shutdown_3.borrow() {
                            tracing::info!("Orchestration task shutting down");
                            break;
                        }
                    }
                    cmd = orchestration_input.recv() => {
                        if let Some(cmd) = cmd {
                            tracing::info!("Received orchestration command: {}", cmd.command);
                            // Dispatch command to engine or config
                        }
                    }
                }
            }
        });

        // Wait for shutdown
        tracing::info!("Execution service running");
    }
}
