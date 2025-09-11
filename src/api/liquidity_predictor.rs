// Predictive Liquidity Analysis Module
// Predicts future liquidity conditions and prevents trades during low liquidity periods

use super::{ApiResult, liquidity::{LiquidityMetrics, TradingPair}};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Liquidity prediction model
#[derive(Debug, Clone)]
pub struct LiquidityPredictor {
    /// Historical liquidity data
    history: Arc<RwLock<HashMap<String, VecDeque<TimedLiquidityData>>>>,
    /// Prediction models per symbol
    models: Arc<RwLock<HashMap<String, LiquidityModel>>>,
    /// Configuration
    config: PredictorConfig,
}

#[derive(Debug, Clone)]
pub struct PredictorConfig {
    /// Minimum historical data points required
    pub min_history_points: usize,
    /// Prediction horizon in minutes
    pub prediction_horizon_minutes: u64,
    /// Minimum predicted liquidity score (0-1)
    pub min_liquidity_score: f64,
    /// Liquidity score thresholds
    pub critical_threshold: f64,
    pub warning_threshold: f64,
    pub optimal_threshold: f64,
}

impl Default for PredictorConfig {
    fn default() -> Self {
        Self {
            min_history_points: 100,
            prediction_horizon_minutes: 30,
            min_liquidity_score: 0.7,
            critical_threshold: 0.5,
            warning_threshold: 0.7,
            optimal_threshold: 0.85,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimedLiquidityData {
    pub metrics: LiquidityMetrics,
    pub timestamp: SystemTime,
    pub liquidity_score: f64,
}

#[derive(Debug, Clone)]
pub struct LiquidityModel {
    /// Moving averages of liquidity score
    pub ma_5min: f64,
    pub ma_15min: f64,
    pub ma_1hour: f64,
    /// Volatility of liquidity
    pub liquidity_volatility: f64,
    /// Time-based patterns (hour of day -> typical liquidity)
    pub hourly_patterns: [f64; 24],
    /// Day of week patterns (0=Sunday -> typical liquidity)
    pub daily_patterns: [f64; 7],
}

/// Liquidity prediction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityPrediction {
    pub symbol: String,
    pub current_score: f64,
    pub predicted_score: f64,
    pub confidence: f64,
    pub liquidity_state: LiquidityState,
    pub risk_factors: Vec<String>,
    pub recommended_action: TradeRecommendation,
    pub next_optimal_time: Option<SystemTime>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LiquidityState {
    Optimal,      // Best liquidity conditions
    Good,         // Acceptable liquidity
    Warning,      // Marginal liquidity
    Critical,     // Poor liquidity
    Insufficient, // Do not trade
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TradeRecommendation {
    Execute,           // Safe to trade
    ReduceSize,        // Trade with reduced size
    WaitForLiquidity,  // Delay trade
    Abort,             // Cancel trade
}

impl LiquidityPredictor {
    pub fn new(config: PredictorConfig) -> Self {
        Self {
            history: Arc::new(RwLock::new(HashMap::new())),
            models: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Record liquidity metrics
    pub async fn record_metrics(&self, symbol: &str, metrics: LiquidityMetrics) {
        let score = self.calculate_liquidity_score(&metrics);
        let timed_data = TimedLiquidityData {
            metrics,
            timestamp: SystemTime::now(),
            liquidity_score: score,
        };

        let mut history = self.history.write().await;
        let symbol_history = history.entry(symbol.to_string())
            .or_insert_with(|| VecDeque::with_capacity(1000));
        
        symbol_history.push_back(timed_data);
        
        // Keep only recent history (24 hours)
        while symbol_history.len() > 1440 { // 1 per minute for 24 hours
            symbol_history.pop_front();
        }

        // Update model
        drop(history);
        self.update_model(symbol).await;
    }

    /// Calculate liquidity score (0-1)
    fn calculate_liquidity_score(&self, metrics: &LiquidityMetrics) -> f64 {
        let mut score = 0.0;
        let mut weight_sum = 0.0;

        // Volume score (40% weight)
        let volume_score = (metrics.volume_24h_usd / 10_000_000.0).min(1.0);
        score += volume_score * 0.4;
        weight_sum += 0.4;

        // Depth score (30% weight)
        let depth_score = ((metrics.bid_depth_usd + metrics.ask_depth_usd) / 1_000_000.0).min(1.0);
        score += depth_score * 0.3;
        weight_sum += 0.3;

        // Spread score (20% weight) - lower is better
        let spread_score = 1.0 - (metrics.spread_percent / 1.0).min(1.0);
        score += spread_score * 0.2;
        weight_sum += 0.2;

        // Market maker score (10% weight)
        let maker_score = (metrics.market_maker_count as f64 / 10.0).min(1.0);
        score += maker_score * 0.1;
        weight_sum += 0.1;

        score / weight_sum
    }

    /// Update prediction model for a symbol
    async fn update_model(&self, symbol: &str) {
        let history = self.history.read().await;
        if let Some(symbol_history) = history.get(symbol) {
            if symbol_history.len() < self.config.min_history_points {
                return; // Not enough data
            }

            // Calculate moving averages
            let now = SystemTime::now();
            let ma_5min = self.calculate_ma(symbol_history, Duration::from_secs(300));
            let ma_15min = self.calculate_ma(symbol_history, Duration::from_secs(900));
            let ma_1hour = self.calculate_ma(symbol_history, Duration::from_secs(3600));

            // Calculate volatility
            let volatility = self.calculate_volatility(symbol_history);

            // Extract patterns
            let hourly_patterns = self.extract_hourly_patterns(symbol_history);
            let daily_patterns = self.extract_daily_patterns(symbol_history);

            let model = LiquidityModel {
                ma_5min,
                ma_15min,
                ma_1hour,
                liquidity_volatility: volatility,
                hourly_patterns,
                daily_patterns,
            };

            let mut models = self.models.write().await;
            models.insert(symbol.to_string(), model);
        }
    }

    /// Calculate moving average
    fn calculate_ma(&self, history: &VecDeque<TimedLiquidityData>, window: Duration) -> f64 {
        let cutoff = SystemTime::now() - window;
        let values: Vec<f64> = history.iter()
            .filter(|d| d.timestamp > cutoff)
            .map(|d| d.liquidity_score)
            .collect();
        
        if values.is_empty() {
            return 0.0;
        }
        
        values.iter().sum::<f64>() / values.len() as f64
    }

    /// Calculate liquidity volatility
    fn calculate_volatility(&self, history: &VecDeque<TimedLiquidityData>) -> f64 {
        if history.len() < 2 {
            return 0.0;
        }

        let scores: Vec<f64> = history.iter()
            .map(|d| d.liquidity_score)
            .collect();
        
        let mean = scores.iter().sum::<f64>() / scores.len() as f64;
        let variance = scores.iter()
            .map(|s| (s - mean).powi(2))
            .sum::<f64>() / scores.len() as f64;
        
        variance.sqrt()
    }

    /// Extract hourly patterns
    fn extract_hourly_patterns(&self, history: &VecDeque<TimedLiquidityData>) -> [f64; 24] {
        let mut hourly_sums = [0.0; 24];
        let mut hourly_counts = [0; 24];

        for data in history {
            if let Ok(duration) = data.timestamp.duration_since(SystemTime::UNIX_EPOCH) {
                let hour = (duration.as_secs() / 3600) % 24;
                hourly_sums[hour as usize] += data.liquidity_score;
                hourly_counts[hour as usize] += 1;
            }
        }

        let mut patterns = [0.0; 24];
        for i in 0..24 {
            if hourly_counts[i] > 0 {
                patterns[i] = hourly_sums[i] / hourly_counts[i] as f64;
            }
        }
        
        patterns
    }

    /// Extract daily patterns
    fn extract_daily_patterns(&self, history: &VecDeque<TimedLiquidityData>) -> [f64; 7] {
        let mut daily_sums = [0.0; 7];
        let mut daily_counts = [0; 7];

        for data in history {
            if let Ok(duration) = data.timestamp.duration_since(SystemTime::UNIX_EPOCH) {
                let day = ((duration.as_secs() / 86400) + 4) % 7; // +4 because epoch was Thursday
                daily_sums[day as usize] += data.liquidity_score;
                daily_counts[day as usize] += 1;
            }
        }

        let mut patterns = [0.0; 7];
        for i in 0..7 {
            if daily_counts[i] > 0 {
                patterns[i] = daily_sums[i] / daily_counts[i] as f64;
            }
        }
        
        patterns
    }

    /// Predict future liquidity
    pub async fn predict_liquidity(
        &self,
        symbol: &str,
        prediction_time: SystemTime,
    ) -> ApiResult<LiquidityPrediction> {
        let models = self.models.read().await;
        let history = self.history.read().await;

        let current_score = history.get(symbol)
            .and_then(|h| h.back())
            .map(|d| d.liquidity_score)
            .unwrap_or(0.0);

        if let Some(model) = models.get(symbol) {
            // Calculate time-based adjustments
            let duration = prediction_time.duration_since(SystemTime::UNIX_EPOCH)?;
            let hour = (duration.as_secs() / 3600) % 24;
            let day = ((duration.as_secs() / 86400) + 4) % 7;
            
            let hourly_factor = model.hourly_patterns[hour as usize];
            let daily_factor = model.daily_patterns[day as usize];
            
            // Predict based on trend and patterns
            let trend = model.ma_5min - model.ma_1hour;
            let volatility_penalty = model.liquidity_volatility * 2.0;
            
            let predicted_score = (current_score + trend * 0.3)
                * hourly_factor
                * daily_factor
                - volatility_penalty;
            
            let predicted_score = predicted_score.max(0.0).min(1.0);
            
            // Calculate confidence based on data quality
            let confidence = if history.get(symbol).map(|h| h.len()).unwrap_or(0) > 500 {
                0.9 - model.liquidity_volatility
            } else {
                0.5
            };

            // Determine state and recommendation
            let (state, recommendation) = self.evaluate_prediction(predicted_score);
            
            // Find risk factors
            let mut risk_factors = Vec::new();
            if model.liquidity_volatility > 0.2 {
                risk_factors.push("High liquidity volatility".to_string());
            }
            if predicted_score < current_score * 0.8 {
                risk_factors.push("Declining liquidity trend".to_string());
            }
            if hourly_factor < 0.7 {
                risk_factors.push("Low liquidity hour".to_string());
            }
            if model.ma_5min < model.ma_15min {
                risk_factors.push("Short-term liquidity drop".to_string());
            }

            // Find next optimal time
            let next_optimal = self.find_next_optimal_time(symbol, model).await;

            Ok(LiquidityPrediction {
                symbol: symbol.to_string(),
                current_score,
                predicted_score,
                confidence,
                liquidity_state: state,
                risk_factors,
                recommended_action: recommendation,
                next_optimal_time: next_optimal,
            })
        } else {
            // No model available
            Ok(LiquidityPrediction {
                symbol: symbol.to_string(),
                current_score,
                predicted_score: current_score,
                confidence: 0.0,
                liquidity_state: LiquidityState::Insufficient,
                risk_factors: vec!["No prediction model available".to_string()],
                recommended_action: TradeRecommendation::Abort,
                next_optimal_time: None,
            })
        }
    }

    /// Evaluate prediction and determine action
    fn evaluate_prediction(&self, score: f64) -> (LiquidityState, TradeRecommendation) {
        if score >= self.config.optimal_threshold {
            (LiquidityState::Optimal, TradeRecommendation::Execute)
        } else if score >= self.config.warning_threshold {
            (LiquidityState::Good, TradeRecommendation::Execute)
        } else if score >= self.config.critical_threshold {
            (LiquidityState::Warning, TradeRecommendation::ReduceSize)
        } else if score >= 0.3 {
            (LiquidityState::Critical, TradeRecommendation::WaitForLiquidity)
        } else {
            (LiquidityState::Insufficient, TradeRecommendation::Abort)
        }
    }

    /// Find next optimal trading time
    async fn find_next_optimal_time(
        &self,
        symbol: &str,
        model: &LiquidityModel,
    ) -> Option<SystemTime> {
        let now = SystemTime::now();
        let mut best_time = None;
        let mut best_score = 0.0;

        // Check next 24 hours
        for hours_ahead in 1..24 {
            let future_time = now + Duration::from_secs(hours_ahead * 3600);
            let duration = future_time.duration_since(SystemTime::UNIX_EPOCH).ok()?;
            let hour = (duration.as_secs() / 3600) % 24;
            let day = ((duration.as_secs() / 86400) + 4) % 7;
            
            let score = model.hourly_patterns[hour as usize] * model.daily_patterns[day as usize];
            
            if score > best_score && score >= self.config.optimal_threshold {
                best_score = score;
                best_time = Some(future_time);
            }
        }

        best_time
    }

    /// Check if trade should proceed
    pub async fn should_execute_trade(
        &self,
        symbol: &str,
        size_usd: f64,
    ) -> ApiResult<(bool, LiquidityPrediction)> {
        let prediction_time = SystemTime::now() + Duration::from_secs(self.config.prediction_horizon_minutes * 60);
        let prediction = self.predict_liquidity(symbol, prediction_time).await?;

        // Additional size-based checks
        let size_factor = (size_usd / 100_000.0).min(2.0); // Penalty for large trades
        let adjusted_score = prediction.predicted_score / (1.0 + size_factor * 0.1);

        let should_execute = adjusted_score >= self.config.min_liquidity_score
            && prediction.recommended_action == TradeRecommendation::Execute;

        Ok((should_execute, prediction))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_liquidity_predictor() {
        let predictor = LiquidityPredictor::new(PredictorConfig::default());
        
        // Add some test data
        for i in 0..100 {
            let metrics = LiquidityMetrics {
                symbol: "BTC/USDT".to_string(),
                volume_24h_usd: 1_000_000_000.0 + (i as f64 * 1_000_000.0),
                bid_depth_usd: 5_000_000.0,
                ask_depth_usd: 5_000_000.0,
                spread_percent: 0.01,
                market_maker_count: 20,
                last_updated: SystemTime::now(),
            };
            predictor.record_metrics("BTC/USDT", metrics).await;
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        // Test prediction
        let (should_trade, prediction) = predictor.should_execute_trade("BTC/USDT", 50_000.0).await.unwrap();
        
        assert!(prediction.confidence > 0.0);
        println!("Should trade: {}, Prediction: {:?}", should_trade, prediction);
    }
}
