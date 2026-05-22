use serde::{Serialize, Deserialize, de::DeserializeOwned};
use std::sync::atomic::{AtomicU64, Ordering};
use crate::core::domain::risk_decision::RiskDecision;
use crate::core::domain::order_lifecycle::OrderLifecycleEvent;

const MSGPACK_MAGIC: &[u8; 4] = b"BGW1";

static DECODE_MSGPACK_TOTAL: AtomicU64 = AtomicU64::new(0);
static DECODE_ERROR_TOTAL: AtomicU64 = AtomicU64::new(0);
static ENCODE_ERROR_TOTAL: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireFormat { MessagePack }

fn encode_msgpack<T: Serialize>(value: &T) -> Result<Vec<u8>, String> {
    let payload = rmp_serde::to_vec_named(value).map_err(|e| {
        ENCODE_ERROR_TOTAL.fetch_add(1, Ordering::Relaxed);
        e.to_string()
    })?;
    let mut framed = Vec::with_capacity(MSGPACK_MAGIC.len() + payload.len());
    framed.extend_from_slice(MSGPACK_MAGIC);
    framed.extend_from_slice(&payload);
    Ok(framed)
}

fn decode_msgpack<T: DeserializeOwned>(bytes: &[u8]) -> Result<(T, WireFormat), String> {
    if !bytes.starts_with(MSGPACK_MAGIC) {
        DECODE_ERROR_TOTAL.fetch_add(1, Ordering::Relaxed);
        return Err(format!("Invalid wire format: expected {} prefix", String::from_utf8_lossy(MSGPACK_MAGIC)));
    }
    let decoded = rmp_serde::from_slice::<T>(&bytes[MSGPACK_MAGIC.len()..]).map_err(|e| {
        DECODE_ERROR_TOTAL.fetch_add(1, Ordering::Relaxed);
        e.to_string()
    })?;
    DECODE_MSGPACK_TOTAL.fetch_add(1, Ordering::Relaxed);
    Ok((decoded, WireFormat::MessagePack))
}

pub fn encode_risk_decision(dec: &RiskDecision) -> Result<Vec<u8>, String> { encode_msgpack(dec) }
pub fn decode_risk_decision(bytes: &[u8]) -> Result<(RiskDecision, WireFormat), String> { decode_msgpack(bytes) }

pub fn encode_order_lifecycle_event(event: &OrderLifecycleEvent) -> Result<Vec<u8>, String> { encode_msgpack(event) }
pub fn decode_order_lifecycle_event(bytes: &[u8]) -> Result<(OrderLifecycleEvent, WireFormat), String> { decode_msgpack(bytes) }

// Gateway request/response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GatewayRequest {
    SubmitOrder(OrderCmd),
    CancelOrder(CancelCmd),
    ReplaceOrder(ReplaceCmd),
    QueryStatus(StatusQuery),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCmd {
    pub correlation_id: Option<String>,
    pub client_order_id: Option<String>,
    pub symbol: String,
    pub qty: f64,
    pub side: String,
    pub time_in_force: String,
    pub extended_hours: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelCmd {
    pub correlation_id: Option<String>,
    pub execution_id: String,
    pub symbol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaceCmd {
    pub correlation_id: Option<String>,
    pub execution_id: String,
    pub symbol: String,
    pub new_qty: Option<f64>,
    pub new_price: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusQuery {
    pub correlation_id: Option<String>,
    pub execution_id: String,
    pub symbol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GatewayResponse {
    Ok { correlation_id: Option<String>, result: String },
    Err { correlation_id: Option<String>, message: String },
}

impl GatewayResponse {
    pub fn correlation_id(&self) -> Option<&String> {
        match self {
            GatewayResponse::Ok { correlation_id, .. } => correlation_id.as_ref(),
            GatewayResponse::Err { correlation_id, .. } => correlation_id.as_ref(),
        }
    }
}

pub fn encode_gateway_request(req: &GatewayRequest) -> Vec<u8> {
    encode_msgpack(req).unwrap_or_else(|e| {
        tracing::error!("Failed to encode gateway request: {}", e);
        vec![]
    })
}

pub fn decode_gateway_response(bytes: &[u8]) -> Result<(GatewayResponse, WireFormat), String> {
    decode_msgpack(bytes)
}

pub fn encode_gateway_response(resp: &GatewayResponse) -> Result<Vec<u8>, String> {
    encode_msgpack(resp)
}

pub fn decode_gateway_request(bytes: &[u8]) -> Result<(GatewayRequest, WireFormat), String> {
    decode_msgpack(bytes)
}
