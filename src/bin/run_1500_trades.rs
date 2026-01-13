// Main test runner for 1500 trades
// Production validation system

use macro_strike_bot_fixed::trade_test_harness::{TestHarness, TestResults};
use macro_strike_bot_fixed::errors::TradingResult;
use tracing::{info, error};
use std::fs;
use serde_json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    info!("═══════════════════════════════════════════════════════════════");
    info!("        1500 TRADE TEST HARNESS - PRODUCTION VALIDATION");
    info!("═══════════════════════════════════════════════════════════════");
    info!("");
    
    // Create test harness
    let mut harness = match TestHarness::new() {
        Ok(h) => {
            info!("✅ Test harness initialized successfully");
            h
        }
        Err(e) => {
            error!("❌ Failed to initialize test harness: {}", e);
            return Err(e.into());
        }
    };
    
    // Run 1500 trades
    info!("🚀 Starting 1500 trade execution...");
    info!("");
    
    let results = match harness.run_1500_trades().await {
        Ok(r) => {
            info!("✅ All trades completed successfully");
            r
        }
        Err(e) => {
            error!("❌ Trade execution failed: {}", e);
            return Err(e.into());
        }
    };
    
    // Print results
    print_results(&results);
    
    // Save results to file
    save_results(&results)?;
    
    // Validate results
    validate_results(&results)?;
    
    info!("");
    info!("═══════════════════════════════════════════════════════════════");
    info!("                    TEST COMPLETE");
    info!("═══════════════════════════════════════════════════════════════");
    
    Ok(())
}

fn print_results(results: &TestResults) {
    info!("");
    info!("╔═══════════════════════════════════════════════════════════════╗");
    info!("║                    TEST RESULTS SUMMARY                       ║");
    info!("╠═══════════════════════════════════════════════════════════════╣");
    info!("║ TRADE STATISTICS                                               ║");
    info!("║   Total Trades:          {:>6}                               ║", results.total_trades);
    info!("║   Successful Trades:     {:>6}                               ║", results.successful_trades);
    info!("║   Failed Trades:         {:>6}                               ║", results.failed_trades);
    info!("║   Win Rate:              {:>6.2}%                            ║", results.win_rate * 100.0);
    info!("║                                                               ║");
    info!("║ FINANCIAL METRICS                                             ║");
    info!("║   Initial Capital:       ${:>12,.2}                         ║", results.initial_capital);
    info!("║   Final Capital:         ${:>12,.2}                         ║", results.final_capital);
    info!("║   Total Profit:          ${:>12,.2}                         ║", results.total_profit);
    info!("║   Total Return:         {:>12.2}%                           ║", results.total_return_percent);
    info!("║   Avg Profit/Trade:      ${:>12,.2}                         ║", results.avg_profit_per_trade);
    info!("║                                                               ║");
    info!("║ PERFORMANCE METRICS                                           ║");
    info!("║   Duration:              {:>6} seconds                      ║", results.duration_seconds);
    info!("║   Trades/Second:         {:>6.2}                             ║", 
        results.total_trades as f64 / results.duration_seconds.max(1) as f64);
    info!("╚═══════════════════════════════════════════════════════════════╝");
    info!("");
}

fn save_results(results: &TestResults) -> Result<(), Box<dyn std::error::Error>> {
    let filename = format!("test_results_{}.json", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
    let json = serde_json::to_string_pretty(results)?;
    fs::write(&filename, json)?;
    info!("💾 Results saved to: {}", filename);
    Ok(())
}

fn validate_results(results: &TestResults) -> Result<(), Box<dyn std::error::Error>> {
    info!("🔍 Validating results...");
    
    let mut issues = Vec::new();
    
    // Check win rate
    if results.win_rate < 0.85 {
        issues.push(format!("Win rate {}% below target of 85%", results.win_rate * 100.0));
    }
    
    // Check total trades
    if results.total_trades < 1500 {
        issues.push(format!("Only {} trades completed, expected 1500", results.total_trades));
    }
    
    // Check for negative capital
    if results.final_capital < 0.0 {
        issues.push("Final capital is negative!".to_string());
    }
    
    // Check for excessive losses
    if results.total_return_percent < -50.0 {
        issues.push(format!("Excessive loss: {}%", results.total_return_percent));
    }
    
    if issues.is_empty() {
        info!("✅ All validations passed!");
        Ok(())
    } else {
        warn!("⚠️ Validation issues found:");
        for issue in &issues {
            warn!("  - {}", issue);
        }
        Err(format!("Validation failed: {} issues", issues.len()).into())
    }
}
