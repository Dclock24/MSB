use std::collections::VecDeque;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use tokio::time::sleep;
use serde::{Deserialize, Serialize};
use log::{info, warn, error};
use rand::Rng;

// MACRO STRIKE CONFIGURATION - 2500 TRADES
const TOTAL_TRADES: usize = 2500;
const TARGET_MONTHLY_RETURN: f64 = 5.0; // 500% return target
const DAILY_TARGET_RETURN: f64 = 0.065; // 6.5% daily target
const INITIAL_CAPITAL: f64 = 1_000_000.0; // $1M
const TARGET_CAPITAL: f64 = 6_000_000.0; // $6M

// OPTIMIZED MACRO STRIKE PARAMETERS
const STRIKE_FORCE: f64 = 0.15; // 15% of capital per strike
const PRECISION_THRESHOLD: f64 = 0.85; // 85% confidence required
const IMPACT_MULTIPLIER: f64 = 3.0; // 3x leverage on strikes
const MAX_EXPOSURE_TIME_MS: u64 = 30000; // 30 seconds max exposure
const STRIKE_COOLDOWN_MS: u64 = 1; // 1ms cooldown

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrikeType {
    MacroArbitrage,
    MacroMomentum,
    MacroVolatility,
    MacroLiquidity,
    MacroFunding,
    MacroFlash,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrikeStatus {
    Targeting,
    Striking,
    Hit,
    Miss,
    Aborted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroStrike {
    pub id: u64,
    pub symbol: u8, // Use u8 instead of String (0-255 symbols max)
    pub strike_type: StrikeType,
    pub entry_price: f64, // Use f64 instead of Decimal
    pub target_price: f64,
    pub stop_loss: f64,
    pub confidence: f64,
    pub expected_return: f64,
    pub max_exposure_time_ms: u64,
    pub strike_force: f64,
    pub timestamp: u64,
    pub status: StrikeStatus,
    pub hit_time: Option<u64>,
    pub exit_price: Option<f64>,
    pub pnl: Option<f64>,
    pub leverage: u32,
}

// SYMBOL MAPPING (u8 to string)
const SYMBOLS: [&str; 8] = [
    "WETH/USDC", "WBTC/USDC", "LINK/USDC", "UNI/USDC", 
    "AAVE/USDC", "CRV/USDC", "USDC/USDT", "DAI/USDC"
];

fn symbol_to_u8(symbol: &str) -> u8 {
    SYMBOLS.iter().position(|&s| s == symbol).unwrap_or(0) as u8
}

fn u8_to_symbol(id: u8) -> &'static str {
    SYMBOLS[id as usize]
}
pub struct MacroStrikeEngine {
    // Use AtomicU64 for lock-free operations
    capital: AtomicU64, // Store as cents (u64)
    target_capital: u64,
    peak_capital: AtomicU64,
    
    // Use VecDeque for O(1) operations
    active_strikes: VecDeque<MacroStrike>,
    completed_strikes: VecDeque<MacroStrike>,
    
    // Lock-free metrics
    metrics: MacroMetrics,
    
    // Strike tracking
    next_strike_id: AtomicU64,
    consecutive_misses: AtomicUsize,
    max_consecutive_misses: usize,
    max_daily_loss: f64,
    emergency_stop: f64,
}

#[derive(Debug)]
pub struct MacroMetrics {
    pub total_strikes: AtomicUsize,
    pub successful_strikes: AtomicUsize,
    pub failed_strikes: AtomicUsize,
    pub total_pnl: AtomicU64, // Stored as cents
    pub trades_completed: AtomicUsize,
    pub trades_remaining: AtomicUsize,
    pub precision_rate: f64,
    pub average_strike_time_ms: f64,
}

impl Default for MacroMetrics {
    fn default() -> Self {
        Self {
            total_strikes: AtomicUsize::new(0),
            successful_strikes: AtomicUsize::new(0),
            failed_strikes: AtomicUsize::new(0),
            total_pnl: AtomicU64::new(0),
            trades_completed: AtomicUsize::new(0),
            trades_remaining: AtomicUsize::new(TOTAL_TRADES),
            precision_rate: 0.0,
            average_strike_time_ms: 0.0,
        }
    }
}

impl MacroStrikeEngine {
    pub fn new() -> Self {
        Self {
            capital: AtomicU64::new((INITIAL_CAPITAL * 100.0) as u64), // Store as cents
            target_capital: (TARGET_CAPITAL * 100.0) as u64,
            peak_capital: AtomicU64::new((INITIAL_CAPITAL * 100.0) as u64),
            active_strikes: VecDeque::with_capacity(100),
            completed_strikes: VecDeque::with_capacity(TOTAL_TRADES),
            metrics: MacroMetrics::default(),
            next_strike_id: AtomicU64::new(1),
            consecutive_misses: AtomicUsize::new(0),
            max_consecutive_misses: 3,
            max_daily_loss: 0.05,
            emergency_stop: 0.15,
        }
    }

    pub async fn execute_macro_campaign(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("🎯 MACRO STRIKE CAMPAIGN INITIATED - 2500 TRADES");
        info!("Target: ${:.2} in 30 days", self.target_capital as f64 / 100.0);
        info!("Total Trades: {}", TOTAL_TRADES);
        info!("Strike Force: {:.1}% per strike", STRIKE_FORCE * 100.0);

        let start_time = Instant::now();

        // Execute all 2500 trades as fast as possible
        while self.metrics.trades_completed.load(Ordering::Relaxed) < TOTAL_TRADES {
            // Generate and execute strikes in batches for efficiency
            let batch_size = std::cmp::min(10, TOTAL_TRADES - self.metrics.trades_completed.load(Ordering::Relaxed));
            
            for _ in 0..batch_size {
                if self.metrics.trades_completed.load(Ordering::Relaxed) >= TOTAL_TRADES {
                    break;
                }

                // Generate strike
                let strike = self.generate_strike().await;
                let strike_pnl = self.execute_strike(strike).await?;
                
                // Update capital atomically
                let current_capital = self.capital.load(Ordering::Relaxed);
                let new_capital = current_capital + (strike_pnl * 100.0) as u64;
                self.capital.store(new_capital, Ordering::Relaxed);
                
                // Update metrics
                self.metrics.trades_completed.fetch_add(1, Ordering::Relaxed);
                self.metrics.trades_remaining.store(
                    TOTAL_TRADES - self.metrics.trades_completed.load(Ordering::Relaxed), 
                    Ordering::Relaxed
                );

                // Check emergency stops
                if self.check_emergency_stops() {
                    error!("🚨 EMERGENCY STOP ACTIVATED");
                    break;
                }
            }

            // Log progress every 100 trades
            if self.metrics.trades_completed.load(Ordering::Relaxed) % 100 == 0 {
                let current_capital = self.capital.load(Ordering::Relaxed) as f64 / 100.0;
                let progress = (current_capital - INITIAL_CAPITAL) / INITIAL_CAPITAL;
                let trades_completed = self.metrics.trades_completed.load(Ordering::Relaxed);
                let elapsed = start_time.elapsed().as_secs();
                let trades_per_second = trades_completed as f64 / elapsed.max(1) as f64;
                
                info!("Progress: {}/{} trades | Capital: ${:.2} | Progress: {:.1}% | Rate: {:.1} trades/sec", 
                      trades_completed, TOTAL_TRADES, current_capital, progress * 100.0, trades_per_second);
            }

            // Minimal cooldown (1ms) to prevent CPU spinning
            sleep(Duration::from_millis(STRIKE_COOLDOWN_MS)).await;
        }

        // Campaign complete
        let final_capital = self.capital.load(Ordering::Relaxed) as f64 / 100.0;
        let final_return = (final_capital - INITIAL_CAPITAL) / INITIAL_CAPITAL;
        let total_time = start_time.elapsed();
        let trades_completed = self.metrics.trades_completed.load(Ordering::Relaxed);
        
        info!("🏁 CAMPAIGN COMPLETE: {:.1}% return | Trades: {}/{} | Time: {:.2}s", 
              final_return * 100.0, trades_completed, TOTAL_TRADES, total_time.as_secs_f64());

        Ok(())
    }

    async fn generate_strike(&self) -> MacroStrike {
        let strike_id = self.next_strike_id.fetch_add(1, Ordering::Relaxed);
        let symbol_id = (strike_id % SYMBOLS.len() as u64) as u8;
        
        // Generate realistic strike data
        let base_prices = [3000.0, 45000.0, 15.50, 8.50, 120.0, 0.85, 1.00, 1.00];
        let base_price = base_prices[symbol_id as usize];
        let mut rng = rand::thread_rng();
        let movement = (rng.gen::<f64>() - 0.5) * 0.02; // ±1% movement
        let entry_price = base_price * (1.0 + movement);
        
        let strike_types = [
            StrikeType::MacroArbitrage,
            StrikeType::MacroMomentum,
            StrikeType::MacroVolatility,
            StrikeType::MacroLiquidity,
            StrikeType::MacroFunding,
            StrikeType::MacroFlash,
        ];
        let strike_type = strike_types[(strike_id % 6) as usize].clone();
        
        let expected_return = match strike_type {
            StrikeType::MacroArbitrage => 0.005,
            StrikeType::MacroMomentum => 0.022,
            StrikeType::MacroVolatility => 0.032,
            StrikeType::MacroLiquidity => 0.035,
            StrikeType::MacroFunding => 0.042,
            StrikeType::MacroFlash => 0.059,
        };

        MacroStrike {
            id: strike_id,
            symbol: symbol_id,
            strike_type,
            entry_price,
            target_price: entry_price * (1.0 + expected_return),
            stop_loss: entry_price * 0.98, // 2% stop loss
            confidence: 0.85 + (rng.gen::<f64>() * 0.15), // 85-100% confidence
            expected_return,
            max_exposure_time_ms: MAX_EXPOSURE_TIME_MS,
            strike_force: 0.0, // Will be calculated
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            status: StrikeStatus::Targeting,
            hit_time: None,
            exit_price: None,
            pnl: None,
            leverage: 1,
        }
    }

    async fn execute_strike(&mut self, mut strike: MacroStrike) -> Result<f64, Box<dyn std::error::Error>> {
        // Calculate strike size
        let current_capital = self.capital.load(Ordering::Relaxed) as f64 / 100.0;
        let mut strike_size = current_capital * STRIKE_FORCE * strike.confidence;

        // Apply impact multiplier for momentum/volatility
        if matches!(strike.strike_type, StrikeType::MacroMomentum | StrikeType::MacroVolatility) {
            strike_size *= IMPACT_MULTIPLIER;
            strike.leverage = IMPACT_MULTIPLIER as u32;
        }

        strike.strike_force = strike_size;
        strike.status = StrikeStatus::Striking;

        // Simulate strike execution (in real implementation, this would be actual trading)
        let start_time = Instant::now();
        
        // Simulate price movement
        let mut rng = rand::thread_rng();
        let price_movement = (rng.gen::<f64>() - 0.5) * 0.04; // ±2% movement
        let final_price = strike.entry_price * (1.0 + price_movement);

        // Determine if hit or miss based on confidence and randomness
        let hit_probability = strike.confidence;
        let is_hit = rng.gen::<f64>() < hit_probability;
        
        let pnl = if is_hit {
            // Hit - calculate profit
            let price_change = (final_price - strike.entry_price) / strike.entry_price;
            strike_size * price_change * strike.leverage as f64
        } else {
            // Miss - calculate loss
            let price_change = (final_price - strike.entry_price) / strike.entry_price;
            strike_size * price_change * strike.leverage as f64
        };

        // Update metrics
        self.metrics.total_strikes.fetch_add(1, Ordering::Relaxed);
        if is_hit {
            self.metrics.successful_strikes.fetch_add(1, Ordering::Relaxed);
            self.consecutive_misses.store(0, Ordering::Relaxed);
        } else {
            self.metrics.failed_strikes.fetch_add(1, Ordering::Relaxed);
            self.consecutive_misses.fetch_add(1, Ordering::Relaxed);
        }

        // Update precision rate
        let total_strikes = self.metrics.total_strikes.load(Ordering::Relaxed);
        let successful_strikes = self.metrics.successful_strikes.load(Ordering::Relaxed);
        self.metrics.precision_rate = successful_strikes as f64 / total_strikes as f64;

        // Log strike result
        let strike_time = start_time.elapsed().as_millis() as f64;
        if is_hit {
            info!("✅ HIT: {} | PnL=${:.2} | Time={:.1}ms | Trades: {}/{}", 
                  u8_to_symbol(strike.symbol), pnl, strike_time,
                  self.metrics.trades_completed.load(Ordering::Relaxed) + 1, TOTAL_TRADES);
        } else {
            warn!("❌ MISS: {} | PnL=${:.2} | Time={:.1}ms | Trades: {}/{}", 
                  u8_to_symbol(strike.symbol), pnl, strike_time,
                  self.metrics.trades_completed.load(Ordering::Relaxed) + 1, TOTAL_TRADES);
        }

        Ok(pnl)
    }

    fn check_emergency_stops(&self) -> bool {
        let current_capital = self.capital.load(Ordering::Relaxed) as f64 / 100.0;
        let peak_capital = self.peak_capital.load(Ordering::Relaxed) as f64 / 100.0;

        // Check emergency stop
        if current_capital < peak_capital * (1.0 - self.emergency_stop) {
            error!("Emergency stop triggered: {:.2} < {:.2}", current_capital, peak_capital * (1.0 - self.emergency_stop));
            return true;
        }

        // Check consecutive misses
        if self.consecutive_misses.load(Ordering::Relaxed) >= self.max_consecutive_misses {
            error!("Too many consecutive misses: {}", self.consecutive_misses.load(Ordering::Relaxed));
            return true;
        }

        // Update peak capital
        if current_capital > peak_capital {
            self.peak_capital.store((current_capital * 100.0) as u64, Ordering::Relaxed);
        }

        false
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::init();

    info!("🚀 STARTING OPTIMIZED MACRO STRIKE BOT");
    info!("Target: 2500 trades in minimum time");
    info!("No sleep delays, lock-free operations, maximum performance");

    // Create and run optimized macro strike engine
    let mut engine = MacroStrikeEngine::new();
    engine.execute_macro_campaign().await?;

    Ok(())
}
