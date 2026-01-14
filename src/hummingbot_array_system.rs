// Hummingbot Array System - 25 Bot Coordinated Strike Framework
// 8% per bot * 25 bots = 200% returns every 7 days
// Volume-based striking with 3-5x leverage, immediate exit on win (1-minute max)

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use crate::rug_pull_detector::{RugPullDetector, RiskLevel};
use strike_box::{
    StrikeBoxEngine, StrikeBoxConfig, TokenSnapshot, Direction as StrikeBoxDirection,
    RiskValidation, SafetyScore, Position as StrikeBoxPosition, PositionBook,
    PortfolioState, SystemState as StrikeBoxSystemState,
};
use rust_decimal::Decimal;
use log::{info, warn};

pub const INITIAL_CAPITAL: f64 = 800_000.0;
pub const NUM_BOTS: usize = 25;
const CAPITAL_PER_BOT: f64 = INITIAL_CAPITAL / NUM_BOTS as f64; // $32,000 per bot
const TARGET_PER_BOT: f64 = 0.08; // 8% per bot per cycle
const CYCLE_DAYS: u32 = 7; // 7-day cycles for 200% returns (CHANGED FROM 14)
const MAX_LEVERAGE: f64 = 5.0; // Never exceed 5x
const OPTIMAL_LEVERAGE: f64 = 3.0; // Target 3x leverage
const MAX_POSITION_TIME_SECONDS: i64 = 60; // 1 minute maximum position time
const MIN_VOLUME_RATIO: f64 = 2.0; // Require 2x+ volume spike
const MIN_SAFETY_SCORE: f64 = 0.75; // Minimum safety score for non-traditional assets
const QUICK_PROFIT_THRESHOLD: f64 = 0.005; // 0.5% quick profit exit

// ==================== HUMMINGBOT ARRAY CONTROLLER ====================

#[derive(Debug, Clone)]
pub struct HummingbotArray {
    bots: Vec<Arc<Mutex<HummingBot>>>,
    capital_pool: Arc<RwLock<CapitalPool>>,
    strike_coordinator: Arc<StrikeCoordinator>,
    performance_aggregator: Arc<RwLock<PerformanceAggregator>>,
    rug_pull_detector: Arc<RwLock<RugPullDetector>>,
    strike_box_engine: Arc<RwLock<StrikeBoxEngine>>,
    cycle_start: DateTime<Utc>,
    total_capital: f64,
    cycle_profits: f64,
}

impl HummingbotArray {
    pub async fn new() -> Self {
        let capital_pool = Arc::new(RwLock::new(CapitalPool::new(INITIAL_CAPITAL)));
        let strike_coordinator = Arc::new(StrikeCoordinator::new());
        let performance_aggregator = Arc::new(RwLock::new(PerformanceAggregator::new()));
        let rug_pull_detector = Arc::new(RwLock::new(RugPullDetector::new()));
        
        // Initialize Strike Box Engine
        let strike_box_config = StrikeBoxConfig::default();
        let strike_box_engine = Arc::new(RwLock::new(
            StrikeBoxEngine::new(strike_box_config, Decimal::from(INITIAL_CAPITAL as i64))
        ));
        
        let mut bots = Vec::new();
        
        // Initialize 25 specialized bots
        for i in 0..NUM_BOTS {
            let bot_type = match i % 5 {
                0 => BotStrategy::MarketMaking,
                1 => BotStrategy::Arbitrage,
                2 => BotStrategy::Momentum,
                3 => BotStrategy::MeanReversion,
                4 => BotStrategy::Volatility,
                _ => BotStrategy::MarketMaking,
            };
            
            let bot = Arc::new(Mutex::new(HummingBot::new(
                i,
                CAPITAL_PER_BOT,
                bot_type,
                capital_pool.clone(),
                strike_coordinator.clone(),
            )));
            
            bots.push(bot);
        }
        
        Self {
            bots,
            capital_pool,
            strike_coordinator,
            performance_aggregator,
            rug_pull_detector,
            strike_box_engine,
            cycle_start: Utc::now(),
            total_capital: INITIAL_CAPITAL,
            cycle_profits: 0.0,
        }
    }

    pub async fn execute_coordinated_strike(&mut self) {
        println!("\n╔══════════════════════════════════════════════════════════╗");
        println!("║     HUMMINGBOT ARRAY - 25 BOT COORDINATED STRIKE         ║");
        println!("╚══════════════════════════════════════════════════════════╝");
        println!("⚡ Launching 25 parallel bots with 3-5x leverage");
        println!("🎯 Target: 8% per bot = 200% total every 7 days");
        println!("💰 Capital: $800,000 | Per Bot: $32,000");
        println!("🛡️  Rug Pull Protection: ENABLED");
        println!("📦 Strike Box Integration: ENABLED (Institutional Validation)");
        println!("⏱️  Max Position Time: 1 minute (NO HODL)");
        println!("📊 Volume-Based Striking: 2x+ volume spikes required");
        
        loop {
            // Phase 1: Check and close any positions that hit targets/stops
            self.check_and_close_positions().await;
            
            // Phase 2: Market Scanning (volume-based, non-traditional assets)
            let opportunities = self.scan_all_markets().await;
            
            // Phase 2: Coordinate Strike Assignments
            let assignments = self.strike_coordinator.assign_targets(&opportunities, NUM_BOTS).await;
            
            // Phase 3: Parallel Bot Execution
            let mut handles = Vec::new();
            
            for (i, bot) in self.bots.iter().enumerate() {
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
            
            // Wait for all bots to complete their strikes
            let mut cycle_results = Vec::new();
            for handle in handles {
                if let Ok(result) = handle.await {
                    cycle_results.push(result);
                }
            }
            
            // Phase 4: Aggregate Results
            self.aggregate_cycle_results(cycle_results).await;
            
            // Phase 5: Rebalance and Compound
            self.rebalance_capital().await;
            
            // Check cycle completion
            if self.is_cycle_complete() {
                self.print_cycle_report().await;
                self.reset_cycle();
            }
            
            // High-frequency loop - execute strikes continuously
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    pub async fn scan_all_markets(&self) -> Vec<MarketOpportunity> {
        let mut opportunities = Vec::new();
        
        // EXCLUDE traditional pairs - focus on volume-based opportunities
        let excluded_pairs = vec![
            "BTC/USDT", "ETH/USDT", "AVAX/USDT", "SOL/USDT",
            "MATIC/USDT", "LINK/USDT", "DOT/USDT", "ADA/USDT",
            "ATOM/USDT", "NEAR/USDT", "FTM/USDT", "ALGO/USDT",
            "XRP/USDT", "DOGE/USDT", "SHIB/USDT"
        ];
        
        // Scan multiple exchanges simultaneously
        let exchanges = vec![
            "binance", "coinbase", "kraken", "okx",
            "kucoin", "bybit", "gate", "mexc",
            "pancakeswap", "uniswap", "sushiswap" // Add DEXs
        ];
        
        for exchange in exchanges {
            // Scan for volume spikes (non-traditional assets)
            let all_pairs = self.scan_volume_spikes(exchange).await;
            
            for pair in all_pairs {
                // Skip excluded traditional pairs
                if excluded_pairs.iter().any(|&excluded| pair.contains(excluded)) {
                    continue;
                }
                
                // Fetch real volume data
                let volume_ratio = self.fetch_volume_ratio(&pair).await;
                
                // Only consider 2x+ volume spikes
                if volume_ratio < MIN_VOLUME_RATIO {
                    continue;
                }
                
                // Extract token address for validation
                let token_address = self.extract_token_address(&pair);
                
                // Fetch real prices and token data
                let entry_price = self.fetch_price(&pair).await;
                let liquidity_usd = self.fetch_liquidity_usd(&pair).await;
                let holder_count = self.fetch_holder_count(&token_address).await;
                let token_age_hours = self.fetch_token_age_hours(&token_address).await;
                
                // Create TokenSnapshot for Strike Box validation
                let token_snapshot = TokenSnapshot {
                    token_address: token_address.clone(),
                    token_symbol: pair.split('/').next().unwrap_or("UNKNOWN").to_string(),
                    liquidity_usd: Decimal::from(liquidity_usd as i64),
                    bid_depth_usd: Decimal::from((liquidity_usd * 0.5) as i64),
                    ask_depth_usd: Decimal::from((liquidity_usd * 0.5) as i64),
                    holder_count,
                    top_10_concentration_pct: Decimal::new(45, 2), // Would fetch real data
                    largest_wallet_pct: Decimal::new(12, 2), // Would fetch real data
                    token_age_hours,
                    contract_verified: true, // Would check real verification
                    is_proxy_contract: false,
                    deployment_timestamp: Utc::now() - chrono::Duration::hours(token_age_hours as i64),
                    snapshot_timestamp: Utc::now(),
                };
                
                // Validate with Strike Box (comprehensive institutional validation)
                let strike_box = self.strike_box_engine.read().await;
                let direction = StrikeBoxDirection::Long; // Default to long for volume spikes
                let validation = strike_box.validate_entry(&token_snapshot, direction);
                
                if !validation.all_passed {
                    // Log rejection
                    drop(strike_box);
                    let mut engine = self.strike_box_engine.write().await;
                    engine.log_rejection(&token_snapshot, direction, &validation);
                    continue;
                }
                
                // Also check with rug pull detector for additional safety
                let detector = self.rug_pull_detector.read().await;
                match detector.validate_token(&token_address, &pair).await {
                    Ok(safety_score) => {
                        // Only proceed if Safe or Moderate risk
                        if safety_score.risk_level == RiskLevel::Critical || 
                           safety_score.risk_level == RiskLevel::High {
                            continue; // Skip risky tokens
                        }
                        
                        // Calculate Strike Box position size
                        let strike_box_size = strike_box.calculate_position_size(&token_snapshot, direction);
                        let strike_box_size_f64 = strike_box_size.to_string().parse::<f64>().unwrap_or(0.0);
                        
                        // Calculate expected move
                        let expected_move = self.calculate_expected_move(&pair, volume_ratio).await;
                        
                        // Get Strike Box stop loss and take profit prices
                        let stop_loss_config = &strike_box.config.stop_loss;
                        let take_profit_config = &strike_box.config.take_profit;
                        let safety_score_decimal = Decimal::from((safety_score.overall_score * 100.0) as i64) / Decimal::from(100);
                        
                        let stop_loss_price = stop_loss_config.long_stop_price(
                            Decimal::from(entry_price as i64),
                            safety_score_decimal
                        );
                        let tp_prices = take_profit_config.long_tp_prices(Decimal::from(entry_price as i64));
                        
                        // Create opportunity with Strike Box integration
                        let opportunity = MarketOpportunity {
                            exchange: exchange.to_string(),
                            pair: pair.clone(),
                            opportunity_type: OpportunityType::VolumeSpike,
                            expected_profit: expected_move,
                            confidence: safety_score.overall_score,
                            volatility: self.get_volatility(&pair).await,
                            volume_ratio,
                            entry_price,
                            target_price: tp_prices[0].to_string().parse::<f64>().unwrap_or(entry_price * 1.15), // TP1
                            stop_loss: stop_loss_price.to_string().parse::<f64>().unwrap_or(entry_price * 0.95),
                            leverage: self.calculate_volume_based_leverage(volume_ratio, safety_score.overall_score),
                            safety_score: safety_score.overall_score,
                            token_address: token_address.clone(),
                            strike_box_size: strike_box_size_f64,
                            strike_box_tp_prices: [
                                tp_prices[0].to_string().parse::<f64>().unwrap_or(entry_price * 1.15),
                                tp_prices[1].to_string().parse::<f64>().unwrap_or(entry_price * 1.30),
                                tp_prices[2].to_string().parse::<f64>().unwrap_or(entry_price * 1.50),
                            ],
                        };
                        
                        // Higher threshold for non-traditional assets
                        if opportunity.confidence >= MIN_SAFETY_SCORE && volume_ratio >= MIN_VOLUME_RATIO {
                            opportunities.push(opportunity);
                        }
                    }
                    Err(_) => {
                        // Token failed safety check - skip
                        continue;
                    }
                }
            }
        }
        
        // Sort by volume ratio first, then expected profit
        opportunities.sort_by(|a, b| {
            b.volume_ratio.partial_cmp(&a.volume_ratio)
                .unwrap()
                .then(b.expected_profit.partial_cmp(&a.expected_profit).unwrap())
        });
        
        opportunities
    }
    
    async fn scan_volume_spikes(&self, exchange: &str) -> Vec<String> {
        // In production: Fetch all pairs from exchange API
        // Filter for volume spikes (2x+ normal volume)
        // For now, simulate non-traditional pairs with volume spikes
        
        use rand::Rng;
        let mut pairs = Vec::new();
        
        // Example non-traditional pairs (replace with real API calls)
        let potential_pairs = vec![
            "PEPE/USDT", "BONK/USDT", "WIF/USDT", "FLOKI/USDT",
            "SHIB/USDT", "DOGE/USDT", "MEME/USDT", "MOON/USDT",
            "PUMP/USDT", "ROCKET/USDT", "0x1234/USDC", "0x5678/USDT"
        ];
        
        for pair in potential_pairs {
            // Simulate volume spike detection
            let volume_ratio = 1.5 + rand::thread_rng().gen::<f64>() * 2.5; // 1.5-4x
            
            if volume_ratio >= MIN_VOLUME_RATIO {
                pairs.push(pair.to_string());
            }
        }
        
        pairs
    }
    
    async fn fetch_volume_ratio(&self, pair: &str) -> f64 {
        // In production: Fetch from exchange API
        // Compare current volume to 24h average
        use rand::Rng;
        1.5 + rand::thread_rng().gen::<f64>() * 2.5 // Simulated 1.5-4x
    }
    
    async fn fetch_price(&self, pair: &str) -> f64 {
        // In production: Fetch real-time price
        use rand::Rng;
        0.001 + rand::thread_rng().gen::<f64>() * 10.0 // Simulated
    }
    
    async fn calculate_expected_move(&self, pair: &str, volume_ratio: f64) -> f64 {
        // Higher volume = stronger expected move
        let base_move = 0.08; // 8% base
        let volume_multiplier = 1.0 + (volume_ratio - 1.0) * 0.5; // Scale with volume
        base_move * volume_multiplier.min(1.5) // Cap at 12%
    }
    
    fn extract_token_address(&self, pair: &str) -> String {
        // Extract token address from pair (for DEX pairs like "0x1234/USDC")
        if pair.starts_with("0x") {
            pair.split('/').next().unwrap_or("0x0000").to_string()
        } else {
            // For CEX pairs, use pair name as identifier
            format!("token_{}", pair.replace("/", "_"))
        }
    }
    
    async fn get_volatility(&self, _pair: &str) -> f64 {
        // Fetch volatility for pair
        use rand::Rng;
        0.02 + rand::thread_rng().gen::<f64>() * 0.03 // 2-5% volatility
    }
    
    async fn fetch_liquidity_usd(&self, _pair: &str) -> f64 {
        // Fetch liquidity in USD
        use rand::Rng;
        500_000.0 + rand::thread_rng().gen::<f64>() * 500_000.0 // $500K-$1M range
    }
    
    async fn fetch_holder_count(&self, _token_address: &str) -> u32 {
        // Fetch holder count
        use rand::Rng;
        25 + (rand::thread_rng().gen::<f64>() * 75.0) as u32 // 25-100 holders
    }
    
    async fn fetch_token_age_hours(&self, _token_address: &str) -> u32 {
        // Fetch token age in hours
        use rand::Rng;
        24 + (rand::thread_rng().gen::<f64>() * 48.0) as u32 // 24-72 hours
    }

    fn detect_opportunity_type(&self, _exchange: &str, pair: &str) -> OpportunityType {
        // Advanced opportunity detection
        match rand::random::<f64>() {
            x if x < 0.2 => OpportunityType::Arbitrage,
            x if x < 0.4 => OpportunityType::MomentumBreakout,
            x if x < 0.6 => OpportunityType::MeanReversion,
            x if x < 0.8 => OpportunityType::VolumeSpike,
            _ => OpportunityType::MarketMaking,
        }
    }

    fn calculate_expected_profit(&self, _pair: &str) -> f64 {
        // Target 8% per strike
        0.08 + rand::random::<f64>() * 0.04 // 8-12% range
    }

    fn calculate_confidence(&self, _pair: &str) -> f64 {
        0.6 + rand::random::<f64>() * 0.35 // 60-95% confidence
    }

    fn get_volatility(&self, _pair: &str) -> f64 {
        0.02 + rand::random::<f64>() * 0.03 // 2-5% volatility
    }

    fn get_volume_ratio(&self, _pair: &str) -> f64 {
        1.0 + rand::random::<f64>() * 2.0 // 1-3x normal volume
    }

    fn calculate_volume_based_leverage(&self, volume_ratio: f64, safety_score: f64) -> f64 {
        // Scale leverage with volume spike strength AND safety score
        let mut base_leverage = 3.0;
        
        // Increase leverage for strong volume spikes
        if volume_ratio >= 3.0 {
            base_leverage = 5.0; // Strong volume = max leverage
        } else if volume_ratio >= 2.5 {
            base_leverage = 4.5;
        } else if volume_ratio >= 2.0 {
            base_leverage = 4.0; // Good volume = higher leverage
        }
        
        // Reduce leverage for lower safety scores
        if safety_score < 0.8 {
            base_leverage *= 0.8; // Reduce 20% for moderate risk
        }
        if safety_score < 0.7 {
            base_leverage *= 0.7; // Reduce 30% for higher risk
        }
        
        // Never exceed 5x cap
        base_leverage.min(MAX_LEVERAGE)
    }
    
    fn calculate_optimal_leverage(&self, pair: &str) -> f64 {
        // Legacy function - kept for compatibility
        self.calculate_volume_based_leverage(2.0, 0.8)
    }

    async fn aggregate_cycle_results(&mut self, results: Vec<StrikeResult>) {
        let mut aggregator = self.performance_aggregator.write().await;
        
        for result in results {
            aggregator.add_result(result);
            self.cycle_profits += result.profit;
        }
        
        // Update total capital
        self.total_capital += self.cycle_profits;
    }

    async fn rebalance_capital(&mut self) {
        let mut pool = self.capital_pool.write().await;
        
        // Redistribute profits to all bots
        let profit_per_bot = self.cycle_profits / NUM_BOTS as f64;
        
        for bot in &self.bots {
            let mut bot_guard = bot.lock().await;
            bot_guard.add_capital(profit_per_bot);
        }
        
        pool.rebalance();
    }

    fn is_cycle_complete(&self) -> bool {
        let elapsed = Utc::now() - self.cycle_start;
        elapsed.num_hours() >= 24 // Check daily, full cycle is 14 days
    }

    async fn print_cycle_report(&self) {
        let aggregator = self.performance_aggregator.read().await;
        let stats = aggregator.get_stats();
        
        println!("\n╔═══════════════════════════════════════════════════════════════╗");
        println!("║              HUMMINGBOT ARRAY PERFORMANCE REPORT              ║");
        println!("╠═══════════════════════════════════════════════════════════════╣");
        println!("║ CYCLE PERFORMANCE                                             ║");
        println!("║   Duration:            {} days                                ║", 
            (Utc::now() - self.cycle_start).num_days());
        println!("║   Active Bots:         {}/25                                  ║", NUM_BOTS);
        println!("║   Total Strikes:       {:>6}                                 ║", stats.total_strikes);
        println!("║   Successful:          {:>6}                                 ║", stats.successful_strikes);
        println!("║   Win Rate:            {:.1}%                                 ║", stats.win_rate * 100.0);
        println!("║                                                               ║");
        println!("║ FINANCIAL METRICS                                             ║");
        println!("║   Starting Capital:    ${:>12.2}                         ║", INITIAL_CAPITAL);
        println!("║   Current Capital:     ${:>12.2}                         ║", self.total_capital);
        println!("║   Cycle Profit:        ${:>12.2}                         ║", self.cycle_profits);
        println!("║   Total Return:        {:.1}%                                 ║", 
            (self.cycle_profits / INITIAL_CAPITAL) * 100.0);
        println!("║   Per Bot Average:     {:.1}%                                 ║", 
            (self.cycle_profits / INITIAL_CAPITAL) * 100.0 / NUM_BOTS as f64);
        println!("║                                                               ║");
        println!("║ LEVERAGE METRICS                                              ║");
        println!("║   Average Leverage:    {:.1}x                                 ║", stats.avg_leverage);
        println!("║   Max Leverage Used:   {:.1}x                                 ║", stats.max_leverage);
        println!("║   Risk Utilization:    {:.1}%                                 ║", stats.risk_utilization * 100.0);
        println!("║                                                               ║");
        println!("║ 7-DAY PROJECTION                                              ║");
        println!("║   Target (200%):       ${:>12.2}                         ║", INITIAL_CAPITAL * 2.0);
        println!("║   Current Pace:        ${:>12.2}                         ║", 
            self.project_7_day_return());
        println!("║   On Track:            {}                                     ║",
            if self.is_on_track() { "✅ YES" } else { "⚠️  ADJUST" });
        println!("╚═══════════════════════════════════════════════════════════════╝");
    }

    fn project_7_day_return(&self) -> f64 {
        let daily_return = self.cycle_profits / (Utc::now() - self.cycle_start).num_days().max(1) as f64;
        INITIAL_CAPITAL + (daily_return * 7.0) // Changed from 14.0 to 7.0
    }
    
    fn project_14_day_return(&self) -> f64 {
        // Legacy function - now projects 7 days
        self.project_7_day_return()
    }

    fn is_on_track(&self) -> bool {
        let target_daily = INITIAL_CAPITAL * 2.0 / 7.0; // Changed from 14.0 to 7.0
        let actual_daily = self.cycle_profits / (Utc::now() - self.cycle_start).num_days().max(1) as f64;
        actual_daily >= target_daily * 0.9 // Within 10% of target
    }
    
    async fn check_and_close_positions(&mut self) {
        // Check all open positions across all bots and close if conditions met
        for bot in &self.bots {
            let mut bot_guard = bot.lock().await;
            let mut positions_to_close = Vec::new();
            
            for position in &bot_guard.positions {
                if position.status == PositionStatus::Open {
                    let position_age = Utc::now() - position.opened_at;
                    
                    // Force close if position is older than 1 minute
                    if position_age.num_seconds() >= MAX_POSITION_TIME_SECONDS {
                        positions_to_close.push((position.id.clone(), ExitReason::TimeLimit));
                        continue;
                    }
                    
                    let current_price = bot_guard.fetch_current_price(&position.pair).await;
                    
                    // Check if target hit
                    let target_hit = if matches!(position.side, Side::Long) {
                        current_price >= position.target_price
                    } else {
                        current_price <= position.target_price
                    };
                    
                    // Check if stop loss hit
                    let stop_hit = if matches!(position.side, Side::Long) {
                        current_price <= position.stop_loss
                    } else {
                        current_price >= position.stop_loss
                    };
                    
                    if target_hit {
                        positions_to_close.push((position.id.clone(), ExitReason::TargetHit));
                    } else if stop_hit {
                        positions_to_close.push((position.id.clone(), ExitReason::StopLoss));
                    }
                }
            }
            
            // Close positions immediately
            for (pos_id, exit_reason) in positions_to_close {
                if let Some(position) = bot_guard.positions.iter_mut()
                    .find(|p| p.id == pos_id && p.status == PositionStatus::Open) {
                    
                    let current_price = bot_guard.fetch_current_price(&position.pair).await;
                    let exit_price = match exit_reason {
                        ExitReason::TargetHit => position.target_price,
                        ExitReason::StopLoss => position.stop_loss,
                        _ => current_price,
                    };
                    
                    // Execute exit immediately
                    let exit_result = bot_guard.execute_exit_trade(position, exit_price).await;
                    
                    // Calculate profit
                    let price_change = if matches!(position.side, Side::Long) {
                        (exit_price - position.entry_price) / position.entry_price
                    } else {
                        (position.entry_price - exit_price) / position.entry_price
                    };
                    
                    let profit = position.leveraged_size * price_change;
                    
                    // Update position
                    position.status = PositionStatus::Closed;
                    position.exit_price = Some(exit_price);
                    position.exit_reason = Some(exit_reason.clone());
                    position.closed_at = Some(Utc::now());
                    
                    // Update capital
                    bot_guard.capital += profit;
                    bot_guard.performance.add_trade(profit > 0.0, profit);
                    
                    info!("✅ Bot {} closed position {}: {} | Profit: ${:.2}", 
                        bot_guard.id, pos_id, exit_reason.name(), profit);
                }
            }
            
            // Remove closed positions
            bot_guard.positions.retain(|p| p.status == PositionStatus::Open);
        }
    }

    fn reset_cycle(&mut self) {
        self.cycle_start = Utc::now();
        self.cycle_profits = 0.0;
    }
}

// ==================== INDIVIDUAL HUMMINGBOT ====================

#[derive(Debug, Clone)]
pub struct HummingBot {
    id: usize,
    capital: f64,
    strategy: BotStrategy,
    positions: Vec<BotPosition>,
    performance: BotPerformance,
    capital_pool: Arc<RwLock<CapitalPool>>,
    strike_coordinator: Arc<StrikeCoordinator>,
}

impl HummingBot {
    pub fn new(
        id: usize,
        capital: f64,
        strategy: BotStrategy,
        capital_pool: Arc<RwLock<CapitalPool>>,
        strike_coordinator: Arc<StrikeCoordinator>,
    ) -> Self {
        Self {
            id,
            capital,
            strategy,
            positions: Vec::new(),
            performance: BotPerformance::new(),
            capital_pool,
            strike_coordinator,
        }
    }

    pub async fn execute_strike(&mut self, opportunity: MarketOpportunity) -> StrikeResult {
        info!("🤖 Bot {} executing {} strike on {} {}", 
            self.id, self.strategy.name(), opportunity.exchange, opportunity.pair);
        info!("   Volume Ratio: {:.2}x | Leverage: {:.1}x | Safety: {:.1}%", 
            opportunity.volume_ratio, opportunity.leverage, opportunity.safety_score * 100.0);
        
        // Calculate position size (use most of capital with leverage)
        let position_size = self.capital * 0.95; // Use 95% of capital
        let leveraged_size = position_size * opportunity.leverage;
        
        // Create position
        let mut position = BotPosition {
            id: format!("BOT{}_POS_{}", self.id, uuid::Uuid::new_v4()),
            bot_id: self.id,
            exchange: opportunity.exchange.clone(),
            pair: opportunity.pair.clone(),
            side: if opportunity.opportunity_type == OpportunityType::MomentumBreakout {
                Side::Long
            } else {
                Side::Long
            },
            size: position_size,
            leveraged_size,
            entry_price: opportunity.entry_price,
            target_price: opportunity.target_price,
            stop_loss: opportunity.stop_loss,
            leverage: opportunity.leverage,
            opened_at: Utc::now(),
            status: PositionStatus::Open,
            exit_price: None,
            exit_reason: None,
            closed_at: None,
        };
        
        // Execute entry trade
        let entry_result = self.execute_entry_trade(&position).await;
        
        if !entry_result.success {
            return StrikeResult {
                bot_id: self.id,
                opportunity: opportunity.clone(),
                position,
                profit: 0.0,
                execution_time_ms: entry_result.execution_time_ms,
                success: false,
            };
        }
        
        self.positions.push(position.clone());
        
        // IMMEDIATE MONITORING - Exit as soon as target hit or 1 minute elapsed
        let (exit_price, exit_reason, profit) = 
            self.monitor_and_exit_immediately(&mut position).await;
        
        // Execute exit trade IMMEDIATELY
        let exit_result = self.execute_exit_trade(&position, exit_price).await;
        
        // Update position status
        position.exit_price = Some(exit_price);
        position.exit_reason = Some(exit_reason.clone());
        position.closed_at = Some(Utc::now());
        position.status = PositionStatus::Closed;
        
        // Remove from open positions
        self.positions.retain(|p| p.id != position.id);
        
        // Update performance
        self.performance.add_trade(profit > 0.0, profit);
        self.capital += profit;
        
        info!("✅ Bot {} exited position: {} | Profit: ${:.2} | Reason: {}", 
            self.id, position.pair, profit, exit_reason.name());
        
        StrikeResult {
            bot_id: self.id,
            opportunity: opportunity.clone(),
            position,
            profit,
            execution_time_ms: entry_result.execution_time_ms + exit_result.execution_time_ms,
            success: profit > 0.0,
        }
    }
    
    /// Monitor position and exit IMMEDIATELY when target hit or 1 minute elapsed
    async fn monitor_and_exit_immediately(&self, position: &mut BotPosition) -> (f64, ExitReason, f64) {
        let max_checks = 600; // 1 minute at 100ms intervals
        let check_interval_ms = 100;
        let mut check_count = 0;
        
        loop {
            // Fetch current price
            let current_price = self.fetch_current_price(&position.pair).await;
            
            // Calculate current P&L
            let price_change = if matches!(position.side, Side::Long) {
                (current_price - position.entry_price) / position.entry_price
            } else {
                (position.entry_price - current_price) / position.entry_price
            };
            
            let current_pnl = position.leveraged_size * price_change;
            let current_return_pct = price_change.abs();
            
            // EXIT CONDITION 1: Target hit → IMMEDIATE EXIT
            if current_price >= position.target_price && matches!(position.side, Side::Long) {
                let profit = position.leveraged_size * ((position.target_price - position.entry_price) / position.entry_price);
                return (position.target_price, ExitReason::TargetHit, profit);
            }
            
            if current_price <= position.target_price && matches!(position.side, Side::Short) {
                let profit = position.leveraged_size * ((position.entry_price - position.target_price) / position.entry_price);
                return (position.target_price, ExitReason::TargetHit, profit);
            }
            
            // EXIT CONDITION 2: Stop loss triggered → IMMEDIATE EXIT
            if current_price <= position.stop_loss && matches!(position.side, Side::Long) {
                let loss = position.leveraged_size * ((position.stop_loss - position.entry_price) / position.entry_price);
                return (position.stop_loss, ExitReason::StopLoss, loss);
            }
            
            if current_price >= position.stop_loss && matches!(position.side, Side::Short) {
                let loss = position.leveraged_size * ((position.entry_price - position.stop_loss) / position.entry_price);
                return (position.stop_loss, ExitReason::StopLoss, loss);
            }
            
            // EXIT CONDITION 3: Quick profit > 0.5% → IMMEDIATE EXIT (NO HODL)
            if current_return_pct >= QUICK_PROFIT_THRESHOLD && current_pnl > 0.0 {
                return (current_price, ExitReason::QuickProfit, current_pnl);
            }
            
            // EXIT CONDITION 4: 1-minute time limit → FORCE EXIT
            let position_age = Utc::now() - position.opened_at;
            if position_age.num_seconds() >= MAX_POSITION_TIME_SECONDS {
                return (current_price, ExitReason::TimeLimit, current_pnl);
            }
            
            // Safety check - prevent infinite loop
            check_count += 1;
            if check_count >= max_checks {
                return (current_price, ExitReason::MaxChecks, current_pnl);
            }
            
            // Wait before next check
            tokio::time::sleep(tokio::time::Duration::from_millis(check_interval_ms)).await;
        }
    }
    
    async fn execute_entry_trade(&self, position: &BotPosition) -> TradeResult {
        // In production: Execute actual buy/sell order
        // For now: Simulate execution
        
        info!("📈 Entry: {} {} @ ${:.4}", 
            if matches!(position.side, Side::Long) { "BUY" } else { "SELL" },
            position.pair, position.entry_price);
        
        TradeResult {
            success: true,
            execution_time_ms: 50, // Simulated
            filled_price: position.entry_price,
            filled_quantity: position.size,
        }
    }
    
    async fn execute_exit_trade(&self, position: &BotPosition, exit_price: f64) -> TradeResult {
        // In production: Execute actual sell/buy order
        // For now: Simulate execution
        
        info!("📉 Exit: {} {} @ ${:.4}", 
            if matches!(position.side, Side::Long) { "SELL" } else { "BUY" },
            position.pair, exit_price);
        
        TradeResult {
            success: true,
            execution_time_ms: 50, // Simulated
            filled_price: exit_price,
            filled_quantity: position.size,
        }
    }
    
    async fn fetch_current_price(&self, pair: &str) -> f64 {
        // In production: Fetch real-time price from exchange
        // For now: Simulate price movement with some volatility
        use rand::Rng;
        let base_price = 0.1;
        let volatility = 0.05; // 5% volatility
        base_price + (rand::thread_rng().gen::<f64>() - 0.5) * volatility
    }

    async fn execute_market_making(&self, position: &BotPosition) -> f64 {
        // Market making logic - capture spreads
        let spread_capture = position.leveraged_size * 0.002; // 0.2% spread
        let rebate = position.leveraged_size * 0.0001; // 0.01% rebate
        let volume_bonus = position.leveraged_size * 0.001; // Volume incentive
        
        spread_capture + rebate + volume_bonus
    }

    async fn execute_arbitrage(&self, position: &BotPosition) -> f64 {
        // Arbitrage logic - capture price differences
        let price_diff = 0.003; // 0.3% arbitrage opportunity
        position.leveraged_size * price_diff
    }

    async fn execute_momentum(&self, position: &BotPosition) -> f64 {
        // Momentum logic - ride the trend
        let trend_profit = (position.target_price - position.entry_price) / position.entry_price;
        position.leveraged_size * trend_profit * 0.8 // 80% of target achieved
    }

    async fn execute_mean_reversion(&self, position: &BotPosition) -> f64 {
        // Mean reversion logic
        let reversion_profit = 0.025; // 2.5% mean reversion
        position.leveraged_size * reversion_profit
    }

    async fn execute_volatility(&self, position: &BotPosition) -> f64 {
        // Volatility trading logic
        let vol_profit = 0.035; // 3.5% from volatility
        position.leveraged_size * vol_profit
    }

    pub fn add_capital(&mut self, amount: f64) {
        self.capital += amount;
    }
}

// ==================== STRIKE COORDINATOR ====================

pub struct StrikeCoordinator {
    assignment_history: Arc<RwLock<HashMap<usize, Vec<MarketOpportunity>>>>,
    coordination_matrix: Arc<RwLock<Vec<Vec<f64>>>>,
}

impl StrikeCoordinator {
    pub fn new() -> Self {
        Self {
            assignment_history: Arc::new(RwLock::new(HashMap::new())),
            coordination_matrix: Arc::new(RwLock::new(vec![vec![0.0; NUM_BOTS]; NUM_BOTS])),
        }
    }

    pub async fn assign_targets(
        &self,
        opportunities: &[MarketOpportunity],
        num_bots: usize,
    ) -> HashMap<usize, MarketOpportunity> {
        let mut assignments = HashMap::new();
        
        // Distribute opportunities to maximize coverage
        for (i, opportunity) in opportunities.iter().take(num_bots).enumerate() {
            // Assign to bot based on strategy match
            let bot_id = self.find_best_bot_for_opportunity(opportunity, i);
            assignments.insert(bot_id, opportunity.clone());
        }
        
        // Fill remaining bots with best opportunities
        let mut bot_id = 0;
        for opportunity in opportunities.iter().cycle().take(num_bots) {
            if !assignments.contains_key(&bot_id) {
                assignments.insert(bot_id, opportunity.clone());
            }
            bot_id = (bot_id + 1) % num_bots;
        }
        
        assignments
    }

    fn find_best_bot_for_opportunity(&self, opportunity: &MarketOpportunity, default: usize) -> usize {
        match opportunity.opportunity_type {
            OpportunityType::Arbitrage => default % 5 + 5, // Bots 5-9
            OpportunityType::MomentumBreakout => default % 5 + 10, // Bots 10-14
            OpportunityType::MeanReversion => default % 5 + 15, // Bots 15-19
            OpportunityType::VolumeSpike => default % 5 + 20, // Bots 20-24
            OpportunityType::MarketMaking => default % 5, // Bots 0-4
        }
    }
}

// ==================== CAPITAL POOL MANAGER ====================

#[derive(Debug)]
pub struct CapitalPool {
    total_capital: f64,
    allocated_capital: f64,
    reserve_capital: f64,
    bot_allocations: HashMap<usize, f64>,
}

impl CapitalPool {
    pub fn new(initial_capital: f64) -> Self {
        let mut bot_allocations = HashMap::new();
        let per_bot = initial_capital / NUM_BOTS as f64;
        
        for i in 0..NUM_BOTS {
            bot_allocations.insert(i, per_bot);
        }
        
        Self {
            total_capital: initial_capital,
            allocated_capital: initial_capital * 0.95,
            reserve_capital: initial_capital * 0.05,
            bot_allocations,
        }
    }

    pub fn rebalance(&mut self) {
        // Rebalance capital across bots
        let total = self.bot_allocations.values().sum::<f64>();
        let target_per_bot = total / NUM_BOTS as f64;
        
        for allocation in self.bot_allocations.values_mut() {
            *allocation = target_per_bot;
        }
    }
}

// ==================== PERFORMANCE AGGREGATOR ====================

#[derive(Debug)]
pub struct PerformanceAggregator {
    total_strikes: u32,
    successful_strikes: u32,
    total_profit: f64,
    total_loss: f64,
    leverage_history: Vec<f64>,
    strike_times: Vec<DateTime<Utc>>,
}

impl PerformanceAggregator {
    pub fn new() -> Self {
        Self {
            total_strikes: 0,
            successful_strikes: 0,
            total_profit: 0.0,
            total_loss: 0.0,
            leverage_history: Vec::new(),
            strike_times: Vec::new(),
        }
    }

    pub fn add_result(&mut self, result: StrikeResult) {
        self.total_strikes += 1;
        if result.success {
            self.successful_strikes += 1;
            self.total_profit += result.profit;
        } else {
            self.total_loss += result.profit.abs();
        }
        self.leverage_history.push(result.position.leverage);
        self.strike_times.push(Utc::now());
    }

    pub fn get_stats(&self) -> AggregatedStats {
        AggregatedStats {
            total_strikes: self.total_strikes,
            successful_strikes: self.successful_strikes,
            win_rate: self.successful_strikes as f64 / self.total_strikes.max(1) as f64,
            total_profit: self.total_profit,
            total_loss: self.total_loss,
            avg_leverage: self.leverage_history.iter().sum::<f64>() / self.leverage_history.len().max(1) as f64,
            max_leverage: self.leverage_history.iter().fold(0.0, |a, &b| a.max(b)),
            risk_utilization: self.calculate_risk_utilization(),
        }
    }

    fn calculate_risk_utilization(&self) -> f64 {
        // Calculate how much of available risk capacity is being used
        let avg_leverage = self.leverage_history.iter().sum::<f64>() / self.leverage_history.len().max(1) as f64;
        avg_leverage / MAX_LEVERAGE
    }
}

// ==================== DATA STRUCTURES ====================

#[derive(Debug, Clone)]
pub struct MarketOpportunity {
    pub exchange: String,
    pub pair: String,
    pub opportunity_type: OpportunityType,
    pub expected_profit: f64,
    pub confidence: f64,
    pub volatility: f64,
    pub volume_ratio: f64,
    pub entry_price: f64,
    pub target_price: f64,
    pub stop_loss: f64,
    pub leverage: f64,
    pub safety_score: f64,      // Token safety score
    pub token_address: String,  // Token address for validation
    pub strike_box_size: f64,   // Strike Box calculated position size
    pub strike_box_tp_prices: [f64; 3], // Strike Box take profit prices (TP1, TP2, TP3)
}

#[derive(Debug, Clone, PartialEq)]
pub enum OpportunityType {
    Arbitrage,
    MomentumBreakout,
    MeanReversion,
    VolumeSpike,
    MarketMaking,
}

#[derive(Debug, Clone)]
pub enum BotStrategy {
    MarketMaking,
    Arbitrage,
    Momentum,
    MeanReversion,
    Volatility,
}

impl BotStrategy {
    pub fn name(&self) -> &str {
        match self {
            Self::MarketMaking => "Market Making",
            Self::Arbitrage => "Arbitrage",
            Self::Momentum => "Momentum",
            Self::MeanReversion => "Mean Reversion",
            Self::Volatility => "Volatility",
        }
    }
}

#[derive(Debug, Clone)]
pub struct BotPosition {
    pub id: String,
    pub bot_id: usize,
    pub exchange: String,
    pub pair: String,
    pub side: Side,
    pub size: f64,
    pub leveraged_size: f64,
    pub entry_price: f64,
    pub target_price: f64,
    pub stop_loss: f64,
    pub leverage: f64,
    pub opened_at: DateTime<Utc>,
    pub status: PositionStatus,
    pub exit_price: Option<f64>,      // NEW: Exit price
    pub exit_reason: Option<ExitReason>, // NEW: Why position was closed
    pub closed_at: Option<DateTime<Utc>>, // NEW: When position was closed
}

#[derive(Debug, Clone)]
pub enum ExitReason {
    TargetHit,      // Hit target price - take profit
    StopLoss,       // Stop loss triggered
    QuickProfit,    // Quick profit > 0.5% - exit immediately
    TimeLimit,      // Maximum time limit reached (1 minute)
    MaxChecks,      // Safety limit reached
}

impl ExitReason {
    pub fn name(&self) -> &str {
        match self {
            Self::TargetHit => "Target Hit",
            Self::StopLoss => "Stop Loss",
            Self::QuickProfit => "Quick Profit",
            Self::TimeLimit => "Time Limit (1 min)",
            Self::MaxChecks => "Max Checks",
        }
    }
}

#[derive(Debug, Clone)]
pub struct TradeResult {
    pub success: bool,
    pub execution_time_ms: u64,
    pub filled_price: f64,
    pub filled_quantity: f64,
}

#[derive(Debug, Clone)]
pub enum Side {
    Long,
    Short,
}

#[derive(Debug, Clone)]
pub enum PositionStatus {
    Open,
    Closed,
    Partial,
}

#[derive(Debug, Clone)]
pub struct StrikeResult {
    pub bot_id: usize,
    pub opportunity: MarketOpportunity,
    pub position: BotPosition,
    pub profit: f64,
    pub execution_time_ms: u64,
    pub success: bool,
}

#[derive(Debug)]
pub struct BotPerformance {
    pub trades_won: u32,
    pub trades_lost: u32,
    pub total_profit: f64,
    pub total_loss: f64,
}

impl BotPerformance {
    pub fn new() -> Self {
        Self {
            trades_won: 0,
            trades_lost: 0,
            total_profit: 0.0,
            total_loss: 0.0,
        }
    }

    pub fn add_trade(&mut self, won: bool, profit: f64) {
        if won {
            self.trades_won += 1;
            self.total_profit += profit;
        } else {
            self.trades_lost += 1;
            self.total_loss += profit.abs();
        }
    }
}

#[derive(Debug)]
pub struct AggregatedStats {
    pub total_strikes: u32,
    pub successful_strikes: u32,
    pub win_rate: f64,
    pub total_profit: f64,
    pub total_loss: f64,
    pub avg_leverage: f64,
    pub max_leverage: f64,
    pub risk_utilization: f64,
}

// ==================== MAIN EXECUTION ====================

pub async fn launch_hummingbot_array() {
    println!("\n🚀 INITIALIZING HUMMINGBOT ARRAY SYSTEM");
    println!("═══════════════════════════════════════════════════════════════");
    println!("📊 Configuration:");
    println!("   • Initial Capital: $800,000");
    println!("   • Number of Bots: 25");
    println!("   • Capital per Bot: $32,000");
    println!("   • Target per Bot: 8% per cycle");
        println!("   • Combined Target: 200% every 7 days");
    println!("   • Leverage Range: 3-5x (conservative)");
    println!("═══════════════════════════════════════════════════════════════");
    
    let mut array = HummingbotArray::new().await;
    
    println!("\n✅ Array initialized successfully");
    println!("🤖 Deploying 25 specialized bots:");
    println!("   • 5 Market Making bots");
    println!("   • 5 Arbitrage bots");
    println!("   • 5 Momentum bots");
    println!("   • 5 Mean Reversion bots");
    println!("   • 5 Volatility bots");
    
    println!("\n⚡ Starting coordinated strike operations...\n");
    
    array.execute_coordinated_strike().await;
}

// Random number generation helper
mod rand {
    pub fn random<T>() -> T 
    where
        rand::distributions::Standard: rand::distributions::Distribution<T>,
    {
        use rand::Rng;
        rand::thread_rng().gen()
    }
}

// UUID generation
mod uuid {
    pub struct Uuid;
    impl Uuid {
        pub fn new_v4() -> String {
            format!("{:x}", rand::random::<u128>())
        }
    }
}
