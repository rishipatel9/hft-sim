use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub timestamp: u128,
    pub bid_px: f64,
    pub bid_sz: u64,
    pub ask_px: f64,
    pub ask_sz: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub timestamp: u128,
    pub px: f64,
    pub sz: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MarketData {
    #[serde(rename = "quote")]
    Quote(Quote),
    #[serde(rename = "trade")]
    Trade(Trade),
}

pub fn current_timestamp_ns() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos()
}