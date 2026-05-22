use std::sync::Arc;
use crate::core::application::kill_switch::KillSwitch;

pub struct KillSwitchSubscriber;

impl KillSwitchSubscriber {
    pub fn spawn(endpoint: String, kill_switch: Arc<KillSwitch>) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || {
            let ctx = zmq::Context::new();
            let socket = ctx.socket(zmq::SUB).unwrap();
            socket.connect(&endpoint).unwrap();
            socket.set_subscribe(b"system.kill_switch").unwrap();
            tracing::info!("Kill switch subscriber connected to {}", endpoint);

            loop {
                match socket.recv_bytes(0) {
                    Ok(_payload) => {
                        tracing::warn!("Kill switch signal received");
                        kill_switch.enable();
                    }
                    Err(e) => {
                        tracing::error!("Kill switch subscriber recv error: {}", e);
                        break;
                    }
                }
            }
        })
    }
}
