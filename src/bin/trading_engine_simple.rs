use chrono::Utc;
use log::{error, info, warn};
use std::sync::Arc;
use tokio::sync::RwLock;

// Import the trading engine and required modules
use macro_strike_bot_fixed::{
    api::{
        coingecko::CoinGeckoClient,
        kraken::KrakenClient,
        liquidity::LiquidityMonitor,
        liquidity_predictor::{LiquidityPredictor, PredictorConfig},
        safety::{SafetyConfig, SafetyMonitor},
        ApiConfig, MarketDataProvider, TradingExchange,
    },
    elite_strategies::EliteStrategyEngine,
    monitoring::{MetricType, MonitoringSystem},
    opportunity_scanner::OpportunityScanner,
    superior_strike_validator::{SuperiorStrikeValidator, ValidationConfig},
    trading_engine::{EngineConfig, TradingEngine},
    MIN_WIN_PROBABILITY,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    info!("🏎️ FERRARI MODE - Elite Trading Engine Starting...");

    // Load configuration from environment
    let kraken_api_key = std::env::var("KRAKEN_API_KEY").unwrap_or_default();
    let kraken_api_secret = std::env::var("KRAKEN_API_SECRET").unwrap_or_default();
    let initial_capital = std::env::var("INITIAL_CAPITAL")
        .unwrap_or_else(|_| "100000.0".to_string())
        .parse::<f64>()?;
    let dry_run = std::env::var("DRY_RUN")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()?;

    info!("Configuration loaded:");
    info!("  Initial Capital: ${}", initial_capital);
    info!("  Dry Run: {}", dry_run);
    info!("  Min Win Probability: {}%", MIN_WIN_PROBABILITY * 100.0);

    // Initialize API clients
    let kraken_config = ApiConfig {
        api_key: kraken_api_key,
        api_secret: kraken_api_secret,
        testnet: dry_run,
        rate_limit_per_minute: 60,
    };

    let kraken_client = Arc::new(KrakenClient::new(kraken_config)) as Arc<dyn TradingExchange>;

    let coingecko_config = ApiConfig {
        api_key: std::env::var("COINGECKO_API_KEY").unwrap_or_default(),
        api_secret: String::new(),
        testnet: false,
        rate_limit_per_minute: 30,
    };

    let coingecko_client =
        Arc::new(CoinGeckoClient::new(coingecko_config)) as Arc<dyn MarketDataProvider>;

    // Initialize trading engine
    let engine_config = EngineConfig {
        max_positions: 5,
        position_size_pct: 0.05,
        stop_loss_pct: 0.02,
        take_profit_pct: 0.06,
        trailing_stop: true,
        use_liquidity_prediction: true,
        min_confidence: MIN_WIN_PROBABILITY,
    };

    let engine = TradingEngine::new(
        kraken_client.clone(),
        coingecko_client.clone(),
        engine_config,
    );

    // Initialize components
    let opportunity_scanner = Arc::new(OpportunityScanner::new(
        coingecko_client.clone(),
        MIN_WIN_PROBABILITY,
    ));

    let liquidity_monitor = Arc::new(LiquidityMonitor::new());
    let liquidity_predictor = Arc::new(LiquidityPredictor::new(PredictorConfig::default()));
    let safety_monitor = Arc::new(SafetyMonitor::new(SafetyConfig::default()));

    let validation_config = ValidationConfig {
        parallel_execution: true,
        fail_fast: false,
        timeout_ms: 1000,
        min_confidence: MIN_WIN_PROBABILITY,
    };

    let strike_validator = Arc::new(SuperiorStrikeValidator::new(
        validation_config,
        liquidity_monitor.clone(),
        liquidity_predictor.clone(),
        safety_monitor.clone(),
    ));

    let elite_strategies = Arc::new(EliteStrategyEngine::new(coingecko_client.clone()));
    let monitoring = Arc::new(MonitoringSystem::new());

    info!("All components initialized successfully");

    // Trading loop
    let symbols = vec![
        "BTC/USDT",
        "ETH/USDT",
        "SOL/USDT",
        "LINK/USDT",
        "AVAX/USDT",
        "MATIC/USDT",
        "DOT/USDT",
        "UNI/USDT",
    ];

    info!("Starting Ferrari engine with {} symbols", symbols.len());
    info!("🏁 Engines are HOT! Trading loop active...");

    let mut consecutive_errors = 0;

    loop {
        for symbol in &symbols {
            // Generate signals from elite strategies
            if let Some(strike) = elite_strategies.generate_strike(symbol).await {
                info!("🎯 Elite signal for {}: {:?}", symbol, strike.strike_type);

                // Validate with Superior validator
                let validation_report = strike_validator.validate(&strike, &kraken_client).await;

                if matches!(validation_report.decision, 
                           macro_strike_bot_fixed::superior_strike_validator::ValidationDecision::Approved { .. } |
                           macro_strike_bot_fixed::superior_strike_validator::ValidationDecision::ConditionallyApproved { .. }) {
                    info!("✅ Strike validated! Confidence: {:.1}%", 
                          strike.confidence * 100.0);
                    
                    if !dry_run {
                        // Execute trade
                        match engine.execute_strike(&strike).await {
                            Ok(pnl) => {
                                info!("💰 Trade executed! P&L: ${:.2}", pnl);
                                monitoring.record_metric(MetricType::TradeExecuted, 1.0).await;
                                consecutive_errors = 0;
                            }
                            Err(e) => {
                                error!("❌ Trade execution failed: {}", e);
                                consecutive_errors += 1;
                            }
                        }
                    } else {
                        info!("📋 DRY RUN: Would execute {} on {} with confidence {:.1}%",
                              match strike.strike_type {
                                  _ => "trade"
                              },
                              symbol,
                              strike.confidence * 100.0);
                    }
                } else {
                    warn!("❌ Strike rejected: {:?}", validation_report.rejection_reasons);
                }
            }
        }

        // Check for circuit breaker
        if consecutive_errors >= 5 {
            error!("🛑 Circuit breaker triggered! Too many consecutive errors.");
            break;
        }

        // Brief pause between cycles
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }

    info!("Ferrari engine stopped");
    Ok(())
}
