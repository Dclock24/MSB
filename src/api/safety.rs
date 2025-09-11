// Trading Safety Module
// Implements circuit breakers, position limits, and risk management

use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use std::collections::HashMap;

/// Circuit breaker for emergency stops
#[derive(Debug)]
pub struct CircuitBreaker {
    /// Is the circuit open (tripped)?
    is_open: Arc<std::sync::atomic::AtomicBool>,
    /// Number of failures
    failure_count: Arc<std::sync::atomic::AtomicUsize>,
    /// Failure threshold
    failure_threshold: usize,
    /// Reset time
    reset_after: Duration,
    /// Last failure time
    last_failure: Arc<RwLock<Option<SystemTime>>>,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: usize, reset_after: Duration) -> Self {
        Self {
            is_open: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            failure_count: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            failure_threshold,
            reset_after,
            last_failure: Arc::new(RwLock::new(None)),
        }
    }
    
    pub async fn call<F, T, E>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce() -> Result<T, E>,
        E: std::fmt::Debug,
    {
        // Check if circuit should be reset
        if self.is_open.load(std::sync::atomic::Ordering::Relaxed) {
            let last = self.last_failure.read().await;
            if let Some(last_time) = *last {
                if SystemTime::now().duration_since(last_time).unwrap_or_default() > self.reset_after {
                    self.is_open.store(false, std::sync::atomic::Ordering::Relaxed);
                    self.failure_count.store(0, std::sync::atomic::Ordering::Relaxed);
                }
            }
        }
        
        // Check if circuit is open
        if self.is_open.load(std::sync::atomic::Ordering::Relaxed) {
            return Err("Circuit breaker is open".to_string());
        }
        
        // Try the operation
        match f() {
            Ok(result) => {
                // Reset failure count on success
                self.failure_count.store(0, std::sync::atomic::Ordering::Relaxed);
                Ok(result)
            }
            Err(e) => {
                // Increment failure count
                let failures = self.failure_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                
                // Update last failure time
                *self.last_failure.write().await = Some(SystemTime::now());
                
                // Trip circuit if threshold reached
                if failures >= self.failure_threshold {
                    self.is_open.store(true, std::sync::atomic::Ordering::Relaxed);
                    log::error!("Circuit breaker tripped after {} failures", failures);
                }
                
                Err(format!("Operation failed: {:?}", e))
            }
        }
    }
}

/// Safety configuration
#[derive(Debug, Clone)]
pub struct SafetyConfig {
    /// Maximum position size per symbol (in USD)
    pub max_position_size: f64,
    
    /// Maximum total portfolio exposure (in USD)
    pub max_total_exposure: f64,
    
    /// Maximum loss per day (in USD)
    pub max_daily_loss: f64,
    
    /// Maximum number of trades per hour
    pub max_trades_per_hour: u32,
    
    /// Minimum time between trades (seconds)
    pub min_trade_interval: u64,
    
    /// Circuit breaker: consecutive losses before halt
    pub max_consecutive_losses: u32,
    
    /// Circuit breaker: loss percentage before halt
    pub max_loss_percentage: f64,
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            max_position_size: 10_000.0,
            max_total_exposure: 50_000.0,
            max_daily_loss: 1_000.0,
            max_trades_per_hour: 60,
            min_trade_interval: 5,
            max_consecutive_losses: 5,
            max_loss_percentage: 0.10, // 10%
        }
    }
}

/// Trading statistics for safety monitoring
#[derive(Debug, Default)]
struct TradingStats {
    trades_this_hour: u32,
    last_trade_time: Option<SystemTime>,
    consecutive_losses: u32,
    daily_pnl: f64,
    hourly_trades: Vec<(SystemTime, String)>,
    positions: HashMap<String, f64>,
}

/// Safety monitor for live trading
pub struct SafetyMonitor {
    config: SafetyConfig,
    stats: Arc<RwLock<TradingStats>>,
    circuit_breaker_active: Arc<RwLock<bool>>,
}

impl SafetyMonitor {
    pub fn new(config: SafetyConfig) -> Self {
        Self {
            config,
            stats: Arc::new(RwLock::new(TradingStats::default())),
            circuit_breaker_active: Arc::new(RwLock::new(false)),
        }
    }

    /// Check if a trade is allowed
    pub async fn check_trade_allowed(
        &self,
        symbol: &str,
        size_usd: f64,
        is_closing: bool,
    ) -> Result<(), String> {
        // Check circuit breaker
        if *self.circuit_breaker_active.read().await {
            return Err("Circuit breaker active - trading halted".to_string());
        }

        // Always allow closing positions
        if is_closing {
            return Ok(());
        }

        let mut stats = self.stats.write().await;

        // Check position size limit
        if size_usd > self.config.max_position_size {
            return Err(format!(
                "Position size ${:.2} exceeds limit ${:.2}",
                size_usd, self.config.max_position_size
            ));
        }

        // Check total exposure
        let current_exposure: f64 = stats.positions.values().sum();
        if current_exposure + size_usd > self.config.max_total_exposure {
            return Err(format!(
                "Total exposure would exceed ${:.2} limit",
                self.config.max_total_exposure
            ));
        }

        // Check trade frequency
        let now = SystemTime::now();
        
        // Clean old hourly trades
        let hour_ago = now - Duration::from_secs(3600);
        stats.hourly_trades.retain(|(time, _)| *time > hour_ago);
        
        if stats.hourly_trades.len() >= self.config.max_trades_per_hour as usize {
            return Err(format!(
                "Exceeded {} trades per hour limit",
                self.config.max_trades_per_hour
            ));
        }

        // Check minimum interval
        if let Some(last_trade) = stats.last_trade_time {
            let elapsed = now.duration_since(last_trade).unwrap_or_default();
            if elapsed < Duration::from_secs(self.config.min_trade_interval) {
                return Err(format!(
                    "Must wait {} seconds between trades",
                    self.config.min_trade_interval
                ));
            }
        }

        // Check daily loss limit
        if stats.daily_pnl < -self.config.max_daily_loss {
            return Err(format!(
                "Daily loss ${:.2} exceeds limit ${:.2}",
                -stats.daily_pnl, self.config.max_daily_loss
            ));
        }

        // Update stats
        stats.last_trade_time = Some(now);
        stats.hourly_trades.push((now, symbol.to_string()));
        stats.positions.insert(symbol.to_string(), size_usd);

        Ok(())
    }

    /// Record trade result
    pub async fn record_trade_result(&self, pnl: f64, is_win: bool) {
        let mut stats = self.stats.write().await;
        
        stats.daily_pnl += pnl;

        if is_win {
            stats.consecutive_losses = 0;
        } else {
            stats.consecutive_losses += 1;
            
            // Check consecutive loss circuit breaker
            if stats.consecutive_losses >= self.config.max_consecutive_losses {
                log::error!(
                    "CIRCUIT BREAKER: {} consecutive losses - halting trading",
                    stats.consecutive_losses
                );
                *self.circuit_breaker_active.write().await = true;
            }
        }

        // Check percentage loss circuit breaker
        if stats.daily_pnl < 0.0 {
            let loss_percentage = -stats.daily_pnl / self.config.max_total_exposure;
            if loss_percentage > self.config.max_loss_percentage {
                log::error!(
                    "CIRCUIT BREAKER: {:.1}% portfolio loss - halting trading",
                    loss_percentage * 100.0
                );
                *self.circuit_breaker_active.write().await = true;
            }
        }
    }

    /// Reset daily statistics
    pub async fn reset_daily_stats(&self) {
        let mut stats = self.stats.write().await;
        stats.daily_pnl = 0.0;
        stats.consecutive_losses = 0;
        stats.hourly_trades.clear();
        log::info!("Daily trading statistics reset");
    }

    /// Get current safety status
    pub async fn get_status(&self) -> SafetyStatus {
        let stats = self.stats.read().await;
        let circuit_breaker = *self.circuit_breaker_active.read().await;

        SafetyStatus {
            circuit_breaker_active: circuit_breaker,
            daily_pnl: stats.daily_pnl,
            trades_this_hour: stats.hourly_trades.len() as u32,
            consecutive_losses: stats.consecutive_losses,
            total_exposure: stats.positions.values().sum(),
            last_trade_time: stats.last_trade_time,
        }
    }

    /// Emergency stop - halt all trading
    pub async fn emergency_stop(&self, reason: &str) {
        log::error!("EMERGENCY STOP: {}", reason);
        *self.circuit_breaker_active.write().await = true;
    }

    /// Resume trading after circuit breaker
    pub async fn resume_trading(&self) {
        log::info!("Resuming trading - circuit breaker deactivated");
        *self.circuit_breaker_active.write().await = false;
        
        // Reset consecutive losses
        let mut stats = self.stats.write().await;
        stats.consecutive_losses = 0;
    }
}

/// Current safety status
#[derive(Debug)]
pub struct SafetyStatus {
    pub circuit_breaker_active: bool,
    pub daily_pnl: f64,
    pub trades_this_hour: u32,
    pub consecutive_losses: u32,
    pub total_exposure: f64,
    pub last_trade_time: Option<SystemTime>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_position_size_limit() {
        let config = SafetyConfig {
            max_position_size: 1000.0,
            ..Default::default()
        };
        let monitor = SafetyMonitor::new(config);

        // Should reject oversized position
        let result = monitor.check_trade_allowed("BTC/USDT", 1500.0, false).await;
        assert!(result.is_err());

        // Should allow normal position
        let result = monitor.check_trade_allowed("BTC/USDT", 500.0, false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_consecutive_losses_circuit_breaker() {
        let config = SafetyConfig {
            max_consecutive_losses: 3,
            ..Default::default()
        };
        let monitor = SafetyMonitor::new(config);

        // Record 3 losses
        monitor.record_trade_result(-100.0, false).await;
        monitor.record_trade_result(-100.0, false).await;
        monitor.record_trade_result(-100.0, false).await;

        // Circuit breaker should be active
        let status = monitor.get_status().await;
        assert!(status.circuit_breaker_active);
    }
}
