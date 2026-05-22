use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationMode { Live, Paper, ReadOnly, Offline }

impl OperationMode {
    pub fn allows_trading(&self) -> bool { matches!(self, Self::Live | Self::Paper) }
    pub fn allows_order_modifications(&self) -> bool { matches!(self, Self::Live | Self::Paper) }
    pub fn is_live(&self) -> bool { matches!(self, Self::Live) }
    pub fn is_paper(&self) -> bool { matches!(self, Self::Paper) }
    pub fn is_read_only(&self) -> bool { matches!(self, Self::ReadOnly) }
    pub fn is_offline(&self) -> bool { matches!(self, Self::Offline) }
}

impl std::fmt::Display for OperationMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Live => write!(f, "Live"),
            Self::Paper => write!(f, "Paper"),
            Self::ReadOnly => write!(f, "ReadOnly"),
            Self::Offline => write!(f, "Offline"),
        }
    }
}

impl std::str::FromStr for OperationMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "live" => Ok(Self::Live),
            "paper" => Ok(Self::Paper),
            "readonly" => Ok(Self::ReadOnly),
            "offline" => Ok(Self::Offline),
            _ => Err(format!("Unknown operation mode: {}", s)),
        }
    }
}
