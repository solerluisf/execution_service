use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Heartbeat {
    pub service: String,
    pub timestamp_ns: u64,
    pub status: String,
}

pub struct HeartbeatPublisher {
    socket: Arc<Mutex<zmq::Socket>>,
}

impl HeartbeatPublisher {
    pub fn spawn(endpoint: String) -> (Self, std::thread::JoinHandle<()>) {
        let ctx = zmq::Context::new();
        let socket = ctx.socket(zmq::PUB).unwrap();
        socket.bind(&endpoint).unwrap();
        tracing::info!("Heartbeat publisher bound to {}", endpoint);

        let socket_arc = Arc::new(Mutex::new(socket));

        let thread_handle = {
            let socket = socket_arc.clone();
            std::thread::spawn(move || {
                loop {
                    let heartbeat = Heartbeat {
                        service: "execution".to_string(),
                        timestamp_ns: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_nanos() as u64,
                        status: "healthy".to_string(),
                    };
                    if let Ok(encoded) = rmp_serde::to_vec_named(&heartbeat) {
                        let socket = socket.blocking_lock();
                        let mut framed = Vec::with_capacity(4 + encoded.len());
                        framed.extend_from_slice(b"BGW1");
                        framed.extend_from_slice(&encoded);
                        let _ = socket.send("service.execution.health", zmq::SNDMORE);
                        let _ = socket.send(&framed, 0);
                    }
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            })
        };

        (Self { socket: socket_arc }, thread_handle)
    }
}
