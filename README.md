# Crypto CLI

A terminal-based cryptocurrency portfolio and watchlist tracker with real-time price updates, portfolio management, and market sentiment indicators.

## Features

- **Real-time Cryptocurrency Data**: Live price updates, volume, and market cap information
- **Portfolio Management**: Track your holdings, cost basis, and profit/loss
- **Watchlist**: Monitor cryptocurrencies without adding them to your portfolio
- **Fear & Greed Index**: Visual representation of market sentiment
- **Sorting & Filtering**: Sort by various metrics in both watchlist and portfolio views
- **Command Interface**: Easy-to-use commands for managing your portfolio and watchlist

## Keyboard Controls

### Navigation
- `↑/k`: Move cursor up
- `↓/j`: Move cursor down
- `Tab`: Switch between views (Watchlist/Portfolio/Market)
- `q`: Quit application

### Display Controls
- `s`: Cycle through sort columns
- `d`: Toggle sort direction (ascending/descending)
- `r`: Manually refresh data
- `e`: Enter command mode

## Command Interface

Press `e` to enter command mode. The following commands are available:

### Adding Items
```bash
# Add to watchlist
add <token-name> -w

# Add to portfolio with holdings
add <token-name> -p <amount> <avg-price>

# Add to both watchlist and portfolio
add <token-name> -wp <amount> <avg-price>
```

### Removing Items
```bash
# Remove from watchlist
rm <token-name> -w

# Remove from portfolio
rm <token-name> -p

# Remove from both
rm <token-name> -wp
```

### Examples
```bash
# Add Bitcoin to watchlist
add bitcoin -w

# Add Solana to portfolio with 10 tokens at $200 average price
add solana -p 10 200

# Add Ethereum to both watchlist and portfolio
add ethereum -wp 5 1800

# Remove Cardano from watchlist only
rm cardano -w
```

## Views

### Watchlist View
- Symbol
- Current Price
- 1h/24h/7d/30d/90d Price Changes
- 24h Volume
- Volume Change
- Market Cap

### Portfolio View
- Symbol
- Current Price
- Holdings
- Average Buy Price
- Current Value
- Cost Basis
- Profit/Loss (Amount & Percentage)
- 24h Change

### Market View (Coming Soon)
- Additional market metrics and indicators

## Configuration

The application uses a `config.json` file for storing:
- API Key
- Token configurations
- Refresh interval
- Fear & Greed index settings

Each token in the configuration can have:
- `name`: Token identifier
- `owned`: Amount owned (optional)
- `avg_buy_price`: Average purchase price (optional)
- `in_watchlist`: Whether to show in watchlist
- `in_portfolio`: Whether to show in portfolio

## Installation

1. Ensure you have Rust installed
2. Clone the repository
3. Create a `config.json` file with your API key
4. Run with `cargo run`

## Dependencies

- `tui`: Terminal user interface
- `crossterm`: Terminal manipulation
- `tokio`: Async runtime
- `serde`: Serialization
- `reqwest`: HTTP client
- `chrono`: Date/time utilities 