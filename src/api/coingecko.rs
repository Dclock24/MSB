// CoinGecko API Integration
// Provides market data and price feeds

use super::{ApiConfig, ApiResult, MarketData, MarketDataProvider};
use reqwest::Client;
use serde_json::Value;
use std::time::SystemTime;
use tokio::time::{sleep, Duration};

pub struct CoinGeckoClient {
    client: Client,
    config: ApiConfig,
    base_url: String,
}

impl CoinGeckoClient {
    pub fn new(config: ApiConfig) -> Self {
        let base_url = if config.testnet {
            "https://api.coingecko.com/api/v3".to_string()
        } else {
            "https://pro-api.coingecko.com/api/v3".to_string()
        };

        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap(),
            config,
            base_url,
        }
    }

    /// Convert CoinGecko ID to trading symbol
    fn id_to_symbol(id: &str) -> String {
        match id {
            "bitcoin" => "BTC/USDT".to_string(),
            "ethereum" => "ETH/USDT".to_string(),
            "solana" => "SOL/USDT".to_string(),
            _ => format!("{}/USDT", id.to_uppercase()),
        }
    }

    /// Rate limiting helper
    async fn rate_limit(&self) {
        // CoinGecko has different limits for free vs pro
        let delay_ms = 60_000 / self.config.rate_limit_per_minute;
        sleep(Duration::from_millis(delay_ms as u64)).await;
    }
}

#[async_trait::async_trait]
impl MarketDataProvider for CoinGeckoClient {
    async fn get_market_data(&self, symbol: &str) -> ApiResult<MarketData> {
        // Convert symbol to CoinGecko ID
        let coin_id = match symbol {
            "BTC/USDT" => "bitcoin",
            "ETH/USDT" => "ethereum",
            "SOL/USDT" => "solana",
            _ => return Err("Unsupported symbol".into()),
        };

        let url = format!(
            "{}/coins/{}?localization=false&tickers=false&community_data=false&developer_data=false",
            self.base_url, coin_id
        );

        let response = self
            .client
            .get(&url)
            .header("x-cg-pro-api-key", &self.config.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("API error: {}", response.status()).into());
        }

        let data: Value = response.json().await?;
        
        // Rate limiting
        self.rate_limit().await;

        Ok(MarketData {
            symbol: symbol.to_string(),
            price: data["market_data"]["current_price"]["usd"]
                .as_f64()
                .ok_or("Missing price data")?,
            volume_24h: data["market_data"]["total_volume"]["usd"]
                .as_f64()
                .unwrap_or(0.0),
            price_change_24h: data["market_data"]["price_change_percentage_24h"]
                .as_f64()
                .unwrap_or(0.0),
            timestamp: SystemTime::now(),
        })
    }

    async fn subscribe_prices(&self, symbols: Vec<String>) -> ApiResult<()> {
        // CoinGecko doesn't have WebSocket support
        // For real-time data, you would poll or use their webhook service
        log::info!("CoinGecko price subscription requested for: {:?}", symbols);
        log::warn!("Note: CoinGecko doesn't support WebSocket. Consider polling or webhooks.");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_conversion() {
        assert_eq!(CoinGeckoClient::id_to_symbol("bitcoin"), "BTC/USDT");
        assert_eq!(CoinGeckoClient::id_to_symbol("ethereum"), "ETH/USDT");
    }
}
