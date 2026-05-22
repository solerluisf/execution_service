use std::sync::Arc;
use tokio::sync::Mutex;
use crate::core::ports::lifecycle_pub_port::IOrderLifecyclePublisher;
use crate::core::domain::order_lifecycle::OrderLifecycleEvent;
use crate::adapters::messaging::wire_codec::encode_order_lifecycle_event;

pub struct LifecyclePublisher {
    socket: Arc<Mutex<zmq::Socket>>,
}

impl LifecyclePublisher {
    pub fn spawn(endpoint: String) -> (Self, std::thread::JoinHandle<()>) {
        let ctx = zmq::Context::new();
        let socket = ctx.socket(zmq::PUB).unwrap();
        socket.bind(&endpoint).unwrap();
        tracing::info!("Lifecycle publisher bound to {}", endpoint);

        let socket_arc = Arc::new(Mutex::new(socket));

        let thread_handle = {
            let _socket = socket_arc.clone();
            std::thread::spawn(move || {
                // Publisher runs in background - messages sent via send method
                loop {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            })
        };

        (Self { socket: socket_arc }, thread_handle)
    }

    pub fn send(&self, event: &OrderLifecycleEvent) {
        if let Ok(encoded) = encode_order_lifecycle_event(event) {
            let socket = self.socket.blocking_lock();
            let _ = socket.send("service.execution.order_lifecycle", zmq::SNDMORE);
            let _ = socket.send(&encoded, 0);
        }
    }
}

#[async_trait::async_trait]
impl IOrderLifecyclePublisher for LifecyclePublisher {
    async fn publish(&self, event: &OrderLifecycleEvent) {
        self.send(event);
    }
}
