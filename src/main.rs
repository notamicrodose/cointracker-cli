use std::{io, time::Duration};
use tui::{
    backend::CrosstermBackend,
    Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::fs;
use anyhow::Result;
use tokio::sync::mpsc;
use chrono::Local;

mod app;
mod models;
mod services;
mod utils;

use app::state::{App, InputMode, SortColumn};
use app::ui;
use models::config::Config;
use services::logger;

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config_str = fs::read_to_string("config.json")?;
    let config: Config = serde_json::from_str(&config_str)?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let app = App::new(config);
    let res = run_app(&mut terminal, app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        logger::log_error("Application Error", &format!("{:?}", err))?;
        println!("{:?}", err)
    }

    Ok(())
}

async fn run_app<B: tui::backend::Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    let (tx, mut rx) = mpsc::channel(1);
    
    // Fetch Fear & Greed data once at startup
    let app_clone = App::new(app.config.clone());
    if let Ok(fg_data) = app_clone.fetch_fear_greed().await {
        app.fear_greed_data = fg_data;
    }

    // Spawn crypto price fetching task
    let config = app.config.clone();
    tokio::spawn(async move {
        loop {
            let app_clone = App::new(config.clone());
            match app_clone.fetch_prices().await {
                Ok(data) => {
                    let _ = tx.send(data).await;
                },
                Err(e) => logger::log_error("Price Fetch Error", &e.to_string()).unwrap_or(()),
            }
            tokio::time::sleep(Duration::from_secs(config.refresh_interval)).await;
        }
    });

    loop {
        // Check for new price data
        if let Ok(new_data) = rx.try_recv() {
            app.crypto_data = new_data;
            app.last_update = Some(Local::now());
        }

        // Handle input
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match app.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Down | KeyCode::Char('j') => app.next(),
                        KeyCode::Up | KeyCode::Char('k') => app.previous(),
                        KeyCode::Char('r') => {
                            if let Ok(new_data) = app.fetch_prices().await {
                                app.crypto_data = new_data;
                                app.last_update = Some(Local::now());
                            }
                        },
                        KeyCode::Char('d') => {
                            app.sort_ascending = !app.sort_ascending;  // Toggle sort direction
                        },
                        KeyCode::Tab => app.next_tab(),
                        KeyCode::Char('s') => {
                            match app.tab_index {
                                0 => {  // Watchlist tab
                                    // Cycle through watchlist columns
                                    app.sort_column = match app.sort_column {
                                        SortColumn::Symbol => SortColumn::Price,
                                        SortColumn::Price => SortColumn::Change1h,
                                        SortColumn::Change1h => SortColumn::Change24h,
                                        SortColumn::Change24h => SortColumn::Change7d,
                                        SortColumn::Change7d => SortColumn::Change30d,
                                        SortColumn::Change30d => SortColumn::Change90d,
                                        SortColumn::Change90d => SortColumn::Volume24h,
                                        SortColumn::Volume24h => SortColumn::VolumeChange,
                                        SortColumn::VolumeChange => SortColumn::MarketCap,
                                        SortColumn::MarketCap => SortColumn::Symbol,
                                        _ => SortColumn::Symbol,
                                    };
                                },
                                1 => {  // Portfolio tab
                                    // Cycle through portfolio columns
                                    app.portfolio_sort_column = match app.portfolio_sort_column {
                                        SortColumn::Symbol => SortColumn::Price,
                                        SortColumn::Price => SortColumn::Holdings,
                                        SortColumn::Holdings => SortColumn::AvgBuy,
                                        SortColumn::AvgBuy => SortColumn::CurrentValue,
                                        SortColumn::CurrentValue => SortColumn::CostBasis,
                                        SortColumn::CostBasis => SortColumn::ProfitLoss,
                                        SortColumn::ProfitLoss => SortColumn::ProfitLossPercent,
                                        SortColumn::ProfitLossPercent => SortColumn::Change24h,
                                        SortColumn::Change24h => SortColumn::Symbol,
                                        _ => SortColumn::Symbol,
                                    };
                                },
                                _ => {}
                            }
                        },
                        KeyCode::Char('e') => app.enter_edit_mode(),
                        _ => {}
                    },
                    InputMode::Editing => match key.code {
                        KeyCode::Enter => {
                            if let Err(e) = app.process_command().await {
                                app.last_error = Some(format!("Command error: {}", e));
                            }
                            app.exit_edit_mode();
                        }
                        KeyCode::Esc => {
                            app.exit_edit_mode();
                        }
                        KeyCode::Char(c) => {
                            app.input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        _ => {}
                    }
                }
            }
        }

        // Draw UI
        terminal.draw(|f| ui::draw(f, &mut app))?;
    }
} 
