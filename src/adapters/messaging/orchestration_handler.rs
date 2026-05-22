use std::sync::Arc;
use tokio::sync::Mutex;
use crate::core::ports::orchestration_port::{IOrchestrationReceiver, OrchestrationCommand};

pub struct OrchestrationHandler {
    receiver: Arc<Mutex<tokio::sync::mpsc::Receiver<OrchestrationCommand>>>,
}

impl OrchestrationHandler {
    pub fn spawn(endpoint: String, topic: String) -> (Self, std::thread::JoinHandle<()>) {
        let (tx, rx) = tokio::sync::mpsc::channel::<OrchestrationCommand>(100);

        let thread_handle = std::thread::spawn(move || {
            let ctx = zmq::Context::new();
            let socket = ctx.socket(zmq::SUB).unwrap();
            socket.connect(&endpoint).unwrap();
            socket.set_subscribe(topic.as_bytes()).unwrap();
            tracing::info!("Orchestration handler connected to {}, subscribed to {}", endpoint, topic);

            loop {
                match socket.recv_multipart(0) {
                    Ok(frames) => {
                        let received_ns = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_nanos() as u64;
                        let command = OrchestrationCommand {
                            command: String::from_utf8_lossy(&frames[0]).to_string(),
                            payload: frames.get(1).cloned().unwrap_or_default(),
                            received_ns,
                        };
                        if tx.blocking_send(command).is_err() {
                            tracing::warn!("Orchestration handler channel closed");
                            break;
                        }
                    }
                    Err(e) => {
                        tracing::error!("Orchestration handler recv error: {}", e);
                        break;
                    }
                }
            }
        });

        (
            Self {
                receiver: Arc::new(Mutex::new(rx)),
            },
            thread_handle,
        )
    }
}

#[async_trait::async_trait]
impl IOrchestrationReceiver for OrchestrationHandler {
    async fn recv(&self) -> Option<OrchestrationCommand> {
        let mut rx = self.receiver.lock().await;
        rx.recv().await
    }
}
