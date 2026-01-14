// 5-Year Historical Backtest Runner
// Validates strategy against 5 years of historical data
// Ensures compatibility before moving to live simulation

use macro_strike_bot_fixed::historical_backtest::{HistoricalBacktester, BacktestResult};
use macro_strike_bot_fixed::errors::TradingResult;
use tracing::{info, error};
use tracing_subscriber;

const INITIAL_CAPITAL: f64 = 800_000.0;
const SYMBOLS: &[&str] = &["BTC/USDT", "ETH/USDT", "SOL/USDT", "AVAX/USDT", "MATIC/USDT"];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    info!("╔═══════════════════════════════════════════════════════════════╗");
    info!("║      5-YEAR HISTORICAL BACKTEST - STRATEGY VALIDATION         ║");
    info!("╚═══════════════════════════════════════════════════════════════╝");
    info!("");
    
    // Initialize backtester
    let backtester = HistoricalBacktester::new(None);
    
    // Step 1: Connect to 5-year data node
    info!("Step 1: Connecting to 5-year historical data node...");
    let node_url = std::env::var("HISTORICAL_NODE_URL")
        .unwrap_or_else(|_| "http://localhost:8545".to_string());
    
    if let Err(e) = backtester.connect_to_node(&node_url).await {
        error!("Failed to connect to data node: {}", e);
        return Err(e.into());
    }
    
    // Step 2: Load 5 years of historical data
    info!("Step 2: Loading 5 years of historical data...");
    let symbols: Vec<String> = SYMBOLS.iter().map(|s| s.to_string()).collect();
    let start_date = chrono::Utc::now() - chrono::Duration::days(5 * 365);
    
    if let Err(e) = backtester.load_historical_data(&symbols, start_date).await {
        error!("Failed to load historical data: {}", e);
        return Err(e.into());
    }
    
    // Step 3: Validate data compatibility
    info!("Step 3: Validating data compatibility...");
    let compatibility = match backtester.validate_data_compatibility().await {
        Ok(score) => {
            info!("✅ Data compatibility validated: {:.2}%", score * 100.0);
            score
        }
        Err(e) => {
            error!("❌ Data compatibility validation failed: {}", e);
            return Err(e.into());
        }
    };
    
    // Step 4: Run backtest
    info!("Step 4: Running 5-year historical backtest...");
    info!("Initial Capital: ${:,.2}", INITIAL_CAPITAL);
    info!("");
    
    let result = match backtester.run_backtest(INITIAL_CAPITAL).await {
        Ok(r) => r,
        Err(e) => {
            error!("Backtest failed: {}", e);
            return Err(e.into());
        }
    };
    
    // Step 5: Display results
    print_backtest_results(&result);
    
    // Step 6: Check qualification for live simulation
    info!("");
    info!("═══════════════════════════════════════════════════════════════");
    info!("QUALIFICATION ASSESSMENT");
    info!("═══════════════════════════════════════════════════════════════");
    
    let qualifies = backtester.qualifies_for_live_sim(&result);
    
    if qualifies {
        info!("✅ STRATEGY QUALIFIES FOR LIVE SIMULATION");
        info!("");
        info!("Next Steps:");
        info!("  1. Start live simulation mode");
        info!("  2. Run for minimum {} days", 7);
        info!("  3. Execute minimum {} trades", 1000);
        info!("  4. Maintain {}%+ win rate", 93);
        info!("  5. If successful, proceed to live trading");
    } else {
        info!("❌ STRATEGY DOES NOT QUALIFY FOR LIVE SIMULATION");
        info!("");
        info!("Issues identified:");
        if result.win_rate < 0.93 {
            info!("  - Win rate {:.2}% below 93% threshold", result.win_rate * 100.0);
        }
        if result.max_drawdown > 0.10 {
            info!("  - Max drawdown {:.2}% exceeds 10% limit", result.max_drawdown * 100.0);
        }
        if compatibility < 0.95 {
            info!("  - Compatibility {:.2}% below 95% threshold", compatibility * 100.0);
        }
        if result.data_quality_score < 0.90 {
            info!("  - Data quality {:.2}% below 90% threshold", result.data_quality_score * 100.0);
        }
        if result.sharpe_ratio <= 1.0 {
            info!("  - Sharpe ratio {:.2} below 1.0 threshold", result.sharpe_ratio);
        }
        if result.profit_factor <= 1.5 {
            info!("  - Profit factor {:.2} below 1.5 threshold", result.profit_factor);
        }
    }
    
    info!("");
    info!("═══════════════════════════════════════════════════════════════");
    info!("BACKTEST COMPLETE");
    info!("═══════════════════════════════════════════════════════════════");
    
    Ok(())
}

fn print_backtest_results(result: &BacktestResult) {
    info!("");
    info!("╔═══════════════════════════════════════════════════════════════╗");
    info!("║              5-YEAR BACKTEST RESULTS                         ║");
    info!("╚═══════════════════════════════════════════════════════════════╝");
    info!("");
    
    info!("Period: {} to {}", result.start_date, result.end_date);
    info!("");
    
    info!("═══════════════════════════════════════════════════════════════");
    info!("TRADE METRICS");
    info!("═══════════════════════════════════════════════════════════════");
    info!("Total Trades:        {}", result.total_trades);
    info!("Successful:          {} ({:.2}%)", 
        result.successful_trades, result.win_rate * 100.0);
    info!("Failed:              {} ({:.2}%)", 
        result.failed_trades, (1.0 - result.win_rate) * 100.0);
    info!("Win Rate:            {:.2}%", result.win_rate * 100.0);
    info!("");
    
    info!("═══════════════════════════════════════════════════════════════");
    info!("FINANCIAL METRICS");
    info!("═══════════════════════════════════════════════════════════════");
    info!("Total Profit:        ${:,.2}", result.total_profit);
    info!("Total Loss:          ${:,.2}", result.total_loss);
    info!("Net Profit:          ${:,.2}", result.net_profit);
    info!("Max Drawdown:        {:.2}%", result.max_drawdown * 100.0);
    info!("Profit Factor:       {:.2}", result.profit_factor);
    info!("Recovery Factor:     {:.2}", result.recovery_factor);
    info!("");
    
    info!("═══════════════════════════════════════════════════════════════");
    info!("RISK METRICS");
    info!("═══════════════════════════════════════════════════════════════");
    info!("Sharpe Ratio:        {:.2}", result.sharpe_ratio);
    info!("Sortino Ratio:       {:.2}", result.sortino_ratio);
    info!("Average Trade Duration: {:.2} minutes", result.average_trade_duration);
    info!("");
    
    info!("═══════════════════════════════════════════════════════════════");
    info!("DATA QUALITY");
    info!("═══════════════════════════════════════════════════════════════");
    info!("Data Quality Score:  {:.2}%", result.data_quality_score * 100.0);
    info!("Compatibility Score: {:.2}%", result.compatibility_score * 100.0);
}

