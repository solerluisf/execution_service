use std::sync::Arc;
use tokio::sync::Mutex;
use crate::core::ports::risk_input_port::{IRiskInputPort, RiskDecisionMessage};

pub struct RiskSubscriber {
    receiver: Arc<Mutex<tokio::sync::mpsc::Receiver<RiskDecisionMessage>>>,
}

impl RiskSubscriber {
    pub fn spawn(endpoint: String) -> (Self, std::thread::JoinHandle<()>) {
        let (tx, rx) = tokio::sync::mpsc::channel::<RiskDecisionMessage>(1000);

        let thread_handle = std::thread::spawn(move || {
            let ctx = zmq::Context::new();
            let socket = ctx.socket(zmq::DEALER).unwrap();
            socket.connect(&endpoint).unwrap();
            tracing::info!("Risk subscriber connected to {}", endpoint);

            loop {
                match socket.recv_bytes(0) {
                    Ok(payload) => {
                        let received_ns = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_nanos() as u64;
                        let msg = RiskDecisionMessage { payload, received_ns };
                        if tx.blocking_send(msg).is_err() {
                            tracing::warn!("Risk subscriber channel closed");
                            break;
                        }
                    }
                    Err(e) => {
                        tracing::error!("Risk subscriber recv error: {}", e);
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
impl IRiskInputPort for RiskSubscriber {
    async fn recv(&self) -> Option<RiskDecisionMessage> {
        let mut rx = self.receiver.lock().await;
        rx.recv().await
    }
}
