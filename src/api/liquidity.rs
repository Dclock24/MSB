// Liquidity Verification Module
// Ensures trading pairs have sufficient liquidity for entry and exit

use super::{ApiResult, MarketData};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Minimum liquidity requirements
#[derive(Debug, Clone)]
pub struct LiquidityRequirements {
    /// Minimum 24h volume in USD
    pub min_daily_volume: f64,
    /// Minimum order book depth in USD
    pub min_order_book_depth: f64,
    /// Maximum spread percentage
    pub max_spread_percent: f64,
    /// Minimum number of market makers
    pub min_market_makers: u32,
}

impl Default for LiquidityRequirements {
    fn default() -> Self {
        Self {
            min_daily_volume: 1_000_000.0,      // $1M daily volume
            min_order_book_depth: 100_000.0,    // $100k order book depth
            max_spread_percent: 0.5,            // 0.5% max spread
            min_market_makers: 3,               // At least 3 market makers
        }
    }
}

/// Liquidity metrics for a trading pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityMetrics {
    pub symbol: String,
    pub volume_24h_usd: f64,
    pub bid_depth_usd: f64,
    pub ask_depth_usd: f64,
    pub spread_percent: f64,
    pub market_maker_count: u32,
    pub last_updated: std::time::SystemTime,
}

/// Approved trading pairs with verified liquidity
#[derive(Debug, Clone)]
pub struct ApprovedPairs {
    /// Primary trading pairs with deep liquidity
    pub primary: Vec<TradingPair>,
    /// Secondary pairs with adequate liquidity
    pub secondary: Vec<TradingPair>,
    /// Pairs to avoid due to liquidity concerns
    pub blacklist: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingPair {
    pub symbol: String,
    pub base_asset: String,
    pub quote_asset: String,
    pub exchange_symbol: String,  // Exchange-specific symbol
    pub min_trade_size: f64,      // Minimum trade size in base asset
    pub max_trade_size: f64,      // Maximum trade size in base asset
    pub tick_size: f64,           // Price tick size
    pub lot_size: f64,            // Quantity step size
    pub maker_fee: f64,           // Maker fee percentage
    pub taker_fee: f64,           // Taker fee percentage
}

impl Default for ApprovedPairs {
    fn default() -> Self {
        Self {
            // High liquidity pairs only
            primary: vec![
                TradingPair {
                    symbol: "BTC/USDT".to_string(),
                    base_asset: "BTC".to_string(),
                    quote_asset: "USDT".to_string(),
                    exchange_symbol: "XBTUSDT".to_string(),
                    min_trade_size: 0.0001,
                    max_trade_size: 100.0,
                    tick_size: 0.01,
                    lot_size: 0.00001,
                    maker_fee: 0.001,  // 0.1%
                    taker_fee: 0.002,  // 0.2%
                },
                TradingPair {
                    symbol: "ETH/USDT".to_string(),
                    base_asset: "ETH".to_string(),
                    quote_asset: "USDT".to_string(),
                    exchange_symbol: "ETHUSDT".to_string(),
                    min_trade_size: 0.001,
                    max_trade_size: 1000.0,
                    tick_size: 0.01,
                    lot_size: 0.0001,
                    maker_fee: 0.001,
                    taker_fee: 0.002,
                },
                TradingPair {
                    symbol: "SOL/USDT".to_string(),
                    base_asset: "SOL".to_string(),
                    quote_asset: "USDT".to_string(),
                    exchange_symbol: "SOLUSDT".to_string(),
                    min_trade_size: 0.01,
                    max_trade_size: 10000.0,
                    tick_size: 0.001,
                    lot_size: 0.001,
                    maker_fee: 0.001,
                    taker_fee: 0.002,
                },
            ],
            // Secondary pairs with good but not exceptional liquidity
            secondary: vec![
                TradingPair {
                    symbol: "MATIC/USDT".to_string(),
                    base_asset: "MATIC".to_string(),
                    quote_asset: "USDT".to_string(),
                    exchange_symbol: "MATICUSDT".to_string(),
                    min_trade_size: 1.0,
                    max_trade_size: 100000.0,
                    tick_size: 0.0001,
                    lot_size: 0.1,
                    maker_fee: 0.001,
                    taker_fee: 0.002,
                },
                TradingPair {
                    symbol: "AVAX/USDT".to_string(),
                    base_asset: "AVAX".to_string(),
                    quote_asset: "USDT".to_string(),
                    exchange_symbol: "AVAXUSDT".to_string(),
                    min_trade_size: 0.1,
                    max_trade_size: 10000.0,
                    tick_size: 0.001,
                    lot_size: 0.01,
                    maker_fee: 0.001,
                    taker_fee: 0.002,
                },
            ],
            // Avoid these due to liquidity issues
            blacklist: vec![
                "LUNA/USDT".to_string(),  // Defunct
                "UST/USDT".to_string(),   // Defunct
                "FTT/USDT".to_string(),   // Defunct
                // Add any low liquidity or problematic pairs
            ],
        }
    }
}

/// Liquidity monitor
pub struct LiquidityMonitor {
    requirements: LiquidityRequirements,
    approved_pairs: ApprovedPairs,
    metrics_cache: Arc<RwLock<HashMap<String, LiquidityMetrics>>>,
}

impl LiquidityMonitor {
    pub fn new() -> Self {
        Self {
            requirements: LiquidityRequirements::default(),
            approved_pairs: ApprovedPairs::default(),
            metrics_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if a trading pair has sufficient liquidity
    pub async fn verify_liquidity(&self, symbol: &str) -> ApiResult<bool> {
        // Check blacklist first
        if self.approved_pairs.blacklist.contains(&symbol.to_string()) {
            log::warn!("Symbol {} is blacklisted due to liquidity concerns", symbol);
            return Ok(false);
        }

        // Check if it's an approved pair
        let is_primary = self.approved_pairs.primary.iter().any(|p| p.symbol == symbol);
        let is_secondary = self.approved_pairs.secondary.iter().any(|p| p.symbol == symbol);

        if !is_primary && !is_secondary {
            log::warn!("Symbol {} is not in approved pairs list", symbol);
            return Ok(false);
        }

        // Get cached metrics or fetch new ones
        let metrics = self.fetch_liquidity_metrics(symbol).await?;

        // Verify against requirements
        let volume_ok = metrics.volume_24h_usd >= self.requirements.min_daily_volume;
        let depth_ok = metrics.bid_depth_usd >= self.requirements.min_order_book_depth
            && metrics.ask_depth_usd >= self.requirements.min_order_book_depth;
        let spread_ok = metrics.spread_percent <= self.requirements.max_spread_percent;
        let makers_ok = metrics.market_maker_count >= self.requirements.min_market_makers;

        let is_liquid = volume_ok && depth_ok && spread_ok && makers_ok;

        if !is_liquid {
            log::warn!(
                "Liquidity check failed for {}: volume_ok={}, depth_ok={}, spread_ok={}, makers_ok={}",
                symbol, volume_ok, depth_ok, spread_ok, makers_ok
            );
        }

        Ok(is_liquid)
    }

    /// Get liquidity metrics for a symbol (internal)
    async fn fetch_liquidity_metrics(&self, symbol: &str) -> ApiResult<LiquidityMetrics> {
        // Check cache first
        {
            let cache = self.metrics_cache.read().await;
            if let Some(metrics) = cache.get(symbol) {
                let age = std::time::SystemTime::now()
                    .duration_since(metrics.last_updated)
                    .unwrap_or_default();
                
                // Return cached data if less than 5 minutes old
                if age.as_secs() < 300 {
                    return Ok(metrics.clone());
                }
            }
        }

        // In production, fetch from exchange API
        // For now, return mock data for approved pairs
        let metrics = if symbol == "BTC/USDT" {
            LiquidityMetrics {
                symbol: symbol.to_string(),
                volume_24h_usd: 2_000_000_000.0,  // $2B
                bid_depth_usd: 5_000_000.0,       // $5M
                ask_depth_usd: 5_000_000.0,       // $5M
                spread_percent: 0.01,             // 0.01%
                market_maker_count: 20,
                last_updated: std::time::SystemTime::now(),
            }
        } else if symbol == "ETH/USDT" {
            LiquidityMetrics {
                symbol: symbol.to_string(),
                volume_24h_usd: 1_000_000_000.0,  // $1B
                bid_depth_usd: 3_000_000.0,       // $3M
                ask_depth_usd: 3_000_000.0,       // $3M
                spread_percent: 0.02,             // 0.02%
                market_maker_count: 15,
                last_updated: std::time::SystemTime::now(),
            }
        } else if symbol == "SOL/USDT" {
            LiquidityMetrics {
                symbol: symbol.to_string(),
                volume_24h_usd: 500_000_000.0,    // $500M
                bid_depth_usd: 1_000_000.0,       // $1M
                ask_depth_usd: 1_000_000.0,       // $1M
                spread_percent: 0.05,             // 0.05%
                market_maker_count: 10,
                last_updated: std::time::SystemTime::now(),
            }
        } else {
            // Default metrics for other pairs
            LiquidityMetrics {
                symbol: symbol.to_string(),
                volume_24h_usd: 10_000_000.0,     // $10M
                bid_depth_usd: 200_000.0,         // $200k
                ask_depth_usd: 200_000.0,         // $200k
                spread_percent: 0.1,              // 0.1%
                market_maker_count: 5,
                last_updated: std::time::SystemTime::now(),
            }
        };

        // Update cache
        {
            let mut cache = self.metrics_cache.write().await;
            cache.insert(symbol.to_string(), metrics.clone());
        }

        Ok(metrics)
    }

    /// Calculate position size based on liquidity
    pub async fn calculate_safe_position_size(
        &self,
        symbol: &str,
        desired_size_usd: f64,
    ) -> ApiResult<f64> {
        let metrics = self.fetch_liquidity_metrics(symbol).await?;
        
        // Don't take more than 1% of order book depth
        let max_from_depth = (metrics.bid_depth_usd.min(metrics.ask_depth_usd)) * 0.01;
        
        // Don't take more than 0.1% of daily volume
        let max_from_volume = metrics.volume_24h_usd * 0.001;
        
        // Use the smaller of the two
        let safe_size = desired_size_usd.min(max_from_depth).min(max_from_volume);
        
        // Find the trading pair configuration
        let pair = self.approved_pairs.primary.iter()
            .chain(self.approved_pairs.secondary.iter())
            .find(|p| p.symbol == symbol);
        
        if let Some(pair) = pair {
            // Apply exchange limits
            let price = 50000.0; // TODO: Get actual price
            let size_in_base = safe_size / price;
            
            // Enforce min/max trade sizes
            let final_size_base = size_in_base
                .max(pair.min_trade_size)
                .min(pair.max_trade_size);
            
            // Round to lot size
            let rounded_size = (final_size_base / pair.lot_size).round() * pair.lot_size;
            
            Ok(rounded_size * price)
        } else {
            Ok(safe_size)
        }
    }

    /// Get list of approved trading symbols
    pub fn get_approved_symbols(&self) -> Vec<String> {
        self.approved_pairs.primary.iter()
            .map(|p| p.symbol.clone())
            .collect()
    }

    /// Get liquidity metrics for a symbol (public wrapper)
    pub async fn get_liquidity_metrics(&self, symbol: &str) -> ApiResult<LiquidityMetrics> {
        // Check cache first
        {
            let cache = self.metrics_cache.read().await;
            if let Some(metrics) = cache.get(symbol) {
                let age = std::time::SystemTime::now()
                    .duration_since(metrics.last_updated)
                    .unwrap_or_default();
                
                // Return cached data if less than 5 minutes old
                if age.as_secs() < 300 {
                    return Ok(metrics.clone());
                }
            }
        }

        // In production, fetch from exchange API
        // For now, return mock data for approved pairs
        let metrics = if symbol == "BTC/USDT" {
            LiquidityMetrics {
                symbol: symbol.to_string(),
                volume_24h_usd: 2_000_000_000.0,  // $2B
                bid_depth_usd: 5_000_000.0,       // $5M
                ask_depth_usd: 5_000_000.0,       // $5M
                spread_percent: 0.01,             // 0.01%
                market_maker_count: 20,
                last_updated: std::time::SystemTime::now(),
            }
        } else if symbol == "ETH/USDT" {
            LiquidityMetrics {
                symbol: symbol.to_string(),
                volume_24h_usd: 1_000_000_000.0,  // $1B
                bid_depth_usd: 3_000_000.0,       // $3M
                ask_depth_usd: 3_000_000.0,       // $3M
                spread_percent: 0.02,             // 0.02%
                market_maker_count: 15,
                last_updated: std::time::SystemTime::now(),
            }
        } else if symbol == "SOL/USDT" {
            LiquidityMetrics {
                symbol: symbol.to_string(),
                volume_24h_usd: 500_000_000.0,    // $500M
                bid_depth_usd: 1_000_000.0,       // $1M
                ask_depth_usd: 1_000_000.0,       // $1M
                spread_percent: 0.05,             // 0.05%
                market_maker_count: 10,
                last_updated: std::time::SystemTime::now(),
            }
        } else {
            // Default metrics for other pairs
            LiquidityMetrics {
                symbol: symbol.to_string(),
                volume_24h_usd: 10_000_000.0,     // $10M
                bid_depth_usd: 200_000.0,         // $200k
                ask_depth_usd: 200_000.0,         // $200k
                spread_percent: 0.1,              // 0.1%
                market_maker_count: 5,
                last_updated: std::time::SystemTime::now(),
            }
        };

        // Update cache
        {
            let mut cache = self.metrics_cache.write().await;
            cache.insert(symbol.to_string(), metrics.clone());
        }

        Ok(metrics)
    }
    
    /// Get trading pair configuration
    pub fn get_pair_config(&self, symbol: &str) -> Option<&TradingPair> {
        self.approved_pairs.primary.iter()
            .chain(self.approved_pairs.secondary.iter())
            .find(|p| p.symbol == symbol)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_liquidity_verification() {
        let monitor = LiquidityMonitor::new();
        
        // Test approved pair
        let result = monitor.verify_liquidity("BTC/USDT").await.unwrap();
        assert!(result);
        
        // Test blacklisted pair
        let result = monitor.verify_liquidity("LUNA/USDT").await.unwrap();
        assert!(!result);
        
        // Test unknown pair
        let result = monitor.verify_liquidity("UNKNOWN/USDT").await.unwrap();
        assert!(!result);
    }

    #[tokio::test]
    async fn test_safe_position_size() {
        let monitor = LiquidityMonitor::new();
        
        // Test position sizing
        let safe_size = monitor.calculate_safe_position_size("BTC/USDT", 100_000.0).await.unwrap();
        assert!(safe_size > 0.0);
        assert!(safe_size <= 100_000.0);
    }
}
