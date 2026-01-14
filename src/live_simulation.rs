// Live Simulation System
// Runs strategy in live simulation mode before going fully live
// Validates real-time performance with paper trading

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::errors::{TradingResult, TradingError};

const MIN_SIM_TRADES: u64 = 1000;
const MIN_SIM_DAYS: u64 = 7;
const REQUIRED_SIM_WIN_RATE: f64 = 0.93;
const MAX_SIM_DRAWDOWN: f64 = 0.10;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationTrade {
    pub id: u64,
    pub timestamp: DateTime<Utc>,
    pub symbol: String,
    pub side: String, // "long" or "short"
    pub entry_price: f64,
    pub exit_price: Option<f64>,
    pub quantity: f64,
    pub pnl: Option<f64>,
    pub status: String, // "open", "closed", "cancelled"
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationMetrics {
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub total_trades: u64,
    pub open_trades: u64,
    pub closed_trades: u64,
    pub successful_trades: u64,
    pub failed_trades: u64,
    pub win_rate: f64,
    pub total_profit: f64,
    pub total_loss: f64,
    pub net_profit: f64,
    pub current_capital: f64,
    pub initial_capital: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    pub average_trade_duration: f64,
    pub profit_factor: f64,
    pub days_running: u64,
}

#[derive(Debug, Clone)]
pub struct LiveSimulator {
    trades: Arc<RwLock<HashMap<u64, SimulationTrade>>>,
    metrics: Arc<RwLock<SimulationMetrics>>,
    initial_capital: f64,
    is_running: Arc<RwLock<bool>>,
    next_trade_id: Arc<RwLock<u64>>,
}

impl LiveSimulator {
    pub fn new(initial_capital: f64) -> Self {
        let start_time = Utc::now();
        
        Self {
            trades: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(SimulationMetrics {
                start_time,
                end_time: None,
                total_trades: 0,
                open_trades: 0,
                closed_trades: 0,
                successful_trades: 0,
                failed_trades: 0,
                win_rate: 0.0,
                total_profit: 0.0,
                total_loss: 0.0,
                net_profit: 0.0,
                current_capital: initial_capital,
                initial_capital,
                max_drawdown: 0.0,
                sharpe_ratio: 0.0,
                average_trade_duration: 0.0,
                profit_factor: 0.0,
                days_running: 0,
            })),
            initial_capital,
            is_running: Arc::new(RwLock::new(false)),
            next_trade_id: Arc::new(RwLock::new(1)),
        }
    }

    /// Start live simulation
    pub async fn start(&self) -> TradingResult<()> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Err(TradingError::InvalidInput("Simulation already running".to_string()));
        }
        
        *is_running = true;
        info!("🚀 Live simulation started with ${:.2} capital", self.initial_capital);
        Ok(())
    }

    /// Stop live simulation
    pub async fn stop(&self) -> TradingResult<()> {
        let mut is_running = self.is_running.write().await;
        if !*is_running {
            return Err(TradingError::InvalidInput("Simulation not running".to_string()));
        }
        
        *is_running = false;
        
        let mut metrics = self.metrics.write().await;
        metrics.end_time = Some(Utc::now());
        
        info!("🛑 Live simulation stopped");
        Ok(())
    }

    /// Execute simulated trade
    pub async fn execute_trade(
        &self,
        symbol: &str,
        side: &str,
        entry_price: f64,
        quantity: f64,
        confidence: f64,
    ) -> TradingResult<u64> {
        // Validate confidence threshold
        if confidence < 0.93 {
            return Err(TradingError::InvalidInput(
                format!("Confidence {} below 93% threshold", confidence * 100.0)
            ));
        }
        
        let mut trades = self.trades.write().await;
        let mut metrics = self.metrics.write().await;
        let mut next_id = self.next_trade_id.write().await;
        
        let trade = SimulationTrade {
            id: *next_id,
            timestamp: Utc::now(),
            symbol: symbol.to_string(),
            side: side.to_string(),
            entry_price,
            exit_price: None,
            quantity,
            pnl: None,
            status: "open".to_string(),
            confidence,
        };
        
        trades.insert(*next_id, trade);
        *next_id += 1;
        
        metrics.total_trades += 1;
        metrics.open_trades += 1;
        
        info!("📊 Simulated trade #{}: {} {} {} @ ${:.2}", 
            *next_id - 1, side, quantity, symbol, entry_price);
        
        Ok(*next_id - 1)
    }

    /// Close simulated trade
    pub async fn close_trade(
        &self,
        trade_id: u64,
        exit_price: f64,
    ) -> TradingResult<f64> {
        let mut trades = self.trades.write().await;
        let mut metrics = self.metrics.write().await;
        
        let trade = trades.get_mut(&trade_id)
            .ok_or_else(|| TradingError::InvalidInput("Trade not found".to_string()))?;
        
        if trade.status != "open" {
            return Err(TradingError::InvalidInput("Trade already closed".to_string()));
        }
        
        let price_change = if trade.side == "long" {
            (exit_price - trade.entry_price) / trade.entry_price
        } else {
            (trade.entry_price - exit_price) / trade.entry_price
        };
        
        let pnl = trade.quantity * price_change;
        
        trade.exit_price = Some(exit_price);
        trade.pnl = Some(pnl);
        trade.status = "closed".to_string();
        
        metrics.open_trades -= 1;
        metrics.closed_trades += 1;
        
        if pnl > 0.0 {
            metrics.successful_trades += 1;
            metrics.total_profit += pnl;
        } else {
            metrics.failed_trades += 1;
            metrics.total_loss += pnl.abs();
        }
        
        metrics.net_profit = metrics.total_profit - metrics.total_loss;
        metrics.current_capital = metrics.initial_capital + metrics.net_profit;
        
        // Update win rate
        if metrics.closed_trades > 0 {
            metrics.win_rate = metrics.successful_trades as f64 / metrics.closed_trades as f64;
        }
        
        // Update drawdown
        let current_drawdown = if metrics.current_capital < metrics.initial_capital {
            (metrics.initial_capital - metrics.current_capital) / metrics.initial_capital
        } else {
            0.0
        };
        if current_drawdown > metrics.max_drawdown {
            metrics.max_drawdown = current_drawdown;
        }
        
        // Update days running
        let duration = Utc::now() - metrics.start_time;
        metrics.days_running = duration.num_days() as u64;
        
        info!("✅ Closed trade #{}: P&L ${:.2} ({:.2}%)", 
            trade_id, pnl, price_change * 100.0);
        
        Ok(pnl)
    }

    /// Get current simulation metrics
    pub async fn get_metrics(&self) -> SimulationMetrics {
        self.metrics.read().await.clone()
    }

    /// Check if simulation qualifies for live trading
    pub async fn qualifies_for_live(&self) -> TradingResult<bool> {
        let metrics = self.get_metrics().await;
        
        // Check minimum requirements
        if metrics.closed_trades < MIN_SIM_TRADES {
            return Ok(false);
        }
        
        if metrics.days_running < MIN_SIM_DAYS {
            return Ok(false);
        }
        
        // Check performance metrics
        let qualifies = metrics.win_rate >= REQUIRED_SIM_WIN_RATE
            && metrics.max_drawdown <= MAX_SIM_DRAWDOWN
            && metrics.net_profit > 0.0
            && metrics.profit_factor > 1.5;
        
        Ok(qualifies)
    }

    /// Generate simulation report
    pub async fn generate_report(&self) -> TradingResult<String> {
        let metrics = self.get_metrics().await;
        let trades = self.trades.read().await;
        
        let mut report = String::new();
        report.push_str("╔═══════════════════════════════════════════════════════════════╗\n");
        report.push_str("║           LIVE SIMULATION REPORT                            ║\n");
        report.push_str("╚═══════════════════════════════════════════════════════════════╝\n\n");
        
        report.push_str(&format!("Start Time:        {}\n", metrics.start_time));
        if let Some(end_time) = metrics.end_time {
            report.push_str(&format!("End Time:          {}\n", end_time));
        }
        report.push_str(&format!("Days Running:      {}\n", metrics.days_running));
        report.push_str("\n");
        
        report.push_str("═══════════════════════════════════════════════════════════════\n");
        report.push_str("TRADE METRICS\n");
        report.push_str("═══════════════════════════════════════════════════════════════\n");
        report.push_str(&format!("Total Trades:      {}\n", metrics.total_trades));
        report.push_str(&format!("Open Trades:       {}\n", metrics.open_trades));
        report.push_str(&format!("Closed Trades:     {}\n", metrics.closed_trades));
        report.push_str(&format!("Successful:        {} ({:.1}%)\n", 
            metrics.successful_trades, metrics.win_rate * 100.0));
        report.push_str(&format!("Failed:            {} ({:.1}%)\n", 
            metrics.failed_trades, (1.0 - metrics.win_rate) * 100.0));
        report.push_str("\n");
        
        report.push_str("═══════════════════════════════════════════════════════════════\n");
        report.push_str("FINANCIAL METRICS\n");
        report.push_str("═══════════════════════════════════════════════════════════════\n");
        report.push_str(&format!("Initial Capital:   ${:,.2}\n", metrics.initial_capital));
        report.push_str(&format!("Current Capital:   ${:,.2}\n", metrics.current_capital));
        report.push_str(&format!("Net Profit:        ${:,.2}\n", metrics.net_profit));
        report.push_str(&format!("Total Profit:      ${:,.2}\n", metrics.total_profit));
        report.push_str(&format!("Total Loss:        ${:,.2}\n", metrics.total_loss));
        report.push_str(&format!("Return:            {:.2}%\n", 
            (metrics.net_profit / metrics.initial_capital) * 100.0));
        report.push_str(&format!("Max Drawdown:      {:.2}%\n", metrics.max_drawdown * 100.0));
        report.push_str(&format!("Profit Factor:     {:.2}\n", metrics.profit_factor));
        report.push_str("\n");
        
        report.push_str("═══════════════════════════════════════════════════════════════\n");
        report.push_str("QUALIFICATION STATUS\n");
        report.push_str("═══════════════════════════════════════════════════════════════\n");
        
        let qualifies = self.qualifies_for_live().await.unwrap_or(false);
        if qualifies {
            report.push_str("✅ QUALIFIED FOR LIVE TRADING\n");
        } else {
            report.push_str("❌ NOT YET QUALIFIED FOR LIVE TRADING\n");
            report.push_str("\nRequirements:\n");
            report.push_str(&format!("  - Minimum {} trades: {} / {}\n", 
                MIN_SIM_TRADES, metrics.closed_trades, MIN_SIM_TRADES));
            report.push_str(&format!("  - Minimum {} days: {} / {}\n", 
                MIN_SIM_DAYS, metrics.days_running, MIN_SIM_DAYS));
            report.push_str(&format!("  - Win rate >= {}%: {:.1}%\n", 
                REQUIRED_SIM_WIN_RATE * 100.0, metrics.win_rate * 100.0));
            report.push_str(&format!("  - Max drawdown <= {}%: {:.2}%\n", 
                MAX_SIM_DRAWDOWN * 100.0, metrics.max_drawdown * 100.0));
        }
        
        Ok(report)
    }
}

use log::info;

