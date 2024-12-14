use anyhow::Result;
use crate::models::crypto::{CMCResponse, CryptoData};
use crate::models::fear_greed::{FearGreedResponse, FearGreedData};
use std::collections::HashMap;
use itertools::Itertools;
use crate::services::logger::{log_error, log_info};

const CMC_QUOTES_URL: &str = "https://pro-api.coinmarketcap.com/v2/cryptocurrency/quotes/latest";
const CMC_FEAR_GREED_URL: &str = "https://pro-api.coinmarketcap.com/v3/fear-and-greed/historical";

/// Fetches current cryptocurrency prices from CoinMarketCap API
pub async fn fetch_prices(api_key: &str, token_names: &[String]) -> Result<HashMap<String, CryptoData>> {
    let client = reqwest::Client::new();
    let slugs = token_names.iter()
        .map(|token| token.as_str())
        .join(",");
    
    let response = client
        .get(CMC_QUOTES_URL)
        .header("X-CMC_PRO_API_KEY", api_key)
        .query(&[
            ("slug", slugs.as_str()),
            ("convert", "USD"),
        ])
        .send()
        .await?;

    let response_text = response.text().await?;
    
    match serde_json::from_str::<CMCResponse>(&response_text) {
        Ok(parsed) => {
            if parsed.status.error_code != 0 {
                let error_msg = parsed.status.error_message.unwrap_or_default();
                log_error("API Error", &error_msg)?;
                anyhow::bail!("API Error: {}", error_msg);
            }
            Ok(parsed.data)
        },
        Err(e) => {
            log_error("Parse Error", &e.to_string())?;
            anyhow::bail!("Failed to parse API response: {}", e)
        }
    }
}

/// Fetches historical fear and greed index data from CoinMarketCap API
pub async fn fetch_fear_greed(api_key: &str, limit: &str) -> Result<Vec<FearGreedData>> {
    let client = reqwest::Client::new();
    
    log_info("Fear & Greed", "Fetching historical data...")?;
    
    let response = client
        .get(CMC_FEAR_GREED_URL)
        .header("X-CMC_PRO_API_KEY", api_key)
        .query(&[
            ("limit", limit),
        ])
        .send()
        .await?;

    let response_text = response.text().await?;
    
    // Don't log the full response, just log the status
    log_info("Fear & Greed", "Response received successfully")?;
    
    match serde_json::from_str::<FearGreedResponse>(&response_text) {
        Ok(parsed) => {
            if parsed.status.error_code_str != "0" {
                // Keep this as error since it's an actual API error
                log_error("Fear & Greed API Error", &parsed.status.error_message)?;
                anyhow::bail!("API Error: {}", parsed.status.error_message);
            }
            
            // Log data points as INFO
            if let Some(first) = parsed.data.first() {
                let ts = first.timestamp.parse::<i64>().unwrap_or(0);
                let date = chrono::DateTime::from_timestamp(ts, 0)
                    .unwrap_or_default()
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string();
                log_info("Fear & Greed", 
                    &format!("Latest data point: {} = {} ({})", 
                        date, first.value, first.value_classification))?;
            }
            
            if let Some(last) = parsed.data.last() {
                let ts = last.timestamp.parse::<i64>().unwrap_or(0);
                let date = chrono::DateTime::from_timestamp(ts, 0)
                    .unwrap_or_default()
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string();
                log_info("Fear & Greed", 
                    &format!("Oldest data point: {} = {} ({})", 
                        date, last.value, last.value_classification))?;
            }

            // Add a summary log
            log_info("Fear & Greed", 
                &format!("Successfully fetched {} data points", parsed.data.len()))?;
            
            Ok(parsed.data)
        },
        Err(e) => {
            // Keep this as error since it's a parsing error
            log_error("Fear & Greed Parse Error", &e.to_string())?;
            anyhow::bail!("Failed to parse Fear & Greed response: {}", e)
        }
    }
}
