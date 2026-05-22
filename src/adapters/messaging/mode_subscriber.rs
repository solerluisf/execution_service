use std::sync::Arc;
use crate::core::application::mode_controller::ModeController;
use crate::core::domain::operation_mode::OperationMode;

pub struct ModeSubscriber;

impl ModeSubscriber {
    pub fn spawn(endpoint: String, mode_controller: Arc<ModeController>) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || {
            let ctx = zmq::Context::new();
            let socket = ctx.socket(zmq::SUB).unwrap();
            socket.connect(&endpoint).unwrap();
            socket.set_subscribe(b"system.mode").unwrap();
            tracing::info!("Mode subscriber connected to {}", endpoint);

            loop {
                match socket.recv_bytes(0) {
                    Ok(payload) => {
                        if let Ok(mode_str) = String::from_utf8(payload) {
                            if let Ok(mode) = mode_str.parse::<OperationMode>() {
                                mode_controller.set(mode);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Mode subscriber recv error: {}", e);
                        break;
                    }
                }
            }
        })
    }
}
