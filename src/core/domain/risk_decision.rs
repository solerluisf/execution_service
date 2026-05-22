use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RiskDecision {
    pub request_id: String,
    pub intent_id: String,
    pub symbol: String,
    pub approved: bool,
    pub rejection_reason: Option<RejectionReason>,
    pub approved_size: Option<ApprovedSize>,
    pub risk_score: f64,
    pub timestamp_ns: u64,
    pub latency_us: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum RejectionReason {
    KillSwitchActive,
    DuplicateIntent,
    StaleIntent,
    ExposureLimitBreached { limit: f64, current: f64 },
    DrawdownLimitBreached { limit_pct: f64, current_pct: f64 },
    LeverageLimitBreached { limit: f64, current: f64 },
    VolatilityGuardTripped { threshold: f64, current_vol: f64 },
    SpreadGuardTripped { threshold_bps: f64, current_spread_bps: f64 },
    OrderRateLimitBreached,
    LiquidityGuardTripped { min_volume: f64, observed_volume: f64 },
    SlippageGuardTripped { limit_bps: f64 },
    PositionLimitBreached { symbol: String, limit: f64, current: f64 },
    PortfolioExposureLimitBreached { limit_pct: f64, current_pct: f64 },
    IntentValidationFailed { reason: String },
    OperationModeBlocked,
    SymbolPaused,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ApprovedSize {
    Units(f64),
    Notional(f64),
}
