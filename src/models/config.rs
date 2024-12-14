use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TokenConfig {
    pub name: String,
    #[serde(default)]
    pub owned: Option<f64>,
    #[serde(default)]
    pub avg_buy_price: Option<f64>,
    #[serde(default = "default_true")]
    pub in_watchlist: bool,
    #[serde(default = "default_true")]
    pub in_portfolio: bool,
}

impl TokenConfig {
    pub fn is_in_portfolio(&self) -> bool {
        self.in_portfolio || (self.owned.is_some() && self.owned.unwrap() > 0.0)
    }

    pub fn is_in_watchlist(&self) -> bool {
        self.in_watchlist
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub api_key: String,
    pub tokens: Vec<TokenConfig>,
    pub refresh_interval: u64,
    pub fear_and_greed_limit: String,
}
