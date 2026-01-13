// Kraken API Integration
// Provides trading execution and account management

use super::{
    ApiConfig, ApiResult, Balance, Order, OrderResponse, OrderSide, OrderStatus, OrderType,
    TradingExchange,
};
use base64::Engine;
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde_json::{json, Value};
use sha2::{Digest, Sha256, Sha512};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{sleep, Duration};

type HmacSha512 = Hmac<Sha512>;

pub struct KrakenClient {
    client: Client,
    config: ApiConfig,
    base_url: String,
}

impl KrakenClient {
    pub fn new(config: ApiConfig) -> Self {
        let base_url = if config.testnet {
            "https://api.kraken.com".to_string() // Kraken doesn't have a testnet
        } else {
            "https://api.kraken.com".to_string()
        };

        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
            config,
            base_url,
        }
    }

    /// Generate Kraken API signature
    fn generate_signature(&self, path: &str, nonce: u64, post_data: &str) -> Result<String, Box<dyn std::error::Error>> {
        let secret_decoded = base64::engine::general_purpose::STANDARD
            .decode(&self.config.api_secret)
            .map_err(|e| format!("Invalid base64 API secret: {}", e))?;

        let sha256_hash = Sha256::digest(format!("{}{}", nonce, post_data).as_bytes());
        let hmac_data = [path.as_bytes(), &sha256_hash[..]].concat();

        let mut mac = HmacSha512::new_from_slice(&secret_decoded)
            .map_err(|e| format!("Invalid API secret: {}", e))?;
        mac.update(&hmac_data);

        Ok(base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes()))
    }

    /// Convert internal symbol to Kraken format
    fn to_kraken_symbol(symbol: &str) -> String {
        match symbol {
            "BTC/USDT" => "XBTUSDT".to_string(),
            "ETH/USDT" => "ETHUSDT".to_string(),
            "SOL/USDT" => "SOLUSDT".to_string(),
            _ => symbol.replace("/", ""),
        }
    }

    /// Convert Kraken symbol to internal format
    fn from_kraken_symbol(symbol: &str) -> String {
        match symbol {
            "XBTUSDT" => "BTC/USDT".to_string(),
            "ETHUSDT" => "ETH/USDT".to_string(),
            "SOLUSDT" => "SOL/USDT".to_string(),
            _ => symbol.to_string(),
        }
    }

    /// Make authenticated request
    async fn private_request(&self, endpoint: &str, params: Value) -> ApiResult<Value> {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis() as u64;

        let mut post_params = params.as_object()
            .ok_or("Invalid parameters format")?
            .clone();
        post_params.insert("nonce".to_string(), json!(nonce));

        let post_data = serde_urlencoded::to_string(&post_params)?;
        let path = format!("/0/private/{}", endpoint);
        let signature = self.generate_signature(&path, nonce, &post_data)?;

        let response = self
            .client
            .post(format!("{}{}", self.base_url, path))
            .header("API-Key", &self.config.api_key)
            .header("API-Sign", signature)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(post_data)
            .send()
            .await?;

        let result: Value = response.json().await?;

        if let Some(errors) = result["error"].as_array() {
            if !errors.is_empty() {
                return Err(format!("Kraken API error: {:?}", errors).into());
            }
        }

        Ok(result["result"].clone())
    }

    /// Rate limiting
    async fn rate_limit(&self) {
        let delay_ms = 60_000 / self.config.rate_limit_per_minute;
        sleep(Duration::from_millis(delay_ms as u64)).await;
    }
}

#[async_trait::async_trait]
impl TradingExchange for KrakenClient {
    async fn place_order(&self, order: Order) -> ApiResult<OrderResponse> {
        let order_type = match &order.order_type {
            OrderType::Market => "market",
            OrderType::Limit { .. } => "limit",
            OrderType::StopLoss { .. } => "stop-loss",
            OrderType::TakeProfit { .. } => "take-profit",
        };

        let mut params = json!({
            "pair": Self::to_kraken_symbol(&order.symbol),
            "type": match order.side {
                OrderSide::Buy => "buy",
                OrderSide::Sell => "sell",
            },
            "ordertype": order_type,
            "volume": order.quantity.to_string(),
            "userref": order.client_order_id,
        });

        // Add price for limit orders
        if let OrderType::Limit { price } = order.order_type {
            params["price"] = json!(price.to_string());
        }

        let result = self.private_request("AddOrder", params).await?;
        self.rate_limit().await;

        let order_id = result["txid"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
            .ok_or("Missing order ID")?
            .to_string();

        Ok(OrderResponse {
            order_id,
            client_order_id: order.client_order_id,
            status: OrderStatus::Pending,
            timestamp: SystemTime::now(),
        })
    }

    async fn cancel_order(&self, order_id: &str) -> ApiResult<()> {
        let params = json!({
            "txid": order_id,
        });

        self.private_request("CancelOrder", params).await?;
        self.rate_limit().await;
        Ok(())
    }

    async fn get_order_status(&self, order_id: &str) -> ApiResult<OrderStatus> {
        let params = json!({
            "txid": order_id,
            "trades": true,
        });

        let result = self.private_request("QueryOrders", params).await?;
        self.rate_limit().await;

        let order_data = result[order_id]
            .as_object()
            .ok_or("Order not found")?;

        let status = order_data["status"]
            .as_str()
            .ok_or("Missing status")?;

        match status {
            "pending" | "open" => Ok(OrderStatus::Pending),
            "closed" => {
                let vol_exec = order_data["vol_exec"]
                    .as_str()
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or(0.0);
                let avg_price = order_data["price"]
                    .as_str()
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or(0.0);
                Ok(OrderStatus::Filled {
                    avg_price,
                    filled_qty: vol_exec,
                })
            }
            "canceled" => Ok(OrderStatus::Cancelled),
            _ => Ok(OrderStatus::Pending),
        }
    }

    async fn get_balances(&self) -> ApiResult<Vec<Balance>> {
        let result = self.private_request("Balance", json!({})).await?;
        self.rate_limit().await;

        let mut balances = Vec::new();
        
        if let Some(obj) = result.as_object() {
            for (asset, balance_str) in obj {
                if let Some(balance) = balance_str.as_str().and_then(|s| s.parse::<f64>().ok()) {
                    // Convert Kraken asset names to standard
                    let asset_name = match asset.as_str() {
                        "XXBT" => "BTC",
                        "XETH" => "ETH",
                        "ZUSD" => "USD",
                        "USDT" => "USDT",
                        _ => asset,
                    };

                    balances.push(Balance {
                        asset: asset_name.to_string(),
                        free: balance,
                        locked: 0.0, // Kraken doesn't separate locked balance
                        total: balance,
                    });
                }
            }
        }

        Ok(balances)
    }
    
    async fn get_order_book(&self, symbol: &str, depth: usize) -> ApiResult<OrderBook> {
        let params = json!({
            "pair": Self::to_kraken_symbol(symbol),
            "count": depth
        });
        
        let endpoint = format!("{}/public/Depth", self.base_url);
        let response = self.client
            .get(&endpoint)
            .json(&params)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;
        
        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            return Err("Rate limited".into());
        }
        
        let result = response.json::<serde_json::Value>().await
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        self.rate_limit().await;
        
        // Parse Kraken's order book format
        let pair_data = result[Self::to_kraken_symbol(symbol)].clone();
        
        let mut bids = Vec::new();
        let mut asks = Vec::new();
        
        // Parse bids
        if let Some(bid_array) = pair_data["bids"].as_array() {
            for bid in bid_array.iter().take(depth) {
                if let Some(arr) = bid.as_array() {
                    if arr.len() >= 2 {
                        let price = arr[0].as_str().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                        let volume = arr[1].as_str().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                        bids.push(OrderBookLevel { price, volume });
                    }
                }
            }
        }
        
        // Parse asks
        if let Some(ask_array) = pair_data["asks"].as_array() {
            for ask in ask_array.iter().take(depth) {
                if let Some(arr) = ask.as_array() {
                    if arr.len() >= 2 {
                        let price = arr[0].as_str().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                        let volume = arr[1].as_str().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                        asks.push(OrderBookLevel { price, volume });
                    }
                }
            }
        }
        
        Ok(OrderBook {
            symbol: symbol.to_string(),
            bids,
            asks,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_conversion() {
        assert_eq!(KrakenClient::to_kraken_symbol("BTC/USDT"), "XBTUSDT");
        assert_eq!(KrakenClient::from_kraken_symbol("XBTUSDT"), "BTC/USDT");
    }
}
