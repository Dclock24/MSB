// API Integration Module
// Provides interfaces for CoinGecko and Kraken APIs

pub mod coingecko;
pub mod kraken;
pub mod safety;
pub mod liquidity;
pub mod liquidity_predictor;

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::time::SystemTime;

/// Result type for API operations
pub type ApiResult<T> = Result<T, Box<dyn Error>>;

/// Market data from price feeds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    pub symbol: String,
    pub price: f64,
    pub volume_24h: f64,
    pub price_change_24h: f64,
    pub timestamp: SystemTime,
}

/// Ticker data for historical analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    pub symbol: String,
    pub price: f64,
    pub volume: f64,
    pub timestamp: SystemTime,
}

/// Order types for trading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit { price: f64 },
    StopLoss { stop_price: f64 },
    TakeProfit { target_price: f64 },
}

/// Order side
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// Trading order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: f64,
    pub client_order_id: String,
}

/// Order status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    PartiallyFilled { filled_qty: f64 },
    Filled { avg_price: f64, filled_qty: f64 },
    Cancelled,
    Rejected { reason: String },
}

/// Order response from exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    pub order_id: String,
    pub client_order_id: String,
    pub status: OrderStatus,
    pub timestamp: SystemTime,
}

/// Balance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub asset: String,
    pub free: f64,
    pub locked: f64,
    pub total: f64,
}

/// Order book entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookLevel {
    pub price: f64,
    pub volume: f64,
    pub timestamp: Option<SystemTime>,
}

/// Order book
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub symbol: String,
    pub bids: Vec<OrderBookLevel>,
    pub asks: Vec<OrderBookLevel>,
    pub timestamp: SystemTime,
}

/// Trait for market data providers
#[async_trait::async_trait]
pub trait MarketDataProvider: Send + Sync {
    /// Get current market data for a symbol
    async fn get_market_data(&self, symbol: &str) -> ApiResult<MarketData>;
    
    /// Subscribe to real-time price updates
    async fn subscribe_prices(&self, symbols: Vec<String>) -> ApiResult<()>;
}

/// Trait for trading exchanges
#[async_trait::async_trait]
pub trait TradingExchange: Send + Sync {
    /// Place a new order
    async fn place_order(&self, order: Order) -> ApiResult<OrderResponse>;
    
    /// Cancel an existing order
    async fn cancel_order(&self, order_id: &str) -> ApiResult<()>;
    
    /// Get order status
    async fn get_order_status(&self, order_id: &str) -> ApiResult<OrderStatus>;
    
    /// Get account balances
    async fn get_balances(&self) -> ApiResult<Vec<Balance>>;
    
    /// Get order book
    async fn get_order_book(&self, symbol: &str, depth: usize) -> ApiResult<OrderBook>;
}

/// API configuration
#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub api_key: String,
    pub api_secret: String,
    pub testnet: bool,
    pub rate_limit_per_minute: u32,
}

impl ApiConfig {
    /// Create config from environment variables
    pub fn from_env(prefix: &str) -> ApiResult<Self> {
        Ok(Self {
            api_key: std::env::var(format!("{}_API_KEY", prefix))?,
            api_secret: std::env::var(format!("{}_API_SECRET", prefix))?,
            testnet: std::env::var(format!("{}_TESTNET", prefix))
                .unwrap_or_else(|_| "false".to_string())
                .parse()?,
            rate_limit_per_minute: std::env::var(format!("{}_RATE_LIMIT", prefix))
                .unwrap_or_else(|_| "60".to_string())
                .parse()?,
        })
    }
}
