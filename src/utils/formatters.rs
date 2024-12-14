/// Formats a volume value into a human-readable string with appropriate unit (B/M)
/// Returns "N/A" if the volume is None
pub fn format_volume(volume: Option<f64>) -> String {
    volume.map_or("N/A".to_string(), |v| {
        if v >= 1_000_000_000.0 {
            format!("${:.1}B", v / 1_000_000_000.0)
        } else {
            format!("${:.1}M", v / 1_000_000.0)
        }
    })
}

/// Formats a market cap value into a human-readable string with appropriate unit (B/M)
/// Returns "N/A" if the market cap is None
pub fn format_market_cap(market_cap: Option<f64>) -> String {
    market_cap.map_or("N/A".to_string(), |v| {
        if v >= 1_000_000_000.0 {
            format!("${:.1}B", v / 1_000_000_000.0)
        } else {
            format!("${:.1}M", v / 1_000_000.0)
        }
    })
}

/// Formats a price value with appropriate decimal places based on its magnitude
/// - For prices >= 1000: 2 decimal places
/// - For prices >= 1: 3 decimal places
/// - For prices < 1: 6 decimal places
pub fn format_price(price: f64) -> String {
    match price {
        p if p >= 1000.0 => format!("${:.2}", p),
        p if p >= 1.0 => format!("${:.3}", p),
        p => format!("${:.6}", p)
    }
}