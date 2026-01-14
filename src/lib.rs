// Macro Strike Bot Library
// Exposes all necessary modules for the standalone trading engine

pub mod api;
pub mod monitoring;
pub mod strike_optimizer;
pub mod trading_engine;
pub mod opportunity_scanner;
pub mod superior_strike_validator;
pub mod elite_strategies;
pub mod quantum_strategies;
pub mod revolutionary_strategies;
pub mod ultra_fast_cascade;
pub mod advanced_cascade_theory;
pub mod stochastic_volatility_models;
pub mod production_test_suite;
pub mod production_monitor;
pub mod proprietary_predictive_engine;
pub mod quant_strike_system;
pub mod proprietary_backtest;
pub mod elite_quant_framework;
pub mod elite_800k_optimizer;
pub mod hummingbot_array_system;
#[cfg(feature = "eip")]
pub mod amm_predictive_arbitrage;
pub mod errors;
pub mod volume_oscillator_fixed;
pub mod consensus_layer_integration;
pub mod trade_test_harness;
pub mod historical_backtest;
pub mod live_simulation;
#[cfg(feature = "eip")]
pub mod diamond_integration;

#[cfg(feature = "eip")]
pub mod eip;

// Re-export key types and constants
pub use crate::trading_engine::{TradingEngine, EngineConfig, Position};

// Constants
pub const MIN_WIN_PROBABILITY: f64 = 0.90;
pub const STRIKE_FORCE: f64 = 0.15;
pub const PRECISION_THRESHOLD: f64 = 0.90;

// Strike types
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
    pub symbol: String,
    pub strike_type: StrikeType,
    pub entry_price: f64,
    pub target_price: f64,
    pub stop_loss: f64,
    pub confidence: f64,
    pub expected_return: f64,
    pub position_size: f64,
    pub max_exposure_time_ms: u64,
    pub strike_force: f64,
    pub timestamp: u64,
    pub status: StrikeStatus,
    pub hit_time: Option<u64>,
    pub exit_price: Option<f64>,
    pub pnl: Option<f64>,
    pub leverage: u32,
}
