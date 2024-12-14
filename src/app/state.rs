use std::collections::HashMap;
use tui::widgets::TableState;
use chrono::{DateTime, Local};
use anyhow::Result;

use crate::models::config::{Config, TokenConfig};
use crate::models::crypto::CryptoData;
use crate::models::fear_greed::FearGreedData;
use crate::services::api;

#[derive(Debug)]
pub enum Command {
    Add {
        name: String,
        watchlist: bool,
        portfolio: bool,
        owned: Option<f64>,
        avg_buy_price: Option<f64>,
    },
    Remove {
        name: String,
        watchlist: bool,
        portfolio: bool,
    },
    Invalid(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortColumn {
    Symbol,
    Price,
    Change1h,
    Change24h,
    Change7d,
    Change30d,
    Change90d,
    Volume24h,
    VolumeChange,
    MarketCap,
    // Portfolio specific columns
    Holdings,
    AvgBuy,
    CurrentValue,
    CostBasis,
    ProfitLoss,
    ProfitLossPercent,
}

pub struct App {
    pub config: Config,
    pub table_state: TableState,
    pub crypto_data: HashMap<String, CryptoData>,
    pub last_update: Option<DateTime<Local>>,
    pub last_error: Option<String>,
    pub fear_greed_data: Vec<FearGreedData>,
    pub tab_index: usize,
    pub sort_column: SortColumn,
    pub sort_ascending: bool,
    pub portfolio_sort_column: SortColumn,
    pub input_mode: InputMode,
    pub input: String,
}

impl App {
    pub fn new(config: Config) -> App {
        App {
            config,
            table_state: TableState::default(),
            crypto_data: HashMap::new(),
            last_update: None,
            last_error: None,
            fear_greed_data: Vec::new(),
            tab_index: 0,
            sort_column: SortColumn::MarketCap,
            sort_ascending: false,
            portfolio_sort_column: SortColumn::CurrentValue,
            input_mode: InputMode::Normal,
            input: String::new(),
        }
    }

    pub fn enter_edit_mode(&mut self) {
        self.input_mode = InputMode::Editing;
        self.input.clear();
    }

    pub fn exit_edit_mode(&mut self) {
        self.input_mode = InputMode::Normal;
        self.input.clear();
    }

    pub async fn fetch_prices(&self) -> Result<HashMap<String, CryptoData>> {
        let token_names: Vec<String> = self.config.tokens
            .iter()
            .map(|token| token.name.clone())
            .collect();
        api::fetch_prices(&self.config.api_key, &token_names).await
    }

    pub async fn fetch_fear_greed(&self) -> Result<Vec<FearGreedData>> {
        api::fetch_fear_greed(&self.config.api_key, &self.config.fear_and_greed_limit).await
    }

    pub fn next(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.crypto_data.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.crypto_data.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn next_tab(&mut self) {
        self.tab_index = (self.tab_index + 1) % 3;
    }

    pub async fn process_command(&mut self) -> Result<()> {
        let command = self.parse_command();
        match command {
            Command::Add { name, watchlist, portfolio, owned, avg_buy_price } => {
                // Update config
                let token = self.config.tokens.iter_mut()
                    .find(|t| t.name.to_lowercase() == name.to_lowercase());

                match token {
                    Some(token) => {
                        // Update existing token
                        if watchlist {
                            token.in_watchlist = true;
                        }
                        if portfolio {
                            token.in_portfolio = true;
                            if let Some(owned) = owned {
                                token.owned = Some(owned);
                            }
                            if let Some(price) = avg_buy_price {
                                token.avg_buy_price = Some(price);
                            }
                        }
                    }
                    None => {
                        // Add new token
                        self.config.tokens.push(TokenConfig {
                            name,
                            owned,
                            avg_buy_price,
                            in_watchlist: watchlist,
                            in_portfolio: portfolio,
                        });
                    }
                }

                // Save config
                let config_str = serde_json::to_string_pretty(&self.config)?;
                std::fs::write("config.json", config_str)?;

                // Refresh data
                if let Ok(new_data) = self.fetch_prices().await {
                    self.crypto_data = new_data;
                    self.last_update = Some(Local::now());
                }
            }
            Command::Remove { name, watchlist, portfolio } => {
                if let Some(token) = self.config.tokens.iter_mut()
                    .find(|t| t.name.to_lowercase() == name.to_lowercase())
                {
                    if watchlist {
                        token.in_watchlist = false;
                    }
                    if portfolio {
                        token.in_portfolio = false;
                        token.owned = None;
                        token.avg_buy_price = None;
                    }
                }

                // Remove token completely if neither in watchlist nor portfolio
                self.config.tokens.retain(|t| t.in_watchlist || t.in_portfolio);

                // Save config
                let config_str = serde_json::to_string_pretty(&self.config)?;
                std::fs::write("config.json", config_str)?;

                // Refresh data
                if let Ok(new_data) = self.fetch_prices().await {
                    self.crypto_data = new_data;
                    self.last_update = Some(Local::now());
                }
            }
            Command::Invalid(msg) => {
                self.last_error = Some(msg);
            }
        }
        Ok(())
    }

    fn parse_command(&self) -> Command {
        let parts: Vec<&str> = self.input.split_whitespace().collect();
        if parts.is_empty() {
            return Command::Invalid("Empty command".to_string());
        }

        match parts[0] {
            "add" => {
                if parts.len() < 2 {
                    return Command::Invalid("Usage: add <name> [-w|-p] [amount] [price]".to_string());
                }

                let name = parts[1].to_string();
                let mut watchlist = false;
                let mut portfolio = false;
                let mut owned = None;
                let mut avg_buy_price = None;

                let mut i = 2;
                while i < parts.len() {
                    match parts[i] {
                        "-w" => watchlist = true,
                        "-p" => {
                            portfolio = true;
                            if i + 2 < parts.len() {
                                owned = parts[i + 1].parse().ok();
                                avg_buy_price = parts[i + 2].parse().ok();
                                i += 2;
                            }
                        }
                        "-wp" | "-pw" => {
                            watchlist = true;
                            portfolio = true;
                            if i + 2 < parts.len() {
                                owned = parts[i + 1].parse().ok();
                                avg_buy_price = parts[i + 2].parse().ok();
                                i += 2;
                            }
                        }
                        _ => return Command::Invalid("Invalid flag".to_string()),
                    }
                    i += 1;
                }

                if !watchlist && !portfolio {
                    watchlist = true; // Default to watchlist if no flags specified
                }

                Command::Add {
                    name,
                    watchlist,
                    portfolio,
                    owned,
                    avg_buy_price,
                }
            }
            "rm" => {
                if parts.len() < 2 {
                    return Command::Invalid("Usage: rm <name> [-w|-p|-wp]".to_string());
                }

                let name = parts[1].to_string();
                let mut watchlist = false;
                let mut portfolio = false;

                for flag in parts.iter().skip(2) {
                    match *flag {
                        "-w" => watchlist = true,
                        "-p" => portfolio = true,
                        "-wp" | "-pw" => {
                            watchlist = true;
                            portfolio = true;
                        }
                        _ => return Command::Invalid("Invalid flag".to_string()),
                    }
                }

                if !watchlist && !portfolio {
                    watchlist = true;
                    portfolio = true;
                }

                Command::Remove {
                    name,
                    watchlist,
                    portfolio,
                }
            }
            _ => Command::Invalid("Unknown command. Available commands: add, rm".to_string()),
        }
    }
}
