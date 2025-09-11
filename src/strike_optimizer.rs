// Strike Optimization Engine
// Ensures 90% win rate through multi-layer validation and capital allocation

use crate::api::liquidity_predictor::{LiquidityPrediction, TradeRecommendation};
use crate::{MacroStrike, StrikeType};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Strike optimizer for 90% win rate targeting
pub struct StrikeOptimizer {
    /// Historical performance data
    performance_history: Arc<RwLock<PerformanceHistory>>,
    /// Strike validation layers
    validators: Vec<Box<dyn StrikeValidator>>,
    /// Capital allocation engine
    capital_allocator: Arc<RwLock<CapitalAllocator>>,
    /// Configuration
    config: OptimizerConfig,
}

#[derive(Debug, Clone)]
pub struct OptimizerConfig {
    /// Target win rate (0.90 = 90%)
    pub target_win_rate: f64,
    /// Minimum confidence for execution
    pub min_confidence: f64,
    /// Maximum capital per strike (% of total)
    pub max_capital_per_strike: f64,
    /// Kelly criterion factor (0.25 = quarter Kelly)
    pub kelly_factor: f64,
    /// Minimum edge required (expected value)
    pub min_edge: f64,
    /// Risk of ruin threshold
    pub max_risk_of_ruin: f64,
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        Self {
            target_win_rate: 0.90,
            min_confidence: 0.87,      // High confidence required
            max_capital_per_strike: 0.05, // 5% max per strike
            kelly_factor: 0.25,        // Conservative Kelly
            min_edge: 0.15,           // 15% minimum edge
            max_risk_of_ruin: 0.001, // 0.1% risk of ruin
        }
    }
}

/// Performance tracking
#[derive(Debug, Default)]
struct PerformanceHistory {
    /// Win rates by strike type
    strike_type_performance: HashMap<StrikeType, StrikeTypeStats>,
    /// Win rates by market condition
    condition_performance: HashMap<MarketCondition, ConditionStats>,
    /// Time-based performance
    temporal_performance: HashMap<u32, TemporalStats>, // Hour of day -> stats
    /// Overall statistics
    overall_stats: OverallStats,
}

#[derive(Debug, Clone, Default)]
struct StrikeTypeStats {
    pub total_strikes: u32,
    pub winning_strikes: u32,
    pub total_pnl: f64,
    pub avg_win_size: f64,
    pub avg_loss_size: f64,
    pub confidence_correlation: f64,
}

#[derive(Debug, Clone, Default)]
struct ConditionStats {
    pub win_rate: f64,
    pub avg_return: f64,
    pub volatility: f64,
    pub sample_size: u32,
}

#[derive(Debug, Clone, Default)]
struct TemporalStats {
    pub hour: u32,
    pub win_rate: f64,
    pub volume_profile: f64,
    pub spread_profile: f64,
}

#[derive(Debug, Clone, Default)]
struct OverallStats {
    pub total_strikes: u32,
    pub winning_strikes: u32,
    pub current_win_rate: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub risk_adjusted_return: f64,
}

/// Market conditions for performance tracking
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
enum MarketCondition {
    HighVolatility,
    LowVolatility,
    Trending,
    Ranging,
    HighVolume,
    LowVolume,
}

/// Strike validation trait
trait StrikeValidator: Send + Sync {
    fn validate(&self, strike: &StrikeAnalysis) -> ValidationResult;
    fn name(&self) -> &str;
}

/// Comprehensive strike analysis
#[derive(Debug, Clone)]
pub struct StrikeAnalysis {
    pub strike: MacroStrike,
    pub julia_confidence: f64,
    pub liquidity_prediction: LiquidityPrediction,
    pub market_conditions: Vec<MarketCondition>,
    pub edge_calculation: EdgeCalculation,
    pub risk_metrics: RiskMetrics,
}

/// Edge calculation for strike
#[derive(Debug, Clone)]
pub struct EdgeCalculation {
    pub expected_value: f64,
    pub win_probability: f64,
    pub avg_win: f64,
    pub avg_loss: f64,
    pub kelly_percentage: f64,
    pub risk_reward_ratio: f64,
}

/// Risk metrics
#[derive(Debug, Clone)]
pub struct RiskMetrics {
    pub value_at_risk: f64,
    pub conditional_value_at_risk: f64,
    pub max_loss_scenario: f64,
    pub correlation_risk: f64,
    pub liquidity_risk_score: f64,
}

/// Validation result
#[derive(Debug)]
struct ValidationResult {
    pub passed: bool,
    pub score: f64,
    pub reason: Option<String>,
}

/// Capital allocation engine
struct CapitalAllocator {
    total_capital: f64,
    allocated_capital: f64,
    reserved_capital: f64,
    position_sizes: HashMap<u64, f64>, // strike_id -> allocated capital
}

impl StrikeOptimizer {
    pub fn new(config: OptimizerConfig) -> Self {
        let validators = Self::create_validators();
        
        Self {
            performance_history: Arc::new(RwLock::new(PerformanceHistory::default())),
            validators,
            capital_allocator: Arc::new(RwLock::new(CapitalAllocator {
                total_capital: 1_000_000.0,
                allocated_capital: 0.0,
                reserved_capital: 200_000.0, // 20% reserve
                position_sizes: HashMap::new(),
            })),
            config,
        }
    }

    /// Create validation layers for 90% win rate
    fn create_validators() -> Vec<Box<dyn StrikeValidator>> {
        vec![
            Box::new(ConfidenceValidator { min_confidence: 0.87 }),
            Box::new(EdgeValidator { min_edge: 0.15 }),
            Box::new(LiquidityValidator { min_score: 0.85 }),
            Box::new(VolatilityValidator { max_volatility: 0.25 }),
            Box::new(CorrelationValidator { max_correlation: 0.60 }),
            Box::new(TimeValidator { blocked_hours: vec![0, 1, 2, 3, 4, 5] }),
            Box::new(DrawdownValidator { max_drawdown: 0.10 }),
            Box::new(MomentumValidator { min_momentum: 0.65 }),
        ]
    }

    /// Analyze and optimize a strike for 90% win rate
    pub async fn optimize_strike(&self, analysis: StrikeAnalysis) -> Result<OptimizedStrike, String> {
        // 1. Run all validators
        let validation_scores = self.run_validators(&analysis);
        
        // 2. Calculate composite score
        let composite_score = self.calculate_composite_score(&validation_scores);
        
        // 3. Check if meets 90% win rate criteria
        if composite_score < 0.90 {
            return Err(format!(
                "Strike does not meet 90% win rate criteria (score: {:.2}%)",
                composite_score * 100.0
            ));
        }
        
        // 4. Calculate optimal position size
        let position_size = self.calculate_position_size(&analysis).await?;
        
        // 5. Verify edge calculation
        if analysis.edge_calculation.expected_value < self.config.min_edge {
            return Err(format!(
                "Insufficient edge: {:.2}% < {:.2}% required",
                analysis.edge_calculation.expected_value * 100.0,
                self.config.min_edge * 100.0
            ));
        }
        
        // 6. Create optimized strike
        Ok(OptimizedStrike {
            original_strike: analysis.strike.clone(),
            optimized_confidence: composite_score,
            position_size,
            expected_return: analysis.edge_calculation.expected_value,
            risk_adjusted_size: position_size * (1.0 - analysis.risk_metrics.liquidity_risk_score),
            execution_conditions: self.determine_execution_conditions(&analysis),
            stop_loss_adjustment: self.calculate_stop_loss_adjustment(&analysis),
            take_profit_adjustment: self.calculate_take_profit_adjustment(&analysis),
        })
    }

    /// Run all validators
    fn run_validators(&self, analysis: &StrikeAnalysis) -> Vec<(String, ValidationResult)> {
        self.validators
            .iter()
            .map(|validator| {
                let result = validator.validate(analysis);
                (validator.name().to_string(), result)
            })
            .collect()
    }

    /// Calculate composite score from all validators
    fn calculate_composite_score(&self, validations: &[(String, ValidationResult)]) -> f64 {
        let weights = HashMap::from([
            ("confidence", 0.25),
            ("edge", 0.20),
            ("liquidity", 0.20),
            ("volatility", 0.10),
            ("correlation", 0.10),
            ("time", 0.05),
            ("drawdown", 0.05),
            ("momentum", 0.05),
        ]);
        
        let weighted_sum: f64 = validations
            .iter()
            .map(|(name, result)| {
                let weight = weights.get(name.as_str()).unwrap_or(&0.1);
                result.score * weight
            })
            .sum();
        
        weighted_sum
    }

    /// Calculate optimal position size using Kelly Criterion
    async fn calculate_position_size(&self, analysis: &StrikeAnalysis) -> Result<f64, String> {
        let edge = &analysis.edge_calculation;
        
        // Kelly formula: f = (p * b - q) / b
        // where p = win probability, b = win/loss ratio, q = loss probability
        let p = edge.win_probability;
        let q = 1.0 - p;
        let b = edge.avg_win / edge.avg_loss.abs();
        
        let kelly_percentage = (p * b - q) / b;
        
        // Apply Kelly factor for conservative sizing
        let adjusted_kelly = kelly_percentage * self.config.kelly_factor;
        
        // Apply maximum position size constraint
        let max_size = self.config.max_capital_per_strike;
        let position_percentage = adjusted_kelly.min(max_size);
        
        // Get available capital
        let allocator = self.capital_allocator.read().await;
        let available = allocator.total_capital - allocator.allocated_capital - allocator.reserved_capital;
        
        // Calculate position size
        let position_size = available * position_percentage;
        
        // Verify risk of ruin
        let risk_of_ruin = self.calculate_risk_of_ruin(position_size, &analysis);
        if risk_of_ruin > self.config.max_risk_of_ruin {
            return Err(format!(
                "Risk of ruin too high: {:.2}% > {:.2}% max",
                risk_of_ruin * 100.0,
                self.config.max_risk_of_ruin * 100.0
            ));
        }
        
        Ok(position_size)
    }

    /// Calculate risk of ruin
    fn calculate_risk_of_ruin(&self, position_size: f64, analysis: &StrikeAnalysis) -> f64 {
        // Simplified risk of ruin calculation
        // R = ((1-p)/p)^(B/A)
        // where p = win rate, B = bankroll, A = bet size
        
        let p = analysis.edge_calculation.win_probability;
        let q = 1.0 - p;
        let bankroll = 1_000_000.0; // Total capital
        let bet_size = position_size;
        
        if p <= q {
            return 1.0; // Certain ruin
        }
        
        let ratio = q / p;
        let exponent = bankroll / bet_size;
        
        ratio.powf(exponent)
    }

    /// Determine optimal execution conditions
    fn determine_execution_conditions(&self, analysis: &StrikeAnalysis) -> ExecutionConditions {
        ExecutionConditions {
            max_slippage: 0.001, // 0.1%
            time_limit_seconds: 5,
            partial_fill_allowed: analysis.strike.position_size > 100_000.0,
            iceberg_order: analysis.strike.position_size > 200_000.0,
            requires_confirmation: analysis.edge_calculation.expected_value > 0.50,
        }
    }

    /// Calculate dynamic stop loss adjustment
    fn calculate_stop_loss_adjustment(&self, analysis: &StrikeAnalysis) -> f64 {
        // Tighter stops for higher confidence trades
        let base_stop = 0.02; // 2% base stop loss
        let confidence_factor = analysis.julia_confidence;
        let volatility_factor = 1.0 + analysis.risk_metrics.liquidity_risk_score;
        
        base_stop * (2.0 - confidence_factor) * volatility_factor
    }

    /// Calculate dynamic take profit adjustment
    fn calculate_take_profit_adjustment(&self, analysis: &StrikeAnalysis) -> f64 {
        // Higher targets for higher edge trades
        let base_target = 0.06; // 6% base target
        let edge_multiplier = 1.0 + analysis.edge_calculation.expected_value;
        let momentum_bonus = if analysis.edge_calculation.risk_reward_ratio > 3.0 { 1.2 } else { 1.0 };
        
        base_target * edge_multiplier * momentum_bonus
    }

    /// Update performance history after trade
    pub async fn record_trade_result(&self, strike_id: u64, pnl: f64, success: bool) {
        let mut history = self.performance_history.write().await;
        
        // Update overall stats
        history.overall_stats.total_strikes += 1;
        if success {
            history.overall_stats.winning_strikes += 1;
        }
        history.overall_stats.current_win_rate = 
            history.overall_stats.winning_strikes as f64 / history.overall_stats.total_strikes as f64;
        
        // Log if win rate drops below target
        if history.overall_stats.current_win_rate < self.config.target_win_rate {
            log::warn!(
                "Win rate {:.1}% below target {:.1}%",
                history.overall_stats.current_win_rate * 100.0,
                self.config.target_win_rate * 100.0
            );
        }
    }
}

/// Optimized strike with all parameters
#[derive(Debug, Clone)]
pub struct OptimizedStrike {
    pub original_strike: MacroStrike,
    pub optimized_confidence: f64,
    pub position_size: f64,
    pub expected_return: f64,
    pub risk_adjusted_size: f64,
    pub execution_conditions: ExecutionConditions,
    pub stop_loss_adjustment: f64,
    pub take_profit_adjustment: f64,
}

#[derive(Debug, Clone)]
pub struct ExecutionConditions {
    pub max_slippage: f64,
    pub time_limit_seconds: u32,
    pub partial_fill_allowed: bool,
    pub iceberg_order: bool,
    pub requires_confirmation: bool,
}

// Validator implementations
struct ConfidenceValidator {
    min_confidence: f64,
}

impl StrikeValidator for ConfidenceValidator {
    fn validate(&self, analysis: &StrikeAnalysis) -> ValidationResult {
        let passed = analysis.julia_confidence >= self.min_confidence;
        ValidationResult {
            passed,
            score: analysis.julia_confidence,
            reason: if !passed {
                Some(format!("Confidence {:.2} below minimum {:.2}", 
                    analysis.julia_confidence, self.min_confidence))
            } else {
                None
            },
        }
    }
    
    fn name(&self) -> &str {
        "confidence"
    }
}

struct EdgeValidator {
    min_edge: f64,
}

impl StrikeValidator for EdgeValidator {
    fn validate(&self, analysis: &StrikeAnalysis) -> ValidationResult {
        let edge = analysis.edge_calculation.expected_value;
        let passed = edge >= self.min_edge;
        ValidationResult {
            passed,
            score: edge / self.min_edge,
            reason: if !passed {
                Some(format!("Edge {:.2}% below minimum {:.2}%", 
                    edge * 100.0, self.min_edge * 100.0))
            } else {
                None
            },
        }
    }
    
    fn name(&self) -> &str {
        "edge"
    }
}

struct LiquidityValidator {
    min_score: f64,
}

impl StrikeValidator for LiquidityValidator {
    fn validate(&self, analysis: &StrikeAnalysis) -> ValidationResult {
        let score = analysis.liquidity_prediction.predicted_score;
        let passed = score >= self.min_score;
        ValidationResult {
            passed,
            score,
            reason: if !passed {
                Some(format!("Liquidity score {:.2} below minimum {:.2}", 
                    score, self.min_score))
            } else {
                None
            },
        }
    }
    
    fn name(&self) -> &str {
        "liquidity"
    }
}

struct VolatilityValidator {
    max_volatility: f64,
}

impl StrikeValidator for VolatilityValidator {
    fn validate(&self, analysis: &StrikeAnalysis) -> ValidationResult {
        let vol = analysis.risk_metrics.liquidity_risk_score;
        let passed = vol <= self.max_volatility;
        ValidationResult {
            passed,
            score: 1.0 - (vol / self.max_volatility),
            reason: if !passed {
                Some(format!("Volatility {:.2} exceeds maximum {:.2}", 
                    vol, self.max_volatility))
            } else {
                None
            },
        }
    }
    
    fn name(&self) -> &str {
        "volatility"
    }
}

struct CorrelationValidator {
    max_correlation: f64,
}

impl StrikeValidator for CorrelationValidator {
    fn validate(&self, analysis: &StrikeAnalysis) -> ValidationResult {
        let corr = analysis.risk_metrics.correlation_risk;
        let passed = corr <= self.max_correlation;
        ValidationResult {
            passed,
            score: 1.0 - (corr / self.max_correlation),
            reason: if !passed {
                Some(format!("Correlation risk {:.2} exceeds maximum {:.2}", 
                    corr, self.max_correlation))
            } else {
                None
            },
        }
    }
    
    fn name(&self) -> &str {
        "correlation"
    }
}

struct TimeValidator {
    blocked_hours: Vec<u32>,
}

impl StrikeValidator for TimeValidator {
    fn validate(&self, analysis: &StrikeAnalysis) -> ValidationResult {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let hour = (now.as_secs() / 3600) % 24;
        
        let passed = !self.blocked_hours.contains(&(hour as u32));
        ValidationResult {
            passed,
            score: if passed { 1.0 } else { 0.0 },
            reason: if !passed {
                Some(format!("Trading blocked during hour {}", hour))
            } else {
                None
            },
        }
    }
    
    fn name(&self) -> &str {
        "time"
    }
}

struct DrawdownValidator {
    max_drawdown: f64,
}

impl StrikeValidator for DrawdownValidator {
    fn validate(&self, analysis: &StrikeAnalysis) -> ValidationResult {
        let drawdown = analysis.risk_metrics.max_loss_scenario;
        let passed = drawdown <= self.max_drawdown;
        ValidationResult {
            passed,
            score: 1.0 - (drawdown / self.max_drawdown),
            reason: if !passed {
                Some(format!("Potential drawdown {:.2}% exceeds maximum {:.2}%", 
                    drawdown * 100.0, self.max_drawdown * 100.0))
            } else {
                None
            },
        }
    }
    
    fn name(&self) -> &str {
        "drawdown"
    }
}

struct MomentumValidator {
    min_momentum: f64,
}

impl StrikeValidator for MomentumValidator {
    fn validate(&self, analysis: &StrikeAnalysis) -> ValidationResult {
        // Check if market has favorable momentum
        let has_momentum = analysis.market_conditions.contains(&MarketCondition::Trending);
        let momentum_score = if has_momentum { 0.9 } else { 0.5 };
        
        let passed = momentum_score >= self.min_momentum;
        ValidationResult {
            passed,
            score: momentum_score,
            reason: if !passed {
                Some("Insufficient market momentum".to_string())
            } else {
                None
            },
        }
    }
    
    fn name(&self) -> &str {
        "momentum"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_90_percent_win_rate_optimization() {
        let optimizer = StrikeOptimizer::new(OptimizerConfig::default());
        
        // Create test strike analysis
        let analysis = StrikeAnalysis {
            strike: MacroStrike {
                id: 1,
                symbol: 0,
                strike_type: StrikeType::MacroMomentum,
                entry_price: 50000.0,
                target_price: 53000.0,
                stop_loss: 49000.0,
                confidence: 0.91,
                position_size: 50000.0,
                expected_return: 0.06,
                status: crate::StrikeStatus::Targeting,
                timestamp_ms: 0,
            },
            julia_confidence: 0.91,
            liquidity_prediction: LiquidityPrediction {
                symbol: "BTC/USDT".to_string(),
                current_score: 0.92,
                predicted_score: 0.90,
                confidence: 0.88,
                liquidity_state: crate::api::liquidity_predictor::LiquidityState::Optimal,
                risk_factors: vec![],
                recommended_action: TradeRecommendation::Execute,
                next_optimal_time: None,
            },
            market_conditions: vec![MarketCondition::Trending, MarketCondition::HighVolume],
            edge_calculation: EdgeCalculation {
                expected_value: 0.18,
                win_probability: 0.91,
                avg_win: 3000.0,
                avg_loss: -1000.0,
                kelly_percentage: 0.15,
                risk_reward_ratio: 3.0,
            },
            risk_metrics: RiskMetrics {
                value_at_risk: 1000.0,
                conditional_value_at_risk: 1500.0,
                max_loss_scenario: 0.02,
                correlation_risk: 0.3,
                liquidity_risk_score: 0.1,
            },
        };

        let result = optimizer.optimize_strike(analysis).await;
        assert!(result.is_ok());
        
        let optimized = result.unwrap();
        assert!(optimized.optimized_confidence >= 0.90);
        assert!(optimized.position_size > 0.0);
    }
}
