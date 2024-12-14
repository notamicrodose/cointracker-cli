use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Alignment},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, Table, Paragraph, Chart, Dataset, Axis, GraphType, Tabs, List, ListItem, Wrap},
    Frame,
    symbols,
    text::{Span, Spans},
};
use crossterm::style::Stylize;

use crate::app::state::{App, SortColumn, InputMode};
use crate::utils::formatters::{format_volume, format_market_cap, format_price};

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let size = f.size();
    
    // Calculate dynamic constraints based on terminal height
    let chart_height = if size.height < 20 {
        // For very small terminals, minimize the chart
        20
    } else if size.height < 40 {
        // For medium-sized terminals, use percentage
        30
    } else {
        // For large terminals, cap the absolute height
        35
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            if app.input_mode == InputMode::Editing {
                vec![
                    Constraint::Length(3),                    // Tab bar (fixed)
                    Constraint::Percentage(chart_height),     // Chart area (dynamic)
                    Constraint::Min(10),                      // Content area (flexible)
                    Constraint::Length(3),                    // Help text (fixed)
                    Constraint::Length(3),                    // Input field (fixed)
                ]
            } else {
                vec![
                    Constraint::Length(3),                    // Tab bar (fixed)
                    Constraint::Percentage(chart_height),     // Chart area (dynamic)
                    Constraint::Min(10),                      // Content area (flexible)
                    Constraint::Length(3),                    // Help text (fixed)
                ]
            }
        )
        .split(size);

    draw_tabs(f, app, chunks[0]);
    
    // Draw different charts based on the current tab
    match app.tab_index {
        0 => draw_fear_greed_chart(f, app, chunks[1]),  // Watchlist tab shows Fear & Greed
        1 => draw_portfolio_summary(f, app, chunks[1]),  // Portfolio tab shows portfolio summary
        _ => {}  // Market tab might show something else in the future
    }
    
    match app.tab_index {
        0 => draw_watchlist(f, app, chunks[2]),
        1 => draw_portfolio(f, app, chunks[2]),
        2 => draw_market(f, chunks[2]),
        _ => unreachable!(),
    }

    draw_help(f, app, chunks[3]);
    
    if app.input_mode == InputMode::Editing {
        draw_input(f, app, chunks[4]);
    }
}

fn draw_tabs<B: Backend>(f: &mut Frame<B>, app: &App, area: tui::layout::Rect) {
    let titles = vec!["Watchlist", "Portfolio", "Market"]
        .iter()
        .map(|t| Spans::from(Span::styled(
            *t,
            Style::default().fg(Color::White)
        )))
        .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD))
        .select(app.tab_index);

    f.render_widget(tabs, area);
}

fn draw_fear_greed_chart<B: Backend>(f: &mut Frame<B>, app: &App, area: tui::layout::Rect) {
    let fear_greed_points: Vec<(f64, f64)> = app.fear_greed_data.iter()
        .rev()  // Reverse to get oldest first
        .enumerate()
        .map(|(i, fg)| {
            (i as f64, fg.value as f64)
        })
        .collect();

    // Calculate trend and stats
    let current_value = app.fear_greed_data.first()
        .map(|fg| fg.value)
        .unwrap_or(0);
    let previous_value = app.fear_greed_data.get(1)
        .map(|fg| fg.value)
        .unwrap_or(current_value);
    let trend = match current_value.cmp(&previous_value) {
        std::cmp::Ordering::Greater => "↑",
        std::cmp::Ordering::Less => "↓",
        std::cmp::Ordering::Equal => "→",
    };

    let values: Vec<u64> = app.fear_greed_data.iter()
        .map(|fg| fg.value)
        .collect();
    
    let min_value = values.iter().min().copied().unwrap_or(0);
    let max_value = values.iter().max().copied().unwrap_or(0);

    let datasets = vec![
        Dataset::default()
            .name("Fear & Greed")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Yellow))
            .data(&fear_greed_points),
    ];

    let unknown_str = "Unknown".to_string();
    let current_classification = app.fear_greed_data.first()
        .map(|fg| &fg.value_classification)
        .unwrap_or(&unknown_str);
    
    let title = format!(
        "Fear & Greed Index: {} {} ({}) | Min: {} | Max: {}", 
        current_value,
        trend,
        current_classification,
        min_value,
        max_value,
    );

    let chart = Chart::new(datasets)
        .block(Block::default()
            .title(title)
            .borders(Borders::ALL))
        .x_axis(Axis::default()
            .style(Style::default().fg(Color::White))
            .bounds([0.0, fear_greed_points.len() as f64])
            .labels(app.fear_greed_data.iter()
                .rev()
                .enumerate()
                .filter(|(i, _)| {
                    let total = app.fear_greed_data.len();
                    *i == 0 || // Always show first (newest) date
                    *i == total - 1 || // Always show last (oldest) date
                    i % 7 == 0 // Show regular intervals
                })
                .map(|(_, fg)| {
                    let ts = fg.timestamp.parse::<i64>().unwrap_or(0);
                    let date = chrono::DateTime::from_timestamp(ts, 0)
                        .unwrap_or_default()
                        .format("%b %-d")
                        .to_string();
                    Span::styled(
                        date,
                        Style::default().fg(Color::Gray)
                    )
                })
                .collect()))
        .y_axis(Axis::default()
            .style(Style::default().fg(Color::White))
            .bounds([25.0, 100.0])
            .labels(vec![
                "25 Fear",
                "50 Neutral",
                "75 Greed",
                "100 Ex.Greed"
            ]
                .into_iter()
                .map(Span::from)
                .collect()));

    f.render_widget(chart, area);
}

fn draw_watchlist<B: Backend>(f: &mut Frame<B>, app: &mut App, area: tui::layout::Rect) {
    let header_cells = [
        ("Symbol", SortColumn::Symbol),
        ("Price", SortColumn::Price),
        ("Δ 1h %", SortColumn::Change1h),
        ("Δ 24h %", SortColumn::Change24h),
        ("Δ 7d %", SortColumn::Change7d),
        ("Δ 30d %", SortColumn::Change30d),
        ("Δ 90d %", SortColumn::Change90d),
        ("Volume (24h)", SortColumn::Volume24h),
        ("Δ 24h %", SortColumn::VolumeChange),
        ("Market Cap", SortColumn::MarketCap),
    ]
    .iter()
    .map(|(h, col)| {
        let mut text = (*h).to_string();
        if *col == app.sort_column {
            text = format!("{} {}", text, if app.sort_ascending { "↑" } else { "↓" });
        }
        tui::widgets::Cell::from(text).style(
            Style::default()
                .fg(if *col == app.sort_column { Color::Cyan } else { Color::Yellow })
                .add_modifier(Modifier::BOLD),
        )
    });

    let mut sorted_cryptos: Vec<_> = app.crypto_data.values()
        .filter(|crypto| {
            app.config.tokens.iter().any(|token| {
                let config_name = token.name.to_lowercase()
                    .replace("-", " ")
                    .replace("_", " ");
                let crypto_name = crypto.name.to_lowercase()
                    .replace("-", " ")
                    .replace("_", " ");
                token.is_in_watchlist() && config_name == crypto_name
            })
        })
        .collect();

    sorted_cryptos.sort_by(|a, b| {
        let quote_a = a.quote.get("USD").unwrap();
        let quote_b = b.quote.get("USD").unwrap();
        let cmp = match app.sort_column {
            SortColumn::Symbol => a.symbol.cmp(&b.symbol),
            SortColumn::Price => quote_a.price.partial_cmp(&quote_b.price).unwrap_or(std::cmp::Ordering::Equal),
            SortColumn::Change1h => quote_a.percent_change_1h.partial_cmp(&quote_b.percent_change_1h).unwrap_or(std::cmp::Ordering::Equal),
            SortColumn::Change24h => quote_a.percent_change_24h.partial_cmp(&quote_b.percent_change_24h).unwrap_or(std::cmp::Ordering::Equal),
            SortColumn::Change7d => quote_a.percent_change_7d.partial_cmp(&quote_b.percent_change_7d).unwrap_or(std::cmp::Ordering::Equal),
            SortColumn::Change30d => quote_a.percent_change_30d.partial_cmp(&quote_b.percent_change_30d).unwrap_or(std::cmp::Ordering::Equal),
            SortColumn::Change90d => quote_a.percent_change_90d.partial_cmp(&quote_b.percent_change_90d).unwrap_or(std::cmp::Ordering::Equal),
            SortColumn::Volume24h => quote_a.volume_24h.partial_cmp(&quote_b.volume_24h).unwrap_or(std::cmp::Ordering::Equal),
            SortColumn::VolumeChange => quote_a.volume_change_24h.partial_cmp(&quote_b.volume_change_24h).unwrap_or(std::cmp::Ordering::Equal),
            SortColumn::MarketCap => quote_a.market_cap.partial_cmp(&quote_b.market_cap).unwrap_or(std::cmp::Ordering::Equal),
            _ => std::cmp::Ordering::Equal, // Handle portfolio-specific columns
        };
        if app.sort_ascending { cmp } else { cmp.reverse() }
    });

    let rows = sorted_cryptos.iter().enumerate().map(|(i, crypto)| {
        let quote = crypto.quote.get("USD").unwrap_or_else(|| {
            panic!("USD quote not found for {}", crypto.symbol)
        });

        // Style helpers for percentage changes
        let style_change = |value: Option<f64>| {
            match value {
                Some(v) if v >= 0.0 => Style::default().fg(Color::Green),
                Some(_) => Style::default().fg(Color::Red),
                None => Style::default(),
            }
        };

        let mut row = Row::new(vec![
            tui::widgets::Cell::from(crypto.symbol.clone()),
            tui::widgets::Cell::from(format_price(quote.price)),
            tui::widgets::Cell::from(quote.percent_change_1h.map_or("N/A".to_string(), |v| format!("{:+.2}%", v)))
                .style(style_change(quote.percent_change_1h)),
            tui::widgets::Cell::from(quote.percent_change_24h.map_or("N/A".to_string(), |v| format!("{:+.2}%", v)))
                .style(style_change(quote.percent_change_24h)),
            tui::widgets::Cell::from(quote.percent_change_7d.map_or("N/A".to_string(), |v| format!("{:+.2}%", v)))
                .style(style_change(quote.percent_change_7d)),
            tui::widgets::Cell::from(quote.percent_change_30d.map_or("N/A".to_string(), |v| format!("{:+.2}%", v)))
                .style(style_change(quote.percent_change_30d)),
            tui::widgets::Cell::from(quote.percent_change_90d.map_or("N/A".to_string(), |v| format!("{:+.2}%", v)))
                .style(style_change(quote.percent_change_90d)),
            tui::widgets::Cell::from(format_volume(quote.volume_24h)),
            tui::widgets::Cell::from(quote.volume_change_24h.map_or("N/A".to_string(), |v| format!("{:+.2}%", v)))
                .style(style_change(quote.volume_change_24h)),
            tui::widgets::Cell::from(format_market_cap(quote.market_cap)),
        ]);

        // Highlight the selected row
        if let Some(selected) = app.table_state.selected() {
            if selected == i {
                row = row.style(Style::default().add_modifier(Modifier::REVERSED));
            }
        }

        row
    });

    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let title = match (&app.last_update, &app.last_error) {
        (Some(time), None) => format!(
            "Crypto Prices (Last Updated: {})",
            time.format("%H:%M:%S")
        ),
        (_, Some(error)) => format!(
            "Crypto Prices (Error: {})",
            error
        ).red().to_string(),
        (None, None) => "Crypto Prices (Not Updated Yet)".to_string(),
    };

    let table = Table::new(rows)
        .header(header)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(title))
        .widths(&[
            Constraint::Length(8),   // Symbol
            Constraint::Length(14),  // Price
            Constraint::Length(10),  // 1h %
            Constraint::Length(10),  // 24h %
            Constraint::Length(10),  // 7d %
            Constraint::Length(10),  // 30d %
            Constraint::Length(10),  // 90d %
            Constraint::Length(14),  // Volume
            Constraint::Length(12),  // Volume Change
            Constraint::Length(12),  // Market Cap
        ])
        .column_spacing(1);

    f.render_stateful_widget(table, area, &mut app.table_state);
}

fn draw_portfolio<B: Backend>(f: &mut Frame<B>, app: &mut App, area: tui::layout::Rect) {
    // Create layout for the portfolio view
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),     // Portfolio Table
        ])
        .split(area);

    // Calculate portfolio data
    let mut owned_tokens: Vec<_> = app.config.tokens.iter()
        .filter(|token| token.is_in_portfolio())
        .filter_map(|token| {
            app.crypto_data.values()
                .find(|crypto| {
                    let config_name = token.name.to_lowercase()
                        .replace("-", " ")
                        .replace("_", " ");
                    let crypto_name = crypto.name.to_lowercase()
                        .replace("-", " ")
                        .replace("_", " ");
                    config_name == crypto_name
                })
                .map(|crypto| (token, crypto))
        })
        .collect();

    // Sort the portfolio data
    owned_tokens.sort_by(|(token_a, crypto_a), (token_b, crypto_b)| {
        let quote_a = crypto_a.quote.get("USD").unwrap();
        let quote_b = crypto_b.quote.get("USD").unwrap();
        let holdings_a = token_a.owned.unwrap_or(0.0);
        let holdings_b = token_b.owned.unwrap_or(0.0);
        let avg_buy_a = token_a.avg_buy_price.unwrap_or(0.0);
        let avg_buy_b = token_b.avg_buy_price.unwrap_or(0.0);
        let current_value_a = holdings_a * quote_a.price;
        let current_value_b = holdings_b * quote_b.price;
        let cost_basis_a = holdings_a * avg_buy_a;
        let cost_basis_b = holdings_b * avg_buy_b;
        let profit_loss_a = current_value_a - cost_basis_a;
        let profit_loss_b = current_value_b - cost_basis_b;
        let profit_loss_pct_a = if cost_basis_a > 0.0 { (profit_loss_a / cost_basis_a) * 100.0 } else { 0.0 };
        let profit_loss_pct_b = if cost_basis_b > 0.0 { (profit_loss_b / cost_basis_b) * 100.0 } else { 0.0 };

        let cmp = match app.portfolio_sort_column {
            SortColumn::Symbol => crypto_a.symbol.cmp(&crypto_b.symbol),
            SortColumn::Price => quote_a.price.partial_cmp(&quote_b.price).unwrap_or(std::cmp::Ordering::Equal),
            SortColumn::Holdings => holdings_a.partial_cmp(&holdings_b).unwrap_or(std::cmp::Ordering::Equal),
            SortColumn::AvgBuy => avg_buy_a.partial_cmp(&avg_buy_b).unwrap_or(std::cmp::Ordering::Equal),
            SortColumn::CurrentValue => current_value_a.partial_cmp(&current_value_b).unwrap_or(std::cmp::Ordering::Equal),
            SortColumn::CostBasis => cost_basis_a.partial_cmp(&cost_basis_b).unwrap_or(std::cmp::Ordering::Equal),
            SortColumn::ProfitLoss => profit_loss_a.partial_cmp(&profit_loss_b).unwrap_or(std::cmp::Ordering::Equal),
            SortColumn::ProfitLossPercent => profit_loss_pct_a.partial_cmp(&profit_loss_pct_b).unwrap_or(std::cmp::Ordering::Equal),
            SortColumn::Change24h => quote_a.percent_change_24h.partial_cmp(&quote_b.percent_change_24h).unwrap_or(std::cmp::Ordering::Equal),
            _ => std::cmp::Ordering::Equal,
        };
        if app.sort_ascending { cmp } else { cmp.reverse() }
    });

    let total_value: f64 = owned_tokens.iter()
        .map(|(token_config, crypto)| {
            token_config.owned.unwrap_or(0.0) * crypto.quote.get("USD").unwrap().price
        })
        .sum();

    let total_cost: f64 = owned_tokens.iter()
        .map(|(token_config, _)| {
            token_config.owned.unwrap_or(0.0) * token_config.avg_buy_price.unwrap_or(0.0)
        })
        .sum();

    let total_pl = total_value - total_cost;
    let total_pl_pct = if total_cost > 0.0 {
        (total_pl / total_cost) * 100.0
    } else {
        0.0
    };

    // Portfolio Table
    let header_cells = [
        ("Symbol", SortColumn::Symbol),
        ("Price", SortColumn::Price),
        ("Holdings", SortColumn::Holdings),
        ("Avg Buy", SortColumn::AvgBuy),
        ("Current Value", SortColumn::CurrentValue),
        ("Cost Basis", SortColumn::CostBasis),
        ("P/L", SortColumn::ProfitLoss),
        ("P/L %", SortColumn::ProfitLossPercent),
        ("24h Change", SortColumn::Change24h),
    ].iter().map(|(h, col)| {
        let mut text = (*h).to_string();
        if *col == app.portfolio_sort_column {
            text = format!("{} {}", text, if app.sort_ascending { "↑" } else { "↓" });
        }
        tui::widgets::Cell::from(text).style(
            Style::default()
                .fg(if *col == app.portfolio_sort_column { Color::Cyan } else { Color::Yellow })
                .add_modifier(Modifier::BOLD),
        )
    });

    let rows = owned_tokens.iter().enumerate().map(|(i, (token_config, crypto))| {
        let quote = crypto.quote.get("USD").unwrap();
        let holdings = token_config.owned.unwrap_or(0.0);
        let avg_buy = token_config.avg_buy_price.unwrap_or(0.0);
        let current_value = holdings * quote.price;
        let cost_basis = holdings * avg_buy;
        let profit_loss = current_value - cost_basis;
        let profit_loss_pct = if cost_basis > 0.0 {
            (profit_loss / cost_basis) * 100.0
        } else {
            0.0
        };

        let pl_style = if profit_loss >= 0.0 {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Red)
        };

        let mut row = Row::new(vec![
            tui::widgets::Cell::from(crypto.symbol.clone()),
            tui::widgets::Cell::from(format_price(quote.price)),
            tui::widgets::Cell::from(format!("{:.4}", holdings)),
            tui::widgets::Cell::from(format_price(avg_buy)),
            tui::widgets::Cell::from(format_price(current_value)),
            tui::widgets::Cell::from(format_price(cost_basis)),
            tui::widgets::Cell::from(format_price(profit_loss)).style(pl_style),
            tui::widgets::Cell::from(format!("{:+.2}%", profit_loss_pct)).style(pl_style),
            tui::widgets::Cell::from(
                quote.percent_change_24h
                    .map_or("N/A".to_string(), |v| format!("{:+.2}%", v))
            ).style(
                quote.percent_change_24h.map_or(
                    Style::default(),
                    |v| if v >= 0.0 {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Red)
                    }
                )
            ),
        ]);

        // Highlight the selected row
        if let Some(selected) = app.table_state.selected() {
            if selected == i {
                row = row.style(Style::default().add_modifier(Modifier::REVERSED));
            }
        }

        row
    });

    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let title = format!(
        "Portfolio - Total Value: ${:.2} | P/L: ${:.2} ({:+.2}%)",
        total_value, total_pl, total_pl_pct
    );

    let table = Table::new(rows)
        .header(header)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(title))
        .widths(&[
            Constraint::Length(8),   // Symbol
            Constraint::Length(12),  // Price
            Constraint::Length(12),  // Holdings
            Constraint::Length(12),  // Avg Buy
            Constraint::Length(14),  // Current Value
            Constraint::Length(14),  // Cost Basis
            Constraint::Length(12),  // P/L
            Constraint::Length(10),  // P/L %
            Constraint::Length(10),  // 24h Change
        ])
        .column_spacing(1);

    // Render the table
    f.render_stateful_widget(table, chunks[0], &mut app.table_state);
}

fn draw_market<B: Backend>(f: &mut Frame<B>, area: tui::layout::Rect) {
    let market_placeholder = Paragraph::new("Market - Coming Soon!")
        .block(Block::default()
            .title("Market")
            .borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center);
    f.render_widget(market_placeholder, area);
}

fn draw_help<B: Backend>(f: &mut Frame<B>, app: &App, area: tui::layout::Rect) {
    let text = match app.input_mode {
        InputMode::Normal => vec![
            Spans::from(vec![
                Span::styled("q", Style::default().fg(Color::Yellow)),
                Span::raw(": Quit | "),
                Span::styled("↓/j", Style::default().fg(Color::Yellow)),
                Span::raw(" "),
                Span::styled("↑/k", Style::default().fg(Color::Yellow)),
                Span::raw(": Navigate | "),
                Span::styled("Tab", Style::default().fg(Color::Yellow)),
                Span::raw(": Switch View | "),
                Span::styled("s", Style::default().fg(Color::Yellow)),
                Span::raw(": Sort | "),
                Span::styled("d", Style::default().fg(Color::Yellow)),
                Span::raw(": Direction | "),
                Span::styled("r", Style::default().fg(Color::Yellow)),
                Span::raw(": Refresh | "),
                Span::styled("e", Style::default().fg(Color::Yellow)),
                Span::raw(": Edit "),
            ])
        ],
        InputMode::Editing => vec![
            Spans::from(vec![
                Span::styled("Enter", Style::default().fg(Color::Yellow)),
                Span::raw(": Execute Command | "),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::raw(": Cancel"),
            ])
        ],
    };

    let help = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);

    f.render_widget(help, area);
}

fn draw_input<B: Backend>(f: &mut Frame<B>, app: &App, area: tui::layout::Rect) {
    let input = Paragraph::new(app.input.as_ref())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Command Input"));
    
    f.render_widget(input, area);
}

fn draw_portfolio_summary<B: Backend>(f: &mut Frame<B>, app: &App, area: tui::layout::Rect) {
    // Calculate portfolio totals
    let owned_tokens: Vec<_> = app.config.tokens.iter()
        .filter(|token| token.is_in_portfolio())
        .filter_map(|token| {
            app.crypto_data.values()
                .find(|crypto| {
                    let config_name = token.name.to_lowercase()
                        .replace("-", " ")
                        .replace("_", " ");
                    let crypto_name = crypto.name.to_lowercase()
                        .replace("-", " ")
                        .replace("_", " ");
                    config_name == crypto_name
                })
                .map(|crypto| (token, crypto))
        })
        .collect();

    let total_value: f64 = owned_tokens.iter()
        .map(|(token_config, crypto)| {
            token_config.owned.unwrap_or(0.0) * crypto.quote.get("USD").unwrap().price
        })
        .sum();

    let total_cost: f64 = owned_tokens.iter()
        .map(|(token_config, _)| {
            token_config.owned.unwrap_or(0.0) * token_config.avg_buy_price.unwrap_or(0.0)
        })
        .sum();

    let total_pl = total_value - total_cost;
    let total_pl_pct = if total_cost > 0.0 {
        (total_pl / total_cost) * 100.0
    } else {
        0.0
    };

    // Calculate 24h change
    let total_24h_change: f64 = owned_tokens.iter()
        .map(|(token_config, crypto)| {
            let quote = crypto.quote.get("USD").unwrap();
            let holdings = token_config.owned.unwrap_or(0.0);
            let current_value = holdings * quote.price;
            quote.percent_change_24h.unwrap_or(0.0) * current_value / 100.0
        })
        .sum();
    
    let total_24h_change_pct = (total_24h_change / total_value) * 100.0;

    // Create layout for the summary blocks
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(45),     // Metrics needs fixed width for labels
            Constraint::Ratio(1, 2), // Allocation takes remaining space
            Constraint::Min(30),     // Performance needs minimal width
        ])
        .split(area);

    // Metrics Block
    let metrics_text = vec![
        // Net Worth
        Spans::from(vec![
            Span::styled("Net Worth", Style::default().fg(Color::DarkGray)),
            Span::raw("  "),
            Span::styled(
                format!("${:.2}", total_value),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            ),
        ]),
        Spans::from(vec![Span::raw("")]),  // Spacing

        // Profit/Loss with percentage
        Spans::from(vec![
            Span::styled("Profit/Loss", Style::default().fg(Color::DarkGray)),
            Span::raw("  "),
            Span::styled(
                format!("${:.2}", total_pl),
                Style::default()
                    .fg(if total_pl >= 0.0 { Color::Green } else { Color::Red })
                    .add_modifier(Modifier::BOLD)
            ),
            Span::raw("  "),
            Span::styled(
                format!("({:+.2}%)", total_pl_pct),
                Style::default().fg(if total_pl >= 0.0 { Color::Green } else { Color::Red })
            ),
        ]),
        Spans::from(vec![Span::raw("")]),  // Spacing

        // 24h Change with percentage
        Spans::from(vec![
            Span::styled("24h Change", Style::default().fg(Color::DarkGray)),
            Span::raw("  "),
            Span::styled(
                format!("${:.2}", total_24h_change),
                Style::default()
                    .fg(if total_24h_change >= 0.0 { Color::Green } else { Color::Red })
                    .add_modifier(Modifier::BOLD)
            ),
            Span::raw("  "),
            Span::styled(
                format!("({:+.2}%)", total_24h_change_pct),
                Style::default().fg(if total_24h_change >= 0.0 { Color::Green } else { Color::Red })
            ),
        ]),
        Spans::from(vec![Span::raw("")]),  // Spacing

        // Cost Basis
        Spans::from(vec![
            Span::styled("Cost Basis", Style::default().fg(Color::DarkGray)),
            Span::raw("  "),
            Span::styled(
                format!("${:.2}", total_cost),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            ),
        ]),
        Spans::from(vec![Span::raw("")]),  // Spacing

        // Assets Count
        Spans::from(vec![
            Span::styled("Assets", Style::default().fg(Color::DarkGray)),
            Span::raw("  "),
            Span::styled(
                format!("{}", owned_tokens.len()),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            ),
            Span::raw(" "),
            Span::styled(
                if owned_tokens.len() == 1 { "token" } else { "tokens" },
                Style::default().fg(Color::DarkGray)
            ),
        ]),
    ];

    let metrics_block = Paragraph::new(metrics_text)
        .block(Block::default()
            .title(Span::styled(" Portfolio Metrics ", 
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White)))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    // Allocations List
    let mut allocations: Vec<_> = owned_tokens.iter()
        .map(|(token_config, crypto)| {
            let value = token_config.owned.unwrap_or(0.0) * crypto.quote.get("USD").unwrap().price;
            let allocation = (value / total_value) * 100.0;
            (
                crypto.symbol.clone(),
                allocation,
                value
            )
        })
        .collect();

    // Sort by allocation percentage (descending)
    allocations.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Calculate dynamic bar width based on available space
    let available_width = chunks[1].width as usize;
    let bar_width = if available_width > 50 {
        15
    } else if available_width > 40 {
        10
    } else {
        5
    };

    let allocation_items: Vec<ListItem> = allocations.iter()
        .map(|(symbol, percentage, value)| {
            let filled_width = ((percentage * bar_width as f64) / 100.0).round() as usize;
            let empty_width = bar_width - filled_width;
            
            ListItem::new(vec![
                // Main content line
                Spans::from(vec![
                    Span::styled(
                        format!("{:<6}", symbol),  // Reduced symbol width
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    ),
                    Span::raw(" "),
                    Span::styled(
                        format!("{:>4.1}%", percentage),  // Reduced percentage width
                        Style::default().fg(Color::Cyan)
                    ),
                    Span::raw(" "),
                    Span::styled(
                        "█".repeat(filled_width),
                        Style::default().fg(Color::Cyan)
                    ),
                    Span::styled(
                        "░".repeat(empty_width),
                        Style::default().fg(Color::DarkGray)
                    ),
                    Span::raw(" "),
                    Span::styled(
                        format!("${}", value.round() as i64),
                        Style::default().fg(Color::White)
                    ),
                ]),
                // Empty line for spacing
                Spans::from(vec![
                    Span::raw(""),
                ]),
            ])
        })
        .collect();

    let allocations_list = List::new(allocation_items)
        .block(Block::default()
            .title("Portfolio Allocation")
            .borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    // Performance Block (placeholder for now)
    let performance_block = Paragraph::new(vec![
        Spans::from(vec![
            Span::raw("Coming soon:"),
        ]),
        Spans::from(vec![
            Span::raw("- Historical"),
        ]),
        Spans::from(vec![
            Span::raw("- Price alerts"),
        ]),
        Spans::from(vec![
            Span::raw("- Analytics"),
        ]),
    ])
    .block(Block::default()
        .title("Performance")
        .borders(Borders::ALL))
    .alignment(Alignment::Left)
    .wrap(Wrap { trim: true });  // Fixed wrap

    // Render blocks
    f.render_widget(metrics_block, chunks[0]);
    f.render_widget(allocations_list, chunks[1]);
    f.render_widget(performance_block, chunks[2]);
}
