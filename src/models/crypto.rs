use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct CMCResponse {
    pub status: Status,
    pub data: HashMap<String, CryptoData>,
}

#[derive(Debug, Deserialize)]
pub struct CryptoData {
    pub name: String,
    pub symbol: String,
    pub quote: HashMap<String, Quote>,
}

#[derive(Debug, Deserialize)]
pub struct Quote {
    pub price: f64,
    pub volume_24h: Option<f64>,
    pub volume_change_24h: Option<f64>,
    pub percent_change_1h: Option<f64>,
    pub percent_change_24h: Option<f64>,
    pub percent_change_7d: Option<f64>,
    pub percent_change_30d: Option<f64>,
    pub percent_change_90d: Option<f64>,
    pub market_cap: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct Status {
    pub error_code: i32,
    pub error_message: Option<String>,
}
