use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, oneshot};
use crate::core::ports::gateway_client_port::{IGatewayClient, ExecutionId, OrderStatusResponse};
use crate::core::domain::errors::ExecutionError;
use crate::adapters::messaging::wire_codec::{
    GatewayRequest, GatewayResponse, OrderCmd, CancelCmd, ReplaceCmd, StatusQuery,
    encode_gateway_request, decode_gateway_response, decode_gateway_request,
};

pub struct GatewayClient {
    pending: Arc<Mutex<HashMap<String, oneshot::Sender<GatewayResponse>>>>,
    sender: Arc<Mutex<tokio::sync::mpsc::Sender<(Vec<u8>, oneshot::Sender<GatewayResponse>)>>>,
}

impl GatewayClient {
    pub fn spawn(endpoint: String) -> (Self, std::thread::JoinHandle<()>) {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<(Vec<u8>, oneshot::Sender<GatewayResponse>)>(1000);
        let pending = Arc::new(Mutex::new(HashMap::<String, oneshot::Sender<GatewayResponse>>::new()));

        let pending_clone = pending.clone();
        let thread_handle = std::thread::spawn(move || {
            let ctx = zmq::Context::new();
            let socket = ctx.socket(zmq::DEALER).unwrap();
            socket.connect(&endpoint).unwrap();
            tracing::info!("Gateway client connected to {}", endpoint);

            loop {
                // Use poll to check for both incoming responses and new requests
                let mut poll_items = vec![socket.as_poll_item(zmq::POLLIN)];
                if zmq::poll(&mut poll_items, 10).is_err() {
                    break;
                }

                // Check for incoming responses
                if let Ok(bytes) = socket.recv_bytes(zmq::DONTWAIT) {
                    if let Ok((response, _)) = decode_gateway_response(&bytes) {
                        if let Some(corr_id) = response.correlation_id() {
                            let mut pending = pending_clone.blocking_lock();
                            if let Some(tx) = pending.remove(corr_id) {
                                let _ = tx.send(response);
                            }
                        }
                    }
                }

                // Check for new requests from channel
                while let Ok((bytes, response_tx)) = rx.try_recv() {
                    // Extract correlation_id from the request to track pending
                    if let Ok((request, _)) = decode_gateway_request(&bytes) {
                        let corr_id = match &request {
                            GatewayRequest::SubmitOrder(cmd) => cmd.correlation_id.clone(),
                            GatewayRequest::CancelOrder(cmd) => cmd.correlation_id.clone(),
                            GatewayRequest::ReplaceOrder(cmd) => cmd.correlation_id.clone(),
                            GatewayRequest::QueryStatus(query) => query.correlation_id.clone(),
                        }.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

                        pending_clone.blocking_lock().insert(corr_id, response_tx);
                    }
                    if socket.send(&bytes, 0).is_err() {
                        tracing::error!("Failed to send to gateway");
                    }
                }
            }
        });

        (
            Self {
                pending,
                sender: Arc::new(Mutex::new(tx)),
            },
            thread_handle,
        )
    }
}

#[async_trait::async_trait]
impl IGatewayClient for GatewayClient {
    async fn submit_order(&self, cmd: OrderCmd) -> Result<ExecutionId, ExecutionError> {
        let _corr_id = cmd.correlation_id.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let (tx, rx) = oneshot::channel();

        let request = GatewayRequest::SubmitOrder(cmd);
        let encoded = encode_gateway_request(&request);

        let sender = self.sender.lock().await;
        sender.send((encoded, tx)).await.map_err(|e| {
            ExecutionError::MessagingError(format!("Failed to send request: {}", e))
        })?;
        drop(sender);

        let response = rx.await.map_err(|e| {
            ExecutionError::MessagingError(format!("Failed to receive response: {}", e))
        })?;

        match response {
            GatewayResponse::Ok { result, .. } => Ok(result),
            GatewayResponse::Err { message, .. } => Err(ExecutionError::GatewayError(message)),
        }
    }

    async fn cancel_order(&self, cmd: CancelCmd) -> Result<(), ExecutionError> {
        let _corr_id = cmd.correlation_id.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let (tx, rx) = oneshot::channel();

        let request = GatewayRequest::CancelOrder(cmd);
        let encoded = encode_gateway_request(&request);

        let sender = self.sender.lock().await;
        sender.send((encoded, tx)).await.map_err(|e| {
            ExecutionError::MessagingError(format!("Failed to send request: {}", e))
        })?;
        drop(sender);

        let response = rx.await.map_err(|e| {
            ExecutionError::MessagingError(format!("Failed to receive response: {}", e))
        })?;

        match response {
            GatewayResponse::Ok { .. } => Ok(()),
            GatewayResponse::Err { message, .. } => Err(ExecutionError::GatewayError(message)),
        }
    }

    async fn replace_order(&self, cmd: ReplaceCmd) -> Result<(), ExecutionError> {
        let _corr_id = cmd.correlation_id.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let (tx, rx) = oneshot::channel();

        let request = GatewayRequest::ReplaceOrder(cmd);
        let encoded = encode_gateway_request(&request);

        let sender = self.sender.lock().await;
        sender.send((encoded, tx)).await.map_err(|e| {
            ExecutionError::MessagingError(format!("Failed to send request: {}", e))
        })?;
        drop(sender);

        let response = rx.await.map_err(|e| {
            ExecutionError::MessagingError(format!("Failed to receive response: {}", e))
        })?;

        match response {
            GatewayResponse::Ok { .. } => Ok(()),
            GatewayResponse::Err { message, .. } => Err(ExecutionError::GatewayError(message)),
        }
    }

    async fn query_status(&self, query: StatusQuery) -> Result<OrderStatusResponse, ExecutionError> {
        let _corr_id = query.correlation_id.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let (tx, rx) = oneshot::channel();

        let request = GatewayRequest::QueryStatus(query);
        let encoded = encode_gateway_request(&request);

        let sender = self.sender.lock().await;
        sender.send((encoded, tx)).await.map_err(|e| {
            ExecutionError::MessagingError(format!("Failed to send request: {}", e))
        })?;
        drop(sender);

        let response = rx.await.map_err(|e| {
            ExecutionError::MessagingError(format!("Failed to receive response: {}", e))
        })?;

        match response {
            GatewayResponse::Ok { result, .. } => Ok(OrderStatusResponse {
                execution_id: result.clone(),
                status: "Unknown".to_string(),
                symbol: String::new(),
            }),
            GatewayResponse::Err { message, .. } => Err(ExecutionError::GatewayError(message)),
        }
    }
}
