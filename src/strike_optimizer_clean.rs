// Strike optimizer with advanced features
// Now uses SuperiorStrikeValidator for all validations

use crate::api::{MarketDataProvider, OrderBook};
use crate::api::liquidity::TradingPair;
use crate::api::liquidity_predictor::{LiquidityPredictor, TradeRecommendation};
use crate::monitoring::{MonitoringSystem, MetricType};
use crate::{MacroStrike, StrikeType};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use log::{info, warn, error};

/// Strike optimizer for finding the best trading opportunities
pub struct StrikeOptimizer {
    /// Market data provider
    market_data: Arc<dyn MarketDataProvider>,
    
    /// Liquidity predictor
    liquidity_predictor: Arc<LiquidityPredictor>,
    
    /// Monitoring system
    monitoring: Arc<MonitoringSystem>,
    
    /// Configuration
    config: OptimizerConfig,
    
    /// Julia analysis results cache
    julia_cache: Arc<RwLock<HashMap<String, JuliaAnalysis>>>,
}

#[derive(Debug, Clone)]
pub struct OptimizerConfig {
    /// Minimum win probability required
    pub min_win_probability: f64,
    
    /// Maximum position size as percentage of capital
    pub max_position_pct: f64,
    
    /// Profit target percentage
    pub profit_target_pct: f64,
    
    /// Stop loss percentage
    pub stop_loss_pct: f64,
    
    /// Maximum correlation allowed between positions
    pub max_correlation: f64,
    
    /// Minimum edge required
    pub min_edge: f64,
    
    /// Maximum volatility allowed
    pub max_volatility: f64,
    
    /// Julia integration enabled
    pub use_julia_analysis: bool,
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        Self {
            min_win_probability: 0.90,
            max_position_pct: 0.05,
            profit_target_pct: 0.06,
            stop_loss_pct: 0.02,
            max_correlation: 0.7,
            min_edge: 0.02,
            max_volatility: 0.5,
            use_julia_analysis: true,
        }
    }
}

/// Julia analysis integration
#[derive(Debug, Clone)]
pub struct JuliaAnalysis {
    pub symbol: String,
    pub julia_confidence: f64,
    pub predicted_return: f64,
    pub risk_score: f64,
    pub optimal_position_size: f64,
    pub entry_timing_score: f64,
    pub market_regime: String,
    pub feature_importance: HashMap<String, f64>,
}

/// Comprehensive strike analysis
#[derive(Debug, Clone)]
pub struct StrikeAnalysis {
    pub strike: MacroStrike,
    pub julia_analysis: JuliaAnalysis,
    pub liquidity_prediction: TradeRecommendation,
    pub edge_calculation: EdgeCalculation,
    pub risk_metrics: RiskMetrics,
    pub optimal_sizing: PositionSizing,
    pub execution_conditions: ExecutionConditions,
    pub market_conditions: Vec<MarketCondition>,
    pub julia_confidence: f64,
}

impl StrikeOptimizer {
    pub fn new(
        market_data: Arc<dyn MarketDataProvider>,
        liquidity_predictor: Arc<LiquidityPredictor>,
        monitoring: Arc<MonitoringSystem>,
        config: OptimizerConfig,
    ) -> Self {
        Self {
            market_data,
            liquidity_predictor,
            monitoring,
            config,
            julia_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Optimize a potential strike
    pub async fn optimize_strike(&self, strike: &MacroStrike) -> StrikeAnalysis {
        info!("Optimizing strike for {} with type {:?}", strike.symbol, strike.strike_type);
        
        // Get market data
        let market_data = self.market_data.get_market_data(&strike.symbol).await
            .expect("Failed to get market data");
        
        // Get liquidity prediction
        let liquidity_prediction = self.liquidity_predictor
            .predict_liquidity(&strike.symbol, strike.position_size).await;
        
        // Get Julia analysis
        let julia_analysis = if self.config.use_julia_analysis {
            self.get_julia_analysis(&strike.symbol).await
        } else {
            self.create_default_julia_analysis(&strike.symbol)
        };
        
        // Calculate edge
        let edge_calculation = self.calculate_edge(strike, &market_data.price);
        
        // Calculate risk metrics
        let risk_metrics = self.calculate_risk_metrics(strike, &liquidity_prediction);
        
        // Determine optimal position sizing
        let optimal_sizing = self.calculate_optimal_sizing(
            strike,
            &julia_analysis,
            &edge_calculation,
            &risk_metrics,
        );
        
        // Determine execution conditions
        let execution_conditions = self.determine_execution_conditions(
            strike,
            &liquidity_prediction,
        );
        
        // Analyze market conditions
        let market_conditions = self.analyze_market_conditions(&market_data);
        
        // Build comprehensive analysis
        let analysis = StrikeAnalysis {
            julia_confidence: julia_analysis.julia_confidence,
            strike: strike.clone(),
            julia_analysis,
            liquidity_prediction,
            edge_calculation,
            risk_metrics,
            optimal_sizing,
            execution_conditions,
            market_conditions,
        };
        
        // Record metrics
        self.record_optimization_metrics(&analysis).await;
        
        analysis
    }
    
    async fn get_julia_analysis(&self, symbol: &str) -> JuliaAnalysis {
        // Check cache first
        let cache = self.julia_cache.read().await;
        if let Some(cached) = cache.get(symbol) {
            if cached.julia_confidence > 0.0 {
                return cached.clone();
            }
        }
        drop(cache);
        
        // Call Julia process (placeholder)
        // In production, this would make an actual call to Julia
        let julia_result = self.call_julia_analysis(symbol).await;
        
        // Cache result
        let mut cache = self.julia_cache.write().await;
        cache.insert(symbol.to_string(), julia_result.clone());
        
        julia_result
    }
    
    async fn call_julia_analysis(&self, symbol: &str) -> JuliaAnalysis {
        // Placeholder for Julia integration
        // In production, this would:
        // 1. Send request to Julia process
        // 2. Wait for analysis
        // 3. Parse results
        
        JuliaAnalysis {
            symbol: symbol.to_string(),
            julia_confidence: 0.92,
            predicted_return: 0.065,
            risk_score: 0.23,
            optimal_position_size: 0.04,
            entry_timing_score: 0.87,
            market_regime: "trending".to_string(),
            feature_importance: HashMap::from([
                ("volatility".to_string(), 0.31),
                ("momentum".to_string(), 0.28),
                ("liquidity".to_string(), 0.22),
                ("correlation".to_string(), 0.19),
            ]),
        }
    }
    
    fn create_default_julia_analysis(&self, symbol: &str) -> JuliaAnalysis {
        JuliaAnalysis {
            symbol: symbol.to_string(),
            julia_confidence: 0.85,
            predicted_return: 0.05,
            risk_score: 0.30,
            optimal_position_size: 0.03,
            entry_timing_score: 0.75,
            market_regime: "unknown".to_string(),
            feature_importance: HashMap::new(),
        }
    }
    
    fn calculate_edge(&self, strike: &MacroStrike, current_price: &f64) -> EdgeCalculation {
        let price_move = (strike.target_price - current_price) / current_price;
        let win_probability = strike.confidence;
        let loss_probability = 1.0 - win_probability;
        
        let expected_value = (price_move * win_probability) - 
                           (self.config.stop_loss_pct * loss_probability);
        
        let kelly_fraction = (win_probability - loss_probability) / 
                           (price_move / self.config.stop_loss_pct);
        
        EdgeCalculation {
            expected_value,
            kelly_fraction: kelly_fraction.max(0.0).min(0.25), // Cap at 25%
            sharpe_ratio: expected_value / self.config.max_volatility,
            risk_reward_ratio: price_move / self.config.stop_loss_pct,
        }
    }
    
    fn calculate_risk_metrics(
        &self,
        strike: &MacroStrike,
        liquidity: &TradeRecommendation,
    ) -> RiskMetrics {
        RiskMetrics {
            liquidity_risk_score: 1.0 - liquidity.confidence,
            slippage_estimate: liquidity.expected_slippage,
            volatility_risk: 0.25, // Placeholder
            correlation_risk: 0.15, // Placeholder
            max_loss_scenario: strike.position_size * self.config.stop_loss_pct,
            time_decay_risk: 0.1, // Placeholder
        }
    }
    
    fn calculate_optimal_sizing(
        &self,
        strike: &MacroStrike,
        julia: &JuliaAnalysis,
        edge: &EdgeCalculation,
        risk: &RiskMetrics,
    ) -> PositionSizing {
        // Use multiple methods and take the minimum
        let kelly_size = edge.kelly_fraction;
        let julia_size = julia.optimal_position_size;
        let risk_adjusted_size = (1.0 - risk.liquidity_risk_score) * self.config.max_position_pct;
        let volatility_adjusted_size = self.config.max_position_pct * 
                                     (1.0 - risk.volatility_risk);
        
        let recommended_size = kelly_size
            .min(julia_size)
            .min(risk_adjusted_size)
            .min(volatility_adjusted_size)
            .min(self.config.max_position_pct);
        
        PositionSizing {
            recommended_size,
            kelly_size,
            julia_size,
            risk_adjusted_size,
            volatility_adjusted_size,
            confidence_bands: (recommended_size * 0.8, recommended_size * 1.2),
        }
    }
    
    fn determine_execution_conditions(
        &self,
        strike: &MacroStrike,
        liquidity: &TradeRecommendation,
    ) -> ExecutionConditions {
        ExecutionConditions {
            max_slippage: liquidity.expected_slippage * 2.0,
            time_limit_seconds: 30,
            partial_fill_allowed: liquidity.confidence < 0.9,
            iceberg_order: strike.position_size > 100_000.0,
            requires_confirmation: strike.confidence < 0.95,
        }
    }
    
    fn analyze_market_conditions(&self, market_data: &crate::api::MarketData) -> Vec<MarketCondition> {
        let mut conditions = Vec::new();
        
        if market_data.price_change_24h.abs() > 0.05 {
            conditions.push(MarketCondition::HighVolatility);
        }
        
        if market_data.price_change_24h > 0.03 {
            conditions.push(MarketCondition::Trending);
        }
        
        if market_data.volume_24h > 1_000_000_000.0 {
            conditions.push(MarketCondition::HighLiquidity);
        }
        
        conditions
    }
    
    async fn record_optimization_metrics(&self, analysis: &StrikeAnalysis) {
        self.monitoring.record_metric(
            MetricType::StrikeOptimized,
            1.0,
            Some(HashMap::from([
                ("symbol".to_string(), analysis.strike.symbol.clone()),
                ("confidence".to_string(), analysis.strike.confidence.to_string()),
                ("edge".to_string(), analysis.edge_calculation.expected_value.to_string()),
                ("julia_confidence".to_string(), analysis.julia_analysis.julia_confidence.to_string()),
            ])),
        ).await;
    }
}

// Supporting types
#[derive(Debug, Clone)]
pub struct EdgeCalculation {
    pub expected_value: f64,
    pub kelly_fraction: f64,
    pub sharpe_ratio: f64,
    pub risk_reward_ratio: f64,
}

#[derive(Debug, Clone)]
pub struct RiskMetrics {
    pub liquidity_risk_score: f64,
    pub slippage_estimate: f64,
    pub volatility_risk: f64,
    pub correlation_risk: f64,
    pub max_loss_scenario: f64,
    pub time_decay_risk: f64,
}

#[derive(Debug, Clone)]
pub struct PositionSizing {
    pub recommended_size: f64,
    pub kelly_size: f64,
    pub julia_size: f64,
    pub risk_adjusted_size: f64,
    pub volatility_adjusted_size: f64,
    pub confidence_bands: (f64, f64),
}

#[derive(Debug, Clone)]
pub enum MarketCondition {
    Trending,
    Ranging,
    HighVolatility,
    LowVolatility,
    HighLiquidity,
    LowLiquidity,
    NewsEvent,
}

#[derive(Debug, Clone)]
pub struct ExecutionConditions {
    pub max_slippage: f64,
    pub time_limit_seconds: u32,
    pub partial_fill_allowed: bool,
    pub iceberg_order: bool,
    pub requires_confirmation: bool,
}

// All validation logic has been moved to SuperiorStrikeValidator
// This ensures consistent validation across the entire system
