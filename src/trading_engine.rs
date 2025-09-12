// Integrated Trading Engine
// Combines all modules for live trading with safety, monitoring, and liquidity checks

use crate::api::{
    Order, OrderSide, OrderType, OrderResponse, TradingExchange,
    MarketDataProvider, ApiConfig,
    safety::{SafetyMonitor, SafetyConfig},
    liquidity::{LiquidityMonitor, TradingPair},
    liquidity_predictor::{LiquidityPredictor, PredictorConfig, TradeRecommendation},
};
use crate::monitoring::{MonitoringSystem, MetricType};
use crate::{MacroStrike, StrikeStatus, StrikeType, MIN_WIN_PROBABILITY};
use crate::opportunity_scanner::OpportunityScanner;
use crate::strike_optimizer::{StrikeOptimizer, OptimizerConfig, StrikeAnalysis, EdgeCalculation, RiskMetrics};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::SystemTime;
use log::{info, warn, error};

/// Integrated trading engine with all safety features
pub struct TradingEngine {
    /// Exchange interface
    exchange: Arc<dyn TradingExchange>,
    /// Market data provider
    market_data: Arc<dyn MarketDataProvider>,
    /// Safety monitor
    safety: Arc<SafetyMonitor>,
    /// Liquidity monitor
    liquidity: Arc<LiquidityMonitor>,
    /// Liquidity predictor
    predictor: Arc<LiquidityPredictor>,
    /// Monitoring system
    monitoring: Arc<MonitoringSystem>,
    /// Active positions
    positions: Arc<RwLock<HashMap<String, Position>>>,
    /// Engine configuration
    config: EngineConfig,
}

#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub max_positions: usize,
    pub position_size_pct: f64,
    pub stop_loss_pct: f64,
    pub take_profit_pct: f64,
    pub trailing_stop: bool,
    pub use_liquidity_prediction: bool,
    pub min_confidence: f64,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            max_positions: 5,
            position_size_pct: 0.02, // 2% per position
            stop_loss_pct: 0.02,     // 2% stop loss
            take_profit_pct: 0.06,   // 6% take profit
            trailing_stop: true,
            use_liquidity_prediction: true,
            min_confidence: 0.90,    // 90% WIN RATE REQUIREMENT
        }
    }
}

#[derive(Debug, Clone)]
pub struct Position {
    pub symbol: String,
    pub side: OrderSide,
    pub entry_price: f64,
    pub quantity: f64,
    pub stop_loss: f64,
    pub take_profit: f64,
    pub entry_time: SystemTime,
    pub order_id: String,
    pub pnl: f64,
}

impl TradingEngine {
    pub fn new(
        exchange: Arc<dyn TradingExchange>,
        market_data: Arc<dyn MarketDataProvider>,
        config: EngineConfig,
    ) -> Self {
        let safety_config = SafetyConfig::default();
        let predictor_config = PredictorConfig::default();

        Self {
            exchange,
            market_data,
            safety: Arc::new(SafetyMonitor::new(safety_config)),
            liquidity: Arc::new(LiquidityMonitor::new()),
            predictor: Arc::new(LiquidityPredictor::new(predictor_config)),
            monitoring: Arc::new(MonitoringSystem::new()),
            positions: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Execute a macro strike with all safety checks
    pub async fn execute_strike(&self, strike: &MacroStrike) -> Result<(), String> {
        info!("Executing strike #{} on {}", strike.id, self.get_symbol_name(strike.symbol));

        // 1. CRITICAL: Enforce 90% win probability requirement
        if strike.confidence < MIN_WIN_PROBABILITY {
            warn!("❌ Strike #{} REJECTED - Win probability {:.1}% < 90% minimum", 
                  strike.id, strike.confidence * 100.0);
            self.monitoring.record_metric(MetricType::TradeCount, 1.0).await;
            return Err(format!("Win probability {:.1}% below 90% minimum requirement", 
                strike.confidence * 100.0));
        }
        
        // 2. Secondary confidence check (config based)
        if strike.confidence < self.config.min_confidence {
            self.monitoring.record_metric(MetricType::TradeCount, 1.0).await;
            return Err(format!("Confidence {:.2} below configured minimum {:.2}", 
                strike.confidence, self.config.min_confidence));
        }

        let symbol = self.get_symbol_name(strike.symbol);

        // 2. Verify liquidity
        if !self.liquidity.verify_liquidity(&symbol).await.map_err(|e| e.to_string())? {
            warn!("Liquidity verification failed for {}", symbol);
            return Err("Insufficient liquidity".to_string());
        }

        // 3. Predict future liquidity
        if self.config.use_liquidity_prediction {
            let (should_trade, prediction) = self.predictor
                .should_execute_trade(&symbol, strike.position_size)
                .await
                .map_err(|e| e.to_string())?;

            if !should_trade {
                warn!("Liquidity prediction negative: {:?}", prediction.recommended_action);
                
                match prediction.recommended_action {
                    TradeRecommendation::WaitForLiquidity => {
                        if let Some(next_time) = prediction.next_optimal_time {
                            info!("Next optimal trading time: {:?}", next_time);
                        }
                        return Err("Waiting for better liquidity".to_string());
                    }
                    TradeRecommendation::ReduceSize => {
                        // Continue with reduced size
                        info!("Reducing position size due to liquidity concerns");
                    }
                    _ => return Err("Liquidity prediction recommends abort".to_string()),
                }
            }
        }

        // 4. Check safety limits
        self.safety.check_trade_allowed(&symbol, strike.position_size, false)
            .await
            .map_err(|e| e)?;

        // 5. Check position limits
        let positions = self.positions.read().await;
        if positions.len() >= self.config.max_positions {
            return Err("Maximum positions reached".to_string());
        }
        drop(positions);

        // 6. Get current market data
        let market_data = self.market_data.get_market_data(&symbol)
            .await
            .map_err(|e| e.to_string())?;

        // 7. Calculate safe position size
        let safe_size = self.liquidity
            .calculate_safe_position_size(&symbol, strike.position_size)
            .await
            .map_err(|e| e.to_string())?;

        // 8. Prepare order
        let quantity = safe_size / market_data.price;
        let side = if strike.strike_type == StrikeType::MacroMomentum {
            OrderSide::Buy
        } else {
            OrderSide::Sell
        };

        let order = Order {
            symbol: symbol.clone(),
            side: side.clone(),
            order_type: OrderType::Market,
            quantity,
            client_order_id: format!("strike_{}", strike.id),
        };

        // 9. Place order
        info!("Placing {} order for {} {} @ market", 
            match side { OrderSide::Buy => "BUY", OrderSide::Sell => "SELL" },
            quantity, symbol
        );

        let order_response = self.exchange.place_order(order)
            .await
            .map_err(|e| e.to_string())?;

        // 10. Create position record
        let order_id = order_response.order_id.clone();
        let position = Position {
            symbol: symbol.clone(),
            side,
            entry_price: market_data.price,
            quantity,
            stop_loss: strike.stop_loss,
            take_profit: strike.target_price,
            entry_time: SystemTime::now(),
            order_id: order_id.clone(),
            pnl: 0.0,
        };

        // 11. Store position
        let mut positions = self.positions.write().await;
        positions.insert(order_id.clone(), position);

        // 12. Place stop loss and take profit orders
        self.place_exit_orders(&symbol, &order_id, strike.stop_loss, strike.target_price)
            .await?;

        // 13. Update monitoring
        self.monitoring.record_metric(MetricType::TradeCount, 1.0).await;
        self.monitoring.record_metric(MetricType::Exposure, safe_size).await;

        info!("Strike #{} executed successfully", strike.id);
        Ok(())
    }

    /// Place stop loss and take profit orders
    async fn place_exit_orders(
        &self,
        symbol: &str,
        position_id: &str,
        stop_price: f64,
        target_price: f64,
    ) -> Result<(), String> {
        let positions = self.positions.read().await;
        let position = positions.get(position_id)
            .ok_or("Position not found")?;

        // Place stop loss
        let stop_order = Order {
            symbol: symbol.to_string(),
            side: match position.side {
                OrderSide::Buy => OrderSide::Sell,
                OrderSide::Sell => OrderSide::Buy,
            },
            order_type: OrderType::StopLoss { stop_price },
            quantity: position.quantity,
            client_order_id: format!("{}_sl", position_id),
        };

        self.exchange.place_order(stop_order)
            .await
            .map_err(|e| format!("Failed to place stop loss: {}", e))?;

        // Place take profit
        let tp_order = Order {
            symbol: symbol.to_string(),
            side: match position.side {
                OrderSide::Buy => OrderSide::Sell,
                OrderSide::Sell => OrderSide::Buy,
            },
            order_type: OrderType::TakeProfit { target_price },
            quantity: position.quantity,
            client_order_id: format!("{}_tp", position_id),
        };

        self.exchange.place_order(tp_order)
            .await
            .map_err(|e| format!("Failed to place take profit: {}", e))?;

        Ok(())
    }

    /// Update position tracking and PnL
    pub async fn update_positions(&self) -> Result<(), String> {
        let mut positions = self.positions.write().await;
        let mut total_pnl = 0.0;
        let mut closed_positions = Vec::new();

        for (order_id, position) in positions.iter_mut() {
            // Get current market price
            let market_data = self.market_data
                .get_market_data(&position.symbol)
                .await
                .map_err(|e| e.to_string())?;

            // Calculate unrealized PnL
            let price_diff = match position.side {
                OrderSide::Buy => market_data.price - position.entry_price,
                OrderSide::Sell => position.entry_price - market_data.price,
            };
            
            position.pnl = price_diff * position.quantity;
            total_pnl += position.pnl;

            // Check if position should be closed
            let should_close = match position.side {
                OrderSide::Buy => {
                    market_data.price <= position.stop_loss || 
                    market_data.price >= position.take_profit
                },
                OrderSide::Sell => {
                    market_data.price >= position.stop_loss || 
                    market_data.price <= position.take_profit
                },
            };

            if should_close {
                closed_positions.push(order_id.clone());
                
                // Record trade result
                let is_win = position.pnl > 0.0;
                self.safety.record_trade_result(position.pnl, is_win).await;
                
                // Update metrics
                self.monitoring.record_metric(
                    if is_win { MetricType::ConsecutiveWins } else { MetricType::ConsecutiveLosses },
                    1.0
                ).await;
            }
        }

        // Remove closed positions
        for order_id in closed_positions {
            positions.remove(&order_id);
        }

        // Update monitoring
        self.monitoring.record_metric(MetricType::TotalPnL, total_pnl).await;
        
        Ok(())
    }

    /// Get symbol name from ID
    fn get_symbol_name(&self, symbol_id: u8) -> String {
        match symbol_id {
            0 => "BTC/USDT",
            1 => "ETH/USDT",
            2 => "SOL/USDT",
            _ => "UNKNOWN",
        }.to_string()
    }

    /// Emergency stop all positions
    pub async fn emergency_stop(&self, reason: &str) -> Result<(), String> {
        error!("EMERGENCY STOP: {}", reason);
        
        // Activate safety circuit breaker
        self.safety.emergency_stop(reason).await;
        
        // Close all positions
        let positions = self.positions.read().await;
        for (order_id, position) in positions.iter() {
            warn!("Emergency closing position {}", order_id);
            
            let close_order = Order {
                symbol: position.symbol.clone(),
                side: match position.side {
                    OrderSide::Buy => OrderSide::Sell,
                    OrderSide::Sell => OrderSide::Buy,
                },
                order_type: OrderType::Market,
                quantity: position.quantity,
                client_order_id: format!("{}_emergency", order_id),
            };
            
            if let Err(e) = self.exchange.place_order(close_order).await {
                error!("Failed to close position {}: {}", order_id, e);
            }
        }
        
        Ok(())
    }

    /// Get current engine status
    pub async fn get_status(&self) -> EngineStatus {
        let positions = self.positions.read().await;
        let safety_status = self.safety.get_status().await;
        let health_status = self.monitoring.get_health_status().await;
        
        EngineStatus {
            active_positions: positions.len(),
            total_exposure: positions.values().map(|p| p.quantity * p.entry_price).sum(),
            circuit_breaker_active: safety_status.circuit_breaker_active,
            health_level: format!("{:?}", health_status.level),
            health_score: health_status.score,
        }
    }
}

#[derive(Debug)]
pub struct EngineStatus {
    pub active_positions: usize,
    pub total_exposure: f64,
    pub circuit_breaker_active: bool,
    pub health_level: String,
    pub health_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Test implementations would go here
}
