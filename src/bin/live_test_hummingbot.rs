// Live Test Binary for Hummingbot Array System
// Tests the updated system with 7-day cycle, volume-based striking, and 1-minute exits

use std::time::{Duration, Instant};
use macro_strike_bot_fixed::hummingbot_array_system::*;
use tokio::time::sleep;
use chrono::Utc;

#[tokio::main]
async fn main() {
    println!("\n╔═══════════════════════════════════════════════════════════════╗");
    println!("║        HUMMINGBOT ARRAY LIVE TEST - PROFIT MODELING           ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!("\n📊 Test Configuration:");
    println!("   • Cycle Duration: 7 days (changed from 14)");
    println!("   • Volume-Based Striking: 2x+ volume spikes");
    println!("   • Rug Pull Protection: ENABLED");
    println!("   • Max Position Time: 1 minute (NO HODL)");
    println!("   • Immediate Exit on Win: YES");
    println!("   • Target: 200% return every 7 days");
    println!("\n🚀 Starting live test simulation...\n");
    
    let start_time = Instant::now();
    let test_duration = Duration::from_secs(300); // 5 minutes of simulation
    let mut array = HummingbotArray::new().await;
    
    // Track metrics
    let mut total_trades = 0u32;
    let mut profitable_trades = 0u32;
    let mut total_profit = 0.0;
    let mut total_loss = 0.0;
    let mut position_times = Vec::new();
    let mut leverage_used = Vec::new();
    let mut volume_ratios = Vec::new();
    
    let mut last_report_time = Instant::now();
    let report_interval = Duration::from_secs(30); // Report every 30 seconds
    
    println!("⏱️  Running simulation for {} seconds...\n", test_duration.as_secs());
    
    let mut iteration = 0;
    while start_time.elapsed() < test_duration {
        iteration += 1;
        
        // Execute one cycle
        let cycle_start = Instant::now();
        
        // Scan markets
        let opportunities = array.scan_all_markets().await;
        
        if !opportunities.is_empty() {
            println!("📈 Found {} opportunities (Iteration {})", opportunities.len(), iteration);
            
            // Assign to bots
            let assignments = array.strike_coordinator.assign_targets(&opportunities, NUM_BOTS).await;
            
            // Execute strikes
            let mut handles = Vec::new();
            for (i, bot) in array.bots.iter().enumerate() {
                if let Some(target) = assignments.get(&i) {
                    let bot_clone = bot.clone();
                    let target_clone = target.clone();
                    
                    let handle = tokio::spawn(async move {
                        let mut bot_guard = bot_clone.lock().await;
                        bot_guard.execute_strike(target_clone).await
                    });
                    
                    handles.push(handle);
                }
            }
            
            // Collect results
            let mut cycle_results = Vec::new();
            for handle in handles {
                if let Ok(result) = handle.await {
                    cycle_results.push(result);
                }
            }
            
            // Process results
            for result in &cycle_results {
                total_trades += 1;
                
                if result.success {
                    profitable_trades += 1;
                    total_profit += result.profit;
                } else {
                    total_loss += result.profit.abs();
                }
                
                // Track metrics
                leverage_used.push(result.position.leverage);
                volume_ratios.push(result.opportunity.volume_ratio);
                
                // Calculate position duration
                if let Some(closed_at) = result.position.closed_at {
                    let duration = closed_at - result.position.opened_at;
                    position_times.push(duration.num_seconds());
                }
            }
            
            // Aggregate
            array.aggregate_cycle_results(cycle_results).await;
            
            let cycle_time = cycle_start.elapsed();
            println!("   ✅ Cycle completed in {:.2}ms | Trades: {} | Profit: ${:.2}", 
                cycle_time.as_millis(), total_trades, array.cycle_profits);
        }
        
        // Periodic reports
        if last_report_time.elapsed() >= report_interval {
            print_performance_report(
                &array,
                total_trades,
                profitable_trades,
                total_profit,
                total_loss,
                &position_times,
                &leverage_used,
                &volume_ratios,
                start_time.elapsed(),
            ).await;
            last_report_time = Instant::now();
        }
        
        // Small delay to prevent tight loop
        sleep(Duration::from_millis(100)).await;
    }
    
    // Final report
    println!("\n╔═══════════════════════════════════════════════════════════════╗");
    println!("║                    FINAL TEST RESULTS                        ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");
    
    print_final_report(
        &array,
        total_trades,
        profitable_trades,
        total_profit,
        total_loss,
        &position_times,
        &leverage_used,
        &volume_ratios,
        start_time.elapsed(),
    ).await;
}

async fn print_performance_report(
    array: &HummingbotArray,
    total_trades: u32,
    profitable_trades: u32,
    total_profit: f64,
    total_loss: f64,
    position_times: &[i64],
    leverage_used: &[f64],
    volume_ratios: &[f64],
    elapsed: Duration,
) {
    let win_rate = if total_trades > 0 {
        profitable_trades as f64 / total_trades as f64
    } else {
        0.0
    };
    
    let avg_position_time = if !position_times.is_empty() {
        position_times.iter().sum::<i64>() as f64 / position_times.len() as f64
    } else {
        0.0
    };
    
    let avg_leverage = if !leverage_used.is_empty() {
        leverage_used.iter().sum::<f64>() / leverage_used.len() as f64
    } else {
        0.0
    };
    
    let avg_volume_ratio = if !volume_ratios.is_empty() {
        volume_ratios.iter().sum::<f64>() / volume_ratios.len() as f64
    } else {
        0.0
    };
    
    println!("\n╔═══════════════════════════════════════════════════════════════╗");
    println!("║              INTERIM PERFORMANCE REPORT                      ║");
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!("║ Runtime:            {:.1} seconds                            ║", elapsed.as_secs_f64());
    println!("║ Total Trades:       {:>6}                                    ║", total_trades);
    println!("║ Profitable:         {:>6}                                    ║", profitable_trades);
    println!("║ Win Rate:           {:.1}%                                   ║", win_rate * 100.0);
    println!("║                                                               ║");
    println!("║ Total Profit:       ${:>12.2}                         ║", total_profit);
    println!("║ Total Loss:         ${:>12.2}                         ║", total_loss);
    println!("║ Net P&L:            ${:>12.2}                         ║", total_profit - total_loss);
    println!("║                                                               ║");
    println!("║ Avg Position Time:  {:.1} seconds                            ║", avg_position_time);
    println!("║ Avg Leverage:       {:.2}x                                    ║", avg_leverage);
    println!("║ Avg Volume Ratio:   {:.2}x                                    ║", avg_volume_ratio);
    println!("║                                                               ║");
    println!("║ Current Capital:    ${:>12.2}                         ║", array.total_capital);
    println!("║ Return:             {:.2}%                                    ║", 
        ((array.total_capital - INITIAL_CAPITAL) / INITIAL_CAPITAL) * 100.0);
    println!("╚═══════════════════════════════════════════════════════════════╝\n");
}

async fn print_final_report(
    array: &HummingbotArray,
    total_trades: u32,
    profitable_trades: u32,
    total_profit: f64,
    total_loss: f64,
    position_times: &[i64],
    leverage_used: &[f64],
    volume_ratios: &[f64],
    elapsed: Duration,
) {
    let win_rate = if total_trades > 0 {
        profitable_trades as f64 / total_trades as f64
    } else {
        0.0
    };
    
    let avg_position_time = if !position_times.is_empty() {
        position_times.iter().sum::<i64>() as f64 / position_times.len() as f64
    } else {
        0.0
    };
    
    let min_position_time = position_times.iter().min().copied().unwrap_or(0);
    let max_position_time = position_times.iter().max().copied().unwrap_or(0);
    
    let avg_leverage = if !leverage_used.is_empty() {
        leverage_used.iter().sum::<f64>() / leverage_used.len() as f64
    } else {
        0.0
    };
    
    let max_leverage = leverage_used.iter().fold(0.0, |a, &b| a.max(b));
    
    let avg_volume_ratio = if !volume_ratios.is_empty() {
        volume_ratios.iter().sum::<f64>() / volume_ratios.len() as f64
    } else {
        0.0
    };
    
    let max_volume_ratio = volume_ratios.iter().fold(0.0, |a, &b| a.max(b));
    
    let net_pnl = total_profit - total_loss;
    let return_pct = ((array.total_capital - INITIAL_CAPITAL) / INITIAL_CAPITAL) * 100.0;
    
    // Project 7-day return
    let elapsed_days = elapsed.as_secs_f64() / 86400.0;
    let daily_return = if elapsed_days > 0.0 {
        return_pct / elapsed_days
    } else {
        0.0
    };
    let projected_7_day_return = daily_return * 7.0;
    let projected_capital = INITIAL_CAPITAL * (1.0 + projected_7_day_return / 100.0);
    
    println!("📊 TRADE STATISTICS");
    println!("   Total Trades:        {}", total_trades);
    println!("   Profitable Trades:   {} ({:.1}%)", profitable_trades, win_rate * 100.0);
    println!("   Total Profit:        ${:.2}", total_profit);
    println!("   Total Loss:          ${:.2}", total_loss);
    println!("   Net P&L:             ${:.2}", net_pnl);
    println!("   Profit Factor:       {:.2}", if total_loss > 0.0 { total_profit / total_loss } else { 0.0 });
    
    println!("\n⏱️  POSITION TIMING");
    println!("   Average Hold Time:   {:.1} seconds", avg_position_time);
    println!("   Min Hold Time:       {} seconds", min_position_time);
    println!("   Max Hold Time:       {} seconds", max_position_time);
    println!("   Positions < 1 min:   {} ({:.1}%)", 
        position_times.iter().filter(|&&t| t < 60).count(),
        if !position_times.is_empty() {
            position_times.iter().filter(|&&t| t < 60).count() as f64 / position_times.len() as f64 * 100.0
        } else { 0.0 });
    
    println!("\n📈 LEVERAGE METRICS");
    println!("   Average Leverage:    {:.2}x", avg_leverage);
    println!("   Max Leverage:        {:.2}x", max_leverage);
    println!("   Leverage Range:      3.0x - 5.0x");
    
    println!("\n📊 VOLUME METRICS");
    println!("   Average Volume Ratio: {:.2}x", avg_volume_ratio);
    println!("   Max Volume Ratio:     {:.2}x", max_volume_ratio);
    println!("   Min Required:         2.0x");
    
    println!("\n💰 CAPITAL PERFORMANCE");
    println!("   Starting Capital:    ${:.2}", INITIAL_CAPITAL);
    println!("   Current Capital:      ${:.2}", array.total_capital);
    println!("   Total Return:         {:.2}%", return_pct);
    println!("   Test Duration:        {:.1} seconds ({:.3} days)", elapsed.as_secs_f64(), elapsed_days);
    
    println!("\n🎯 7-DAY PROJECTION");
    println!("   Daily Return Rate:    {:.2}%", daily_return);
    println!("   Projected 7-Day:      {:.2}%", projected_7_day_return);
    println!("   Projected Capital:    ${:.2}", projected_capital);
    println!("   Target (200%):        ${:.2}", INITIAL_CAPITAL * 3.0);
    println!("   On Track:             {}", 
        if projected_7_day_return >= 200.0 { "✅ YES" } else { "⚠️  NEEDS IMPROVEMENT" });
    
    println!("\n✅ Test completed successfully!");
}

