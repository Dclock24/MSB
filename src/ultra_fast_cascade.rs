// Ultra-Fast Sentiment Cascade Detection
// Detects information cascades 30 seconds to 2 minutes before price impact

use std::collections::{HashMap, VecDeque, BTreeMap};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};
use log::{info, debug, warn};

/// Ultra-fast cascade detection with sub-second latency
pub struct UltraFastCascadeDetector {
    // Real-time data streams
    whale_alerts: Arc<RwLock<WhaleAlertStream>>,
    order_flow_analyzer: Arc<RwLock<OrderFlowAnalyzer>>,
    mempool_scanner: Arc<RwLock<MempoolScanner>>,
    social_velocity_tracker: Arc<RwLock<SocialVelocityTracker>>,
    dex_flow_monitor: Arc<RwLock<DexFlowMonitor>>,
    
    // Pattern detection
    cascade_patterns: Arc<RwLock<HashMap<String, CascadePattern>>>,
    early_signals: Arc<RwLock<VecDeque<EarlySignal>>>,
    
    // Timing prediction
    impact_predictor: Arc<RwLock<ImpactTimePredictor>>,
}

#[derive(Debug, Clone)]
pub struct CascadePattern {
    pub cascade_id: String,
    pub symbol: String,
    pub cascade_type: CascadeType,
    pub strength: f64,
    pub velocity: f64,
    pub acceleration: f64,
    pub time_to_impact_ms: u64,
    pub confidence: f64,
    pub sources: Vec<SignalSource>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum CascadeType {
    WhaleAccumulation,      // Large wallets accumulating
    SmartMoneyFlow,         // Known profitable wallets moving
    SocialViralSpike,       // Viral spread pattern detected
    MempoolAnomalies,       // Unusual mempool activity
    DexAggregation,         // Multiple DEX buys in sequence
    InstitutionalEntry,     // Pattern matching institutional buying
    InsiderActivity,        // Suspicious pre-announcement activity
}

#[derive(Debug, Clone)]
pub struct SignalSource {
    pub source_type: SourceType,
    pub signal_strength: f64,
    pub timestamp: DateTime<Utc>,
    pub reliability_score: f64,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum SourceType {
    WhaleWallet(String),
    SmartContract(String),
    SocialInfluencer(String),
    MempoolTransaction(String),
    DexTrade(String),
    OrderBookImbalance,
}

#[derive(Debug, Clone)]
pub struct EarlySignal {
    pub signal_id: String,
    pub detected_at: DateTime<Utc>,
    pub signal_type: SignalType,
    pub magnitude: f64,
    pub propagation_speed: f64,
}

#[derive(Debug, Clone)]
pub enum SignalType {
    WhaleMovement(WhaleAction),
    OrderFlowShift(FlowDirection),
    MempoolSpike(TxType),
    SocialBurst(Platform),
    DexSweep(Vec<String>),
}

#[derive(Debug, Clone)]
pub struct WhaleAction {
    pub wallet: String,
    pub action: String, // "accumulate", "distribute", "transfer"
    pub amount_usd: f64,
    pub historical_accuracy: f64,
}

#[derive(Debug, Clone)]
pub enum FlowDirection {
    Bullish,
    Bearish,
    Neutral,
}

#[derive(Debug, Clone)]
pub enum TxType {
    LargeTransfer,
    ContractInteraction,
    DexTrade,
    NFTMint,
}

#[derive(Debug, Clone)]
pub enum Platform {
    Twitter,
    Telegram,
    Discord,
    Reddit,
    TradingView,
}

// Component implementations

pub struct WhaleAlertStream {
    tracked_wallets: HashMap<String, WalletProfile>,
    real_time_transfers: VecDeque<WhaleTransfer>,
    pattern_matcher: WhalePatternMatcher,
}

pub struct WalletProfile {
    pub address: String,
    pub historical_pnl: f64,
    pub accuracy_score: f64,
    pub typical_hold_time: Duration,
    pub influence_score: f64,
}

pub struct WhaleTransfer {
    pub from: String,
    pub to: String,
    pub token: String,
    pub amount_usd: f64,
    pub timestamp: DateTime<Utc>,
    pub tx_hash: String,
}

pub struct WhalePatternMatcher {
    pub accumulation_patterns: Vec<AccumulationPattern>,
    pub distribution_patterns: Vec<DistributionPattern>,
}

pub struct AccumulationPattern {
    pub min_wallets: usize,
    pub time_window: Duration,
    pub min_total_usd: f64,
    pub price_impact_correlation: f64,
}

pub struct DistributionPattern {
    pub wallet_dispersion: f64,
    pub time_clustering: f64,
    pub exchange_deposits: bool,
}

pub struct OrderFlowAnalyzer {
    pub flow_history: VecDeque<OrderFlowSnapshot>,
    pub imbalance_detector: ImbalanceDetector,
    pub sweep_detector: SweepDetector,
}

pub struct OrderFlowSnapshot {
    pub timestamp: DateTime<Utc>,
    pub buy_volume: f64,
    pub sell_volume: f64,
    pub large_orders: Vec<LargeOrder>,
    pub flow_toxicity: f64,
}

pub struct LargeOrder {
    pub side: String,
    pub size_usd: f64,
    pub price: f64,
    pub exchange: String,
    pub aggressive: bool,
}

pub struct ImbalanceDetector {
    pub threshold: f64,
    pub time_window: Duration,
    pub min_volume: f64,
}

pub struct SweepDetector {
    pub sweep_threshold: f64,
    pub time_limit: Duration,
    pub exchanges_required: usize,
}

pub struct MempoolScanner {
    pub pending_txs: BTreeMap<String, PendingTransaction>,
    pub anomaly_detector: AnomalyDetector,
    pub frontrun_detector: FrontrunDetector,
}

pub struct PendingTransaction {
    pub hash: String,
    pub from: String,
    pub to: String,
    pub value: f64,
    pub gas_price: f64,
    pub input_data: Vec<u8>,
    pub detected_at: DateTime<Utc>,
}

pub struct AnomalyDetector {
    pub unusual_gas_threshold: f64,
    pub bundle_detector: BundleDetector,
    pub contract_patterns: HashMap<String, ContractPattern>,
}

pub struct BundleDetector {
    pub min_txs: usize,
    pub time_correlation: f64,
    pub address_correlation: f64,
}

pub struct ContractPattern {
    pub contract_address: String,
    pub typical_gas: f64,
    pub known_methods: Vec<String>,
}

pub struct FrontrunDetector {
    pub similar_tx_threshold: f64,
    pub time_window: Duration,
    pub profit_threshold: f64,
}

pub struct SocialVelocityTracker {
    pub mention_streams: HashMap<Platform, MentionStream>,
    pub influencer_tracker: InfluencerTracker,
    pub viral_detector: ViralDetector,
}

pub struct MentionStream {
    pub platform: Platform,
    pub mentions_per_second: VecDeque<(DateTime<Utc>, f64)>,
    pub sentiment_scores: VecDeque<(DateTime<Utc>, f64)>,
    pub reach_scores: VecDeque<(DateTime<Utc>, f64)>,
}

pub struct InfluencerTracker {
    pub tracked_accounts: HashMap<String, InfluencerProfile>,
    pub cascade_history: HashMap<String, Vec<CascadeEvent>>,
}

pub struct InfluencerProfile {
    pub handle: String,
    pub platform: Platform,
    pub followers: u64,
    pub avg_engagement: f64,
    pub price_impact_history: Vec<(DateTime<Utc>, f64)>,
}

pub struct CascadeEvent {
    pub influencer: String,
    pub timestamp: DateTime<Utc>,
    pub engagement_rate: f64,
    pub price_impact: f64,
    pub time_to_impact: Duration,
}

pub struct ViralDetector {
    pub viral_threshold: f64,
    pub acceleration_threshold: f64,
    pub network_effect_multiplier: f64,
}

pub struct DexFlowMonitor {
    pub dex_aggregators: HashMap<String, DexAggregator>,
    pub routing_analyzer: RoutingAnalyzer,
    pub sandwich_detector: SandwichDetector,
}

pub struct DexAggregator {
    pub name: String,
    pub monitored_pairs: Vec<String>,
    pub trade_stream: VecDeque<DexTrade>,
}

pub struct DexTrade {
    pub pair: String,
    pub side: String,
    pub amount_usd: f64,
    pub price_impact: f64,
    pub router: String,
    pub timestamp: DateTime<Utc>,
}

pub struct RoutingAnalyzer {
    pub common_paths: HashMap<String, Vec<String>>,
    pub arbitrage_detector: ArbitrageDetector,
}

pub struct ArbitrageDetector {
    pub min_profit: f64,
    pub max_hops: usize,
    pub gas_threshold: f64,
}

pub struct SandwichDetector {
    pub victim_size_threshold: f64,
    pub profit_threshold: f64,
    pub time_window: Duration,
}

pub struct ImpactTimePredictor {
    pub historical_patterns: HashMap<CascadeType, Vec<HistoricalImpact>>,
    pub ml_predictor: MLPredictor,
    pub confidence_calculator: ConfidenceCalculator,
}

pub struct HistoricalImpact {
    pub cascade_strength: f64,
    pub time_to_impact: Duration,
    pub price_impact: f64,
    pub accuracy: f64,
}

pub struct MLPredictor {
    pub features: Vec<String>,
    pub model_weights: Vec<f64>,
    pub bias: f64,
}

pub struct ConfidenceCalculator {
    pub base_confidence: f64,
    pub source_weights: HashMap<SourceType, f64>,
    pub recency_decay: f64,
}

impl UltraFastCascadeDetector {
    pub fn new() -> Self {
        Self {
            whale_alerts: Arc::new(RwLock::new(WhaleAlertStream {
                tracked_wallets: Self::load_whale_wallets(),
                real_time_transfers: VecDeque::with_capacity(1000),
                pattern_matcher: WhalePatternMatcher {
                    accumulation_patterns: Self::load_accumulation_patterns(),
                    distribution_patterns: Self::load_distribution_patterns(),
                },
            })),
            order_flow_analyzer: Arc::new(RwLock::new(OrderFlowAnalyzer {
                flow_history: VecDeque::with_capacity(1000),
                imbalance_detector: ImbalanceDetector {
                    threshold: 0.7,
                    time_window: Duration::seconds(30),
                    min_volume: 100000.0,
                },
                sweep_detector: SweepDetector {
                    sweep_threshold: 0.8,
                    time_limit: Duration::seconds(5),
                    exchanges_required: 3,
                },
            })),
            mempool_scanner: Arc::new(RwLock::new(MempoolScanner {
                pending_txs: BTreeMap::new(),
                anomaly_detector: AnomalyDetector {
                    unusual_gas_threshold: 2.0,
                    bundle_detector: BundleDetector {
                        min_txs: 3,
                        time_correlation: 0.9,
                        address_correlation: 0.8,
                    },
                    contract_patterns: HashMap::new(),
                },
                frontrun_detector: FrontrunDetector {
                    similar_tx_threshold: 0.95,
                    time_window: Duration::seconds(2),
                    profit_threshold: 100.0,
                },
            })),
            social_velocity_tracker: Arc::new(RwLock::new(SocialVelocityTracker {
                mention_streams: HashMap::new(),
                influencer_tracker: InfluencerTracker {
                    tracked_accounts: Self::load_influencers(),
                    cascade_history: HashMap::new(),
                },
                viral_detector: ViralDetector {
                    viral_threshold: 10.0, // 10x normal activity
                    acceleration_threshold: 2.0, // 2x acceleration
                    network_effect_multiplier: 1.5,
                },
            })),
            dex_flow_monitor: Arc::new(RwLock::new(DexFlowMonitor {
                dex_aggregators: Self::init_dex_aggregators(),
                routing_analyzer: RoutingAnalyzer {
                    common_paths: HashMap::new(),
                    arbitrage_detector: ArbitrageDetector {
                        min_profit: 50.0,
                        max_hops: 3,
                        gas_threshold: 0.01,
                    },
                },
                sandwich_detector: SandwichDetector {
                    victim_size_threshold: 10000.0,
                    profit_threshold: 100.0,
                    time_window: Duration::seconds(12), // 1 block
                },
            })),
            cascade_patterns: Arc::new(RwLock::new(HashMap::new())),
            early_signals: Arc::new(RwLock::new(VecDeque::with_capacity(100))),
            impact_predictor: Arc::new(RwLock::new(ImpactTimePredictor {
                historical_patterns: Self::load_historical_patterns(),
                ml_predictor: MLPredictor {
                    features: vec![
                        "cascade_strength".to_string(),
                        "source_count".to_string(),
                        "velocity".to_string(),
                        "acceleration".to_string(),
                    ],
                    model_weights: vec![0.3, 0.2, 0.3, 0.2],
                    bias: 30.0, // 30 second base time
                },
                confidence_calculator: ConfidenceCalculator {
                    base_confidence: 0.85,
                    source_weights: Self::init_source_weights(),
                    recency_decay: 0.95,
                },
            })),
        }
    }
    
    /// Main detection method - finds cascades 30 seconds to 2 minutes before impact
    pub async fn detect_ultra_fast_cascade(&self, symbol: &str) -> Option<CascadePattern> {
        let now = Utc::now();
        let mut signals = Vec::new();
        
        // Check all signal sources in parallel
        let (whale_signal, flow_signal, mempool_signal, social_signal, dex_signal) = tokio::join!(
            self.check_whale_activity(symbol),
            self.check_order_flow(symbol),
            self.check_mempool_anomalies(symbol),
            self.check_social_velocity(symbol),
            self.check_dex_flows(symbol)
        );
        
        // Collect valid signals
        if let Some(s) = whale_signal { signals.push(s); }
        if let Some(s) = flow_signal { signals.push(s); }
        if let Some(s) = mempool_signal { signals.push(s); }
        if let Some(s) = social_signal { signals.push(s); }
        if let Some(s) = dex_signal { signals.push(s); }
        
        // Need at least 2 confirming signals
        if signals.len() < 2 {
            return None;
        }
        
        // Calculate cascade metrics
        let cascade_strength = self.calculate_cascade_strength(&signals).await;
        let velocity = self.calculate_signal_velocity(&signals).await;
        let acceleration = self.calculate_signal_acceleration(&signals).await;
        
        // Predict time to impact
        let time_to_impact = self.predict_impact_time(
            cascade_strength,
            velocity,
            acceleration,
            &signals
        ).await;
        
        // Only return if impact is within 2 minutes
        if time_to_impact <= 120000 { // 2 minutes in milliseconds
            let cascade_type = self.determine_cascade_type(&signals).await;
            let confidence = self.calculate_confidence(&signals, cascade_strength).await;
            
            Some(CascadePattern {
                cascade_id: format!("{}-{}", symbol, now.timestamp_millis()),
                symbol: symbol.to_string(),
                cascade_type,
                strength: cascade_strength,
                velocity,
                acceleration,
                time_to_impact_ms: time_to_impact,
                confidence,
                sources: signals.into_iter()
                    .map(|s| SignalSource {
                        source_type: s.source_type,
                        signal_strength: s.strength,
                        timestamp: s.timestamp,
                        reliability_score: s.reliability,
                    })
                    .collect(),
            })
        } else {
            None
        }
    }
    
    // Signal detection methods
    
    async fn check_whale_activity(&self, symbol: &str) -> Option<Signal> {
        let whale_alerts = self.whale_alerts.read().await;
        
        // Look for recent whale transfers
        let recent_transfers: Vec<_> = whale_alerts.real_time_transfers
            .iter()
            .filter(|t| t.token == symbol && t.timestamp > Utc::now() - Duration::seconds(60))
            .collect();
        
        if recent_transfers.is_empty() {
            return None;
        }
        
        // Calculate whale accumulation score
        let mut accumulation_score = 0.0;
        let mut total_usd = 0.0;
        
        for transfer in &recent_transfers {
            if let Some(profile) = whale_alerts.tracked_wallets.get(&transfer.to) {
                // Weight by wallet's historical accuracy
                accumulation_score += profile.accuracy_score;
                total_usd += transfer.amount_usd;
            }
        }
        
        // Significant whale accumulation detected
        if accumulation_score > 2.0 && total_usd > 500000.0 {
            return Some(Signal {
                source_type: SourceType::WhaleWallet(recent_transfers[0].to.clone()),
                strength: (accumulation_score / 5.0).min(1.0),
                timestamp: recent_transfers[0].timestamp,
                reliability: 0.92, // Whale signals are highly reliable
            });
        }
        
        None
    }
    
    async fn check_order_flow(&self, symbol: &str) -> Option<Signal> {
        let flow_analyzer = self.order_flow_analyzer.read().await;
        
        // Get last 30 seconds of flow
        let cutoff = Utc::now() - Duration::seconds(30);
        let recent_flow: Vec<_> = flow_analyzer.flow_history
            .iter()
            .filter(|f| f.timestamp > cutoff)
            .collect();
        
        if recent_flow.len() < 5 {
            return None;
        }
        
        // Calculate flow imbalance
        let total_buy: f64 = recent_flow.iter().map(|f| f.buy_volume).sum();
        let total_sell: f64 = recent_flow.iter().map(|f| f.sell_volume).sum();
        let imbalance = (total_buy - total_sell) / (total_buy + total_sell);
        
        // Check for aggressive sweeps
        let large_orders: Vec<_> = recent_flow.iter()
            .flat_map(|f| &f.large_orders)
            .filter(|o| o.aggressive && o.size_usd > 50000.0)
            .collect();
        
        if imbalance.abs() > 0.7 || large_orders.len() > 3 {
            return Some(Signal {
                source_type: SourceType::OrderBookImbalance,
                strength: imbalance.abs(),
                timestamp: recent_flow.last()?.timestamp,
                reliability: 0.88,
            });
        }
        
        None
    }
    
    async fn check_mempool_anomalies(&self, symbol: &str) -> Option<Signal> {
        let mempool = self.mempool_scanner.read().await;
        
        // Look for unusual transactions
        let mut anomaly_score = 0.0;
        let now = Utc::now();
        
        for (_, tx) in mempool.pending_txs.iter() {
            if tx.detected_at > now - Duration::seconds(10) {
                // High gas transactions
                if tx.gas_price > mempool.anomaly_detector.unusual_gas_threshold {
                    anomaly_score += 0.3;
                }
                
                // Large value transfers
                if tx.value > 100000.0 {
                    anomaly_score += 0.2;
                }
                
                // Known MEV contracts
                if mempool.anomaly_detector.contract_patterns.contains_key(&tx.to) {
                    anomaly_score += 0.5;
                }
            }
        }
        
        if anomaly_score > 1.0 {
            return Some(Signal {
                source_type: SourceType::MempoolTransaction("anomaly".to_string()),
                strength: ((anomaly_score / 2.0) as f64).min(1.0),
                timestamp: now,
                reliability: 0.85,
            });
        }
        
        None
    }
    
    async fn check_social_velocity(&self, symbol: &str) -> Option<Signal> {
        let social_tracker = self.social_velocity_tracker.read().await;
        
        // Calculate mention velocity across platforms
        let mut total_velocity = 0.0;
        let mut max_platform_velocity = 0.0;
        let now = Utc::now();
        
        for (platform, stream) in &social_tracker.mention_streams {
            if let Some(recent_mentions) = stream.mentions_per_second.back() {
                let current_rate = recent_mentions.1;
                let avg_rate = stream.mentions_per_second.iter()
                    .map(|(_, r)| r)
                    .sum::<f64>() / stream.mentions_per_second.len() as f64;
                
                let velocity = current_rate / avg_rate.max(0.1);
                total_velocity += velocity;
                        max_platform_velocity = max_platform_velocity.max(velocity as f64);
            }
        }
        
        // Check for influencer activity
        let mut influencer_signal = false;
        for (handle, profile) in &social_tracker.influencer_tracker.tracked_accounts {
            if let Some(events) = social_tracker.influencer_tracker.cascade_history.get(handle) {
                if let Some(recent) = events.last() {
                    if recent.timestamp > now - Duration::seconds(120) {
                        influencer_signal = true;
                        break;
                    }
                }
            }
        }
        
        // Viral threshold or influencer activity
        if max_platform_velocity > 5.0 || influencer_signal {
            return Some(Signal {
                source_type: SourceType::SocialInfluencer("aggregate".to_string()),
                strength: ((max_platform_velocity / 10.0) as f64).min(1.0),
                timestamp: now,
                reliability: if influencer_signal { 0.90 } else { 0.82 },
            });
        }
        
        None
    }
    
    async fn check_dex_flows(&self, symbol: &str) -> Option<Signal> {
        let dex_monitor = self.dex_flow_monitor.read().await;
        
        // Look for coordinated DEX buying
        let cutoff = Utc::now() - Duration::seconds(20);
        let mut dex_buys = 0;
        let mut total_volume = 0.0;
        
        for (_, aggregator) in &dex_monitor.dex_aggregators {
            let recent_trades: Vec<_> = aggregator.trade_stream
                .iter()
                .filter(|t| t.pair.contains(symbol) && t.timestamp > cutoff)
                .collect();
            
            for trade in recent_trades {
                if trade.side == "buy" && trade.amount_usd > 5000.0 {
                    dex_buys += 1;
                    total_volume += trade.amount_usd;
                }
            }
        }
        
        // Multiple DEX buys in quick succession
        if dex_buys >= 5 && total_volume > 100000.0 {
            return Some(Signal {
                source_type: SourceType::DexTrade("multi-dex".to_string()),
                strength: (dex_buys as f64 / 10.0).min(1.0),
                timestamp: Utc::now(),
                reliability: 0.87,
            });
        }
        
        None
    }
    
    // Helper methods
    
    async fn calculate_cascade_strength(&self, signals: &[Signal]) -> f64 {
        let base_strength = signals.iter()
            .map(|s| s.strength * s.reliability)
            .sum::<f64>() / signals.len() as f64;
        
        // Boost for multiple confirming signals
        let signal_diversity = signals.iter()
            .map(|s| match &s.source_type {
                SourceType::WhaleWallet(_) => 0,
                SourceType::OrderBookImbalance => 1,
                SourceType::MempoolTransaction(_) => 2,
                SourceType::SocialInfluencer(_) => 3,
                SourceType::DexTrade(_) => 4,
                _ => 5,
            })
            .collect::<std::collections::HashSet<_>>()
            .len();
        
        let diversity_bonus = (signal_diversity as f64 - 1.0) * 0.1;
        
        (base_strength + diversity_bonus).min(1.0)
    }
    
    async fn calculate_signal_velocity(&self, signals: &[Signal]) -> f64 {
        // How fast are signals appearing
        if signals.len() < 2 {
            return 0.0;
        }
        
        let time_span = signals.last().unwrap().timestamp - signals.first().unwrap().timestamp;
        let seconds = time_span.num_seconds() as f64;
        
        if seconds > 0.0 {
            signals.len() as f64 / seconds
        } else {
            1.0
        }
    }
    
    async fn calculate_signal_acceleration(&self, signals: &[Signal]) -> f64 {
        // Rate of change of velocity
        if signals.len() < 3 {
            return 0.0;
        }
        
        let mid = signals.len() / 2;
        let first_half_velocity = mid as f64 / 
            (signals[mid-1].timestamp - signals[0].timestamp).num_seconds() as f64;
        let second_half_velocity = (signals.len() - mid) as f64 / 
            (signals.last().unwrap().timestamp - signals[mid].timestamp).num_seconds() as f64;
        
        second_half_velocity - first_half_velocity
    }
    
    async fn predict_impact_time(
        &self,
        strength: f64,
        velocity: f64,
        acceleration: f64,
        signals: &[Signal]
    ) -> u64 {
        let predictor = self.impact_predictor.read().await;
        
        // ML prediction based on features
        let features = vec![strength, velocity, acceleration, signals.len() as f64];
        let mut prediction = predictor.ml_predictor.bias;
        
        for (i, feature) in features.iter().enumerate() {
            if i < predictor.ml_predictor.model_weights.len() {
                prediction += feature * predictor.ml_predictor.model_weights[i];
            }
        }
        
        // Adjust based on signal types
        let has_whale = signals.iter().any(|s| matches!(s.source_type, SourceType::WhaleWallet(_)));
        let has_social = signals.iter().any(|s| matches!(s.source_type, SourceType::SocialInfluencer(_)));
        
        if has_whale {
            prediction *= 0.7; // Whales move markets faster
        }
        if has_social {
            prediction *= 1.2; // Social takes longer to impact
        }
        
        // Convert to milliseconds and clamp
        (prediction * 1000.0).max(15000.0).min(120000.0) as u64
    }
    
    async fn determine_cascade_type(&self, signals: &[Signal]) -> CascadeType {
        // Determine primary cascade type based on strongest signal
        let strongest = signals.iter()
            .max_by(|a, b| (a.strength * a.reliability)
                .partial_cmp(&(b.strength * b.reliability))
                .unwrap())
            .unwrap();
        
        match &strongest.source_type {
            SourceType::WhaleWallet(_) => CascadeType::WhaleAccumulation,
            SourceType::SmartContract(_) => CascadeType::SmartMoneyFlow,
            SourceType::SocialInfluencer(_) => CascadeType::SocialViralSpike,
            SourceType::MempoolTransaction(_) => CascadeType::MempoolAnomalies,
            SourceType::DexTrade(_) => CascadeType::DexAggregation,
            SourceType::OrderBookImbalance => CascadeType::InstitutionalEntry,
        }
    }
    
    async fn calculate_confidence(&self, signals: &[Signal], strength: f64) -> f64 {
        let calculator = self.impact_predictor.read().await;
        
        // Base confidence from strength
        let mut confidence = calculator.confidence_calculator.base_confidence * strength;
        
        // Adjust for signal reliability
        let avg_reliability = signals.iter()
            .map(|s| s.reliability)
            .sum::<f64>() / signals.len() as f64;
        
        confidence *= avg_reliability;
        
        // Boost for multiple independent confirmations
        if signals.len() >= 3 {
            confidence *= 1.05;
        }
        
        confidence.min(0.95)
    }
    
    // Static initialization methods
    
    fn load_whale_wallets() -> HashMap<String, WalletProfile> {
        // In production, load from database
        let mut wallets = HashMap::new();
        
        // Known profitable wallets
        wallets.insert("0x1234...".to_string(), WalletProfile {
            address: "0x1234...".to_string(),
            historical_pnl: 5000000.0,
            accuracy_score: 0.85,
            typical_hold_time: Duration::hours(24),
            influence_score: 0.9,
        });
        
        wallets
    }
    
    fn load_accumulation_patterns() -> Vec<AccumulationPattern> {
        vec![
            AccumulationPattern {
                min_wallets: 3,
                time_window: Duration::minutes(5),
                min_total_usd: 500000.0,
                price_impact_correlation: 0.8,
            },
            AccumulationPattern {
                min_wallets: 5,
                time_window: Duration::minutes(15),
                min_total_usd: 1000000.0,
                price_impact_correlation: 0.7,
            },
        ]
    }
    
    fn load_distribution_patterns() -> Vec<DistributionPattern> {
        vec![
            DistributionPattern {
                wallet_dispersion: 0.8,
                time_clustering: 0.6,
                exchange_deposits: true,
            },
        ]
    }
    
    fn load_influencers() -> HashMap<String, InfluencerProfile> {
        let mut influencers = HashMap::new();
        
        influencers.insert("@cryptowhale".to_string(), InfluencerProfile {
            handle: "@cryptowhale".to_string(),
            platform: Platform::Twitter,
            followers: 500000,
            avg_engagement: 0.05,
            price_impact_history: vec![
                (Utc::now() - Duration::days(1), 0.03),
                (Utc::now() - Duration::days(7), 0.05),
            ],
        });
        
        influencers
    }
    
    fn init_dex_aggregators() -> HashMap<String, DexAggregator> {
        let mut aggregators = HashMap::new();
        
        for dex in &["uniswap", "sushiswap", "pancakeswap", "1inch"] {
            aggregators.insert(dex.to_string(), DexAggregator {
                name: dex.to_string(),
                monitored_pairs: vec!["ETH/USDC".to_string(), "BTC/USDC".to_string()],
                trade_stream: VecDeque::with_capacity(1000),
            });
        }
        
        aggregators
    }
    
    fn load_historical_patterns() -> HashMap<CascadeType, Vec<HistoricalImpact>> {
        let mut patterns = HashMap::new();
        
        patterns.insert(CascadeType::WhaleAccumulation, vec![
            HistoricalImpact {
                cascade_strength: 0.8,
                time_to_impact: Duration::seconds(45),
                price_impact: 0.02,
                accuracy: 0.9,
            },
        ]);
        
        patterns
    }
    
    fn init_source_weights() -> HashMap<SourceType, f64> {
        let mut weights = HashMap::new();
        
        // Reliability weights for different sources
        weights.insert(SourceType::WhaleWallet("".to_string()), 0.9);
        weights.insert(SourceType::OrderBookImbalance, 0.85);
        weights.insert(SourceType::DexTrade("".to_string()), 0.8);
        
        weights
    }
}

// Helper struct for internal signal representation
#[derive(Debug, Clone)]
struct Signal {
    source_type: SourceType,
    strength: f64,
    timestamp: DateTime<Utc>,
    reliability: f64,
}
