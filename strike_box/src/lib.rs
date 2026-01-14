//! ============================================================
//! STRIKE BOX - INSTITUTIONAL BILATERAL EXECUTION FRAMEWORK
//! ============================================================
//! Version: 1.0.0
//! Module: HummingbotArray Integration
//! Grade: Institutional Enterprise
//! Deployment: Binary Terminal Execution
//! ============================================================

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================
// SECTION 1: CORE ENUMERATIONS
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    Long,
    Short,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExitType {
    TakeProfit1,
    TakeProfit2,
    TakeProfit3,
    StopLoss,
    TimeStop,
    Manual,
    Emergency,
    SqueezeProtection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskLevel {
    Safe,
    Moderate,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SystemState {
    Active,
    PausedLongs,
    PausedShorts,
    PausedAll,
    EmergencyHalt,
    Recovering,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    Submitted,
    PartialFill,
    Filled,
    Cancelled,
    Rejected,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GateResult {
    Passed,
    Failed,
    Timeout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PositionStatus {
    Open,
    PartialExit,
    Closed,
    Liquidated,
}

// ============================================================
// SECTION 2: TOKEN VALIDATION CONFIGURATION
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenValidationConfig {
    pub liquidity_min_usd: Decimal,
    pub liquidity_max_usd: Decimal,
    pub liquidity_single_side_min_pct: Decimal,
    pub holder_count_min: u32,
    pub holder_count_preferred: u32,
    pub top_10_concentration_max_pct: Decimal,
    pub single_wallet_max_pct: Decimal,
    pub token_age_min_hours: u32,
    pub token_age_preferred_long_hours: u32,
    pub token_age_preferred_short_hours: u32,
    pub require_verified_contract: bool,
    pub reject_proxy_contracts: bool,
}

impl Default for TokenValidationConfig {
    fn default() -> Self {
        Self {
            liquidity_min_usd: Decimal::new(500_000, 0),
            liquidity_max_usd: Decimal::new(1_000_000, 0),
            liquidity_single_side_min_pct: Decimal::new(40, 2),
            holder_count_min: 25,
            holder_count_preferred: 50,
            top_10_concentration_max_pct: Decimal::new(60, 2),
            single_wallet_max_pct: Decimal::new(20, 2),
            token_age_min_hours: 24,
            token_age_preferred_long_hours: 48,
            token_age_preferred_short_hours: 48,
            require_verified_contract: true,
            reject_proxy_contracts: false,
        }
    }
}

// ============================================================
// SECTION 3: TOKEN DATA SNAPSHOT
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSnapshot {
    pub token_address: String,
    pub token_symbol: String,
    pub liquidity_usd: Decimal,
    pub bid_depth_usd: Decimal,
    pub ask_depth_usd: Decimal,
    pub holder_count: u32,
    pub top_10_concentration_pct: Decimal,
    pub largest_wallet_pct: Decimal,
    pub token_age_hours: u32,
    pub contract_verified: bool,
    pub is_proxy_contract: bool,
    pub deployment_timestamp: DateTime<Utc>,
    pub snapshot_timestamp: DateTime<Utc>,
}

impl TokenSnapshot {
    pub fn liquidity_in_range(&self, config: &TokenValidationConfig) -> bool {
        self.liquidity_usd >= config.liquidity_min_usd
            && self.liquidity_usd <= config.liquidity_max_usd
    }

    pub fn holder_distribution_valid(&self, config: &TokenValidationConfig) -> bool {
        self.holder_count >= config.holder_count_min
            && self.top_10_concentration_pct <= config.top_10_concentration_max_pct
    }

    pub fn has_squeeze_risk(&self, config: &TokenValidationConfig) -> bool {
        self.largest_wallet_pct > config.single_wallet_max_pct
    }
}

// ============================================================
// SECTION 4: SAFETY SCORING SYSTEM
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyScoreConfig {
    pub liquidity_weight: Decimal,
    pub holder_weight: Decimal,
    pub age_weight: Decimal,
    pub contract_weight: Decimal,
    pub long_entry_min: Decimal,
    pub short_entry_min: Decimal,
    pub manual_review_min: Decimal,
    pub auto_reject_below: Decimal,
}

impl Default for SafetyScoreConfig {
    fn default() -> Self {
        Self {
            liquidity_weight: Decimal::new(30, 2),
            holder_weight: Decimal::new(25, 2),
            age_weight: Decimal::new(15, 2),
            contract_weight: Decimal::new(30, 2),
            long_entry_min: Decimal::new(60, 2),
            short_entry_min: Decimal::new(50, 2),
            manual_review_min: Decimal::new(50, 2),
            auto_reject_below: Decimal::new(40, 2),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyScore {
    pub total_score: Decimal,
    pub liquidity_score: Decimal,
    pub holder_score: Decimal,
    pub age_score: Decimal,
    pub contract_score: Decimal,
    pub risk_level: RiskLevel,
    pub calculated_at: DateTime<Utc>,
}

impl SafetyScore {
    pub fn calculate(
        token: &TokenSnapshot,
        config: &SafetyScoreConfig,
        validation: &TokenValidationConfig,
    ) -> Self {
        let liquidity_score = Self::calc_liquidity_score(token, validation);
        let holder_score = Self::calc_holder_score(token, validation);
        let age_score = Self::calc_age_score(token);
        let contract_score = Self::calc_contract_score(token);

        let total_score = (liquidity_score * config.liquidity_weight)
            + (holder_score * config.holder_weight)
            + (age_score * config.age_weight)
            + (contract_score * config.contract_weight);

        let risk_level = Self::classify_risk(total_score);

        Self {
            total_score,
            liquidity_score,
            holder_score,
            age_score,
            contract_score,
            risk_level,
            calculated_at: Utc::now(),
        }
    }

    fn calc_liquidity_score(token: &TokenSnapshot, validation: &TokenValidationConfig) -> Decimal {
        if token.liquidity_usd >= validation.liquidity_min_usd
            && token.liquidity_usd <= validation.liquidity_max_usd
        {
            Decimal::ONE
        } else if token.liquidity_usd >= Decimal::new(400_000, 0)
            && token.liquidity_usd < validation.liquidity_min_usd
        {
            Decimal::new(5, 1)
        } else if token.liquidity_usd > validation.liquidity_max_usd
            && token.liquidity_usd <= Decimal::new(1_200_000, 0)
        {
            Decimal::new(5, 1)
        } else {
            Decimal::ZERO
        }
    }

    fn calc_holder_score(token: &TokenSnapshot, validation: &TokenValidationConfig) -> Decimal {
        if token.holder_count >= validation.holder_count_preferred
            && token.top_10_concentration_pct < Decimal::new(50, 2)
        {
            Decimal::ONE
        } else if token.holder_count >= validation.holder_count_min
            && token.top_10_concentration_pct < validation.top_10_concentration_max_pct
        {
            Decimal::new(7, 1)
        } else {
            Decimal::ZERO
        }
    }

    fn calc_age_score(token: &TokenSnapshot) -> Decimal {
        if token.token_age_hours >= 72 {
            Decimal::ONE
        } else if token.token_age_hours >= 24 {
            Decimal::new(7, 1)
        } else {
            Decimal::ZERO
        }
    }

    fn calc_contract_score(token: &TokenSnapshot) -> Decimal {
        if token.contract_verified {
            Decimal::ONE
        } else {
            Decimal::ZERO
        }
    }

    fn classify_risk(score: Decimal) -> RiskLevel {
        if score >= Decimal::new(70, 2) {
            RiskLevel::Safe
        } else if score >= Decimal::new(50, 2) {
            RiskLevel::Moderate
        } else if score >= Decimal::new(40, 2) {
            RiskLevel::High
        } else {
            RiskLevel::Critical
        }
    }

    pub fn qualifies_for_long(&self, config: &SafetyScoreConfig) -> bool {
        self.total_score >= config.long_entry_min
    }

    pub fn qualifies_for_short(&self, config: &SafetyScoreConfig) -> bool {
        self.total_score >= config.short_entry_min
    }
}

// ============================================================
// SECTION 5: POSITION SIZING FRAMEWORK
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionSizingConfig {
    pub max_single_position_pct: Decimal,
    pub max_sector_exposure_pct: Decimal,
    pub max_correlation_exposure_pct: Decimal,
    pub max_gross_exposure_pct: Decimal,
    pub long_book_max_pct: Decimal,
    pub short_book_max_pct: Decimal,
    pub long_book_max_positions: u32,
    pub short_book_max_positions: u32,
    pub long_order_max_pool_pct: Decimal,
    pub short_order_max_pool_pct: Decimal,
    pub scale_order_count: u32,
    pub scale_interval_seconds: u32,
}

impl Default for PositionSizingConfig {
    fn default() -> Self {
        Self {
            max_single_position_pct: Decimal::new(2, 2),
            max_sector_exposure_pct: Decimal::new(10, 2),
            max_correlation_exposure_pct: Decimal::new(15, 2),
            max_gross_exposure_pct: Decimal::ONE,
            long_book_max_pct: Decimal::new(70, 2),
            short_book_max_pct: Decimal::new(30, 2),
            long_book_max_positions: 10,
            short_book_max_positions: 3,
            long_order_max_pool_pct: Decimal::new(1, 2),
            short_order_max_pool_pct: Decimal::new(5, 3),
            scale_order_count: 4,
            scale_interval_seconds: 30,
        }
    }
}

pub struct LiquidityScaler;

impl LiquidityScaler {
    pub fn max_position_pct(liquidity_usd: Decimal) -> Decimal {
        if liquidity_usd >= Decimal::new(900_000, 0) {
            Decimal::new(2, 2)
        } else if liquidity_usd >= Decimal::new(750_000, 0) {
            Decimal::new(15, 3)
        } else if liquidity_usd >= Decimal::new(600_000, 0) {
            Decimal::new(1, 2)
        } else {
            Decimal::new(5, 3)
        }
    }

    pub fn max_position_usd(portfolio_value: Decimal, liquidity_usd: Decimal) -> Decimal {
        portfolio_value * Self::max_position_pct(liquidity_usd)
    }

    pub fn max_order_vs_pool(liquidity_usd: Decimal, direction: Direction) -> Decimal {
        let pct = match direction {
            Direction::Long => Decimal::new(1, 2),
            Direction::Short => Decimal::new(5, 3),
        };
        liquidity_usd * pct
    }
}

// ============================================================
// SECTION 6: STOP LOSS CONFIGURATION
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopLossConfig {
    pub long_default_pct: Decimal,
    pub long_volatile_pct: Decimal,
    pub long_volatile_min_safety: Decimal,
    pub long_trailing_activation_pct: Decimal,
    pub long_trailing_distance_pct: Decimal,
    pub long_hard_floor_pct: Decimal,
    pub short_fixed_pct: Decimal,
    pub short_squeeze_trigger_pct: Decimal,
    pub short_squeeze_window_seconds: u32,
}

impl Default for StopLossConfig {
    fn default() -> Self {
        Self {
            long_default_pct: Decimal::new(5, 2),
            long_volatile_pct: Decimal::new(8, 2),
            long_volatile_min_safety: Decimal::new(70, 2),
            long_trailing_activation_pct: Decimal::new(15, 2),
            long_trailing_distance_pct: Decimal::new(10, 2),
            long_hard_floor_pct: Decimal::new(10, 2),
            short_fixed_pct: Decimal::new(8, 2),
            short_squeeze_trigger_pct: Decimal::new(5, 2),
            short_squeeze_window_seconds: 3600,
        }
    }
}

impl StopLossConfig {
    pub fn long_stop_price(&self, entry_price: Decimal, safety_score: Decimal) -> Decimal {
        let stop_pct = if safety_score >= self.long_volatile_min_safety {
            self.long_volatile_pct
        } else {
            self.long_default_pct
        };
        entry_price * (Decimal::ONE - stop_pct)
    }

    pub fn short_stop_price(&self, entry_price: Decimal) -> Decimal {
        entry_price * (Decimal::ONE + self.short_fixed_pct)
    }

    pub fn trailing_stop_price(&self, high_water_mark: Decimal) -> Decimal {
        high_water_mark * (Decimal::ONE - self.long_trailing_distance_pct)
    }

    pub fn should_activate_trailing(&self, entry_price: Decimal, current_price: Decimal) -> bool {
        let gain_pct = (current_price - entry_price) / entry_price;
        gain_pct >= self.long_trailing_activation_pct
    }
}

// ============================================================
// SECTION 7: TAKE PROFIT CONFIGURATION
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TakeProfitConfig {
    pub long_tp1_pct: Decimal,
    pub long_tp1_exit_pct: Decimal,
    pub long_tp2_pct: Decimal,
    pub long_tp2_exit_pct: Decimal,
    pub long_tp3_pct: Decimal,
    pub long_tp3_exit_pct: Decimal,
    pub short_tp1_pct: Decimal,
    pub short_tp1_exit_pct: Decimal,
    pub short_tp2_pct: Decimal,
    pub short_tp2_exit_pct: Decimal,
    pub short_tp3_pct: Decimal,
    pub short_tp3_exit_pct: Decimal,
}

impl Default for TakeProfitConfig {
    fn default() -> Self {
        Self {
            long_tp1_pct: Decimal::new(15, 2),
            long_tp1_exit_pct: Decimal::new(33, 2),
            long_tp2_pct: Decimal::new(30, 2),
            long_tp2_exit_pct: Decimal::new(33, 2),
            long_tp3_pct: Decimal::new(50, 2),
            long_tp3_exit_pct: Decimal::new(34, 2),
            short_tp1_pct: Decimal::new(10, 2),
            short_tp1_exit_pct: Decimal::new(33, 2),
            short_tp2_pct: Decimal::new(20, 2),
            short_tp2_exit_pct: Decimal::new(33, 2),
            short_tp3_pct: Decimal::new(30, 2),
            short_tp3_exit_pct: Decimal::new(34, 2),
        }
    }
}

impl TakeProfitConfig {
    pub fn long_tp_prices(&self, entry_price: Decimal) -> [Decimal; 3] {
        [
            entry_price * (Decimal::ONE + self.long_tp1_pct),
            entry_price * (Decimal::ONE + self.long_tp2_pct),
            entry_price * (Decimal::ONE + self.long_tp3_pct),
        ]
    }

    pub fn short_tp_prices(&self, entry_price: Decimal) -> [Decimal; 3] {
        [
            entry_price * (Decimal::ONE - self.short_tp1_pct),
            entry_price * (Decimal::ONE - self.short_tp2_pct),
            entry_price * (Decimal::ONE - self.short_tp3_pct),
        ]
    }

    pub fn exit_percentages(&self, direction: Direction) -> [Decimal; 3] {
        match direction {
            Direction::Long => [
                self.long_tp1_exit_pct,
                self.long_tp2_exit_pct,
                self.long_tp3_exit_pct,
            ],
            Direction::Short => [
                self.short_tp1_exit_pct,
                self.short_tp2_exit_pct,
                self.short_tp3_exit_pct,
            ],
        }
    }
}

// ============================================================
// SECTION 8: TIME CONTROL CONFIGURATION
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeControlConfig {
    pub long_review_days: u32,
    pub long_flag_days: u32,
    pub short_max_hours: u32,
    pub long_no_movement_flag_hours: u32,
}

impl Default for TimeControlConfig {
    fn default() -> Self {
        Self {
            long_review_days: 7,
            long_flag_days: 14,
            short_max_hours: 72,
            long_no_movement_flag_hours: 24,
        }
    }
}

impl TimeControlConfig {
    pub fn long_needs_review(&self, opened_at: DateTime<Utc>) -> bool {
        let duration = Utc::now() - opened_at;
        duration.num_days() >= self.long_review_days as i64
    }

    pub fn long_should_flag(&self, opened_at: DateTime<Utc>) -> bool {
        let duration = Utc::now() - opened_at;
        duration.num_days() >= self.long_flag_days as i64
    }

    pub fn short_time_exceeded(&self, opened_at: DateTime<Utc>) -> bool {
        let duration = Utc::now() - opened_at;
        duration.num_hours() >= self.short_max_hours as i64
    }

    pub fn short_time_stop(&self, opened_at: DateTime<Utc>) -> DateTime<Utc> {
        opened_at + chrono::Duration::hours(self.short_max_hours as i64)
    }
}

// ============================================================
// SECTION 9: RISK CONTROLLER CONFIGURATION
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskControllerConfig {
    pub daily_drawdown_halt_pct: Decimal,
    pub weekly_drawdown_halt_pct: Decimal,
    pub monthly_review_pct: Decimal,
    pub net_exposure_min_pct: Decimal,
    pub net_exposure_max_pct: Decimal,
    pub market_crash_trigger_pct: Decimal,
    pub liquidity_crisis_trigger_pct: Decimal,
    pub execution_failure_max: u32,
    pub data_feed_stale_seconds: u32,
    pub max_latency_ms: u32,
    pub order_ack_timeout_ms: u32,
    pub slippage_pause_pct: Decimal,
    pub slippage_reduce_pct: Decimal,
    pub partial_fill_min_pct: Decimal,
}

impl Default for RiskControllerConfig {
    fn default() -> Self {
        Self {
            daily_drawdown_halt_pct: Decimal::new(5, 2),
            weekly_drawdown_halt_pct: Decimal::new(10, 2),
            monthly_review_pct: Decimal::new(15, 2),
            net_exposure_min_pct: Decimal::new(-30, 2),
            net_exposure_max_pct: Decimal::new(70, 2),
            market_crash_trigger_pct: Decimal::new(15, 2),
            liquidity_crisis_trigger_pct: Decimal::new(50, 2),
            execution_failure_max: 3,
            data_feed_stale_seconds: 30,
            max_latency_ms: 500,
            order_ack_timeout_ms: 1000,
            slippage_pause_pct: Decimal::new(5, 3),
            slippage_reduce_pct: Decimal::new(15, 3),
            partial_fill_min_pct: Decimal::new(80, 2),
        }
    }
}

// ============================================================
// SECTION 10: RISK GATE VALIDATION
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskGateCheck {
    pub gate_name: String,
    pub result: GateResult,
    pub reason: Option<String>,
    pub checked_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskValidation {
    pub validation_id: Uuid,
    pub direction: Direction,
    pub gates: Vec<RiskGateCheck>,
    pub all_passed: bool,
    pub validated_at: DateTime<Utc>,
}

impl RiskValidation {
    pub fn new(direction: Direction) -> Self {
        Self {
            validation_id: Uuid::new_v4(),
            direction,
            gates: Vec::with_capacity(10),
            all_passed: true,
            validated_at: Utc::now(),
        }
    }

    pub fn add_gate(&mut self, name: &str, result: GateResult, reason: Option<String>) {
        if result != GateResult::Passed {
            self.all_passed = false;
        }
        self.gates.push(RiskGateCheck {
            gate_name: name.to_string(),
            result,
            reason,
            checked_at: Utc::now(),
        });
    }

    pub fn first_failure(&self) -> Option<&RiskGateCheck> {
        self.gates.iter().find(|g| g.result == GateResult::Failed)
    }

    pub fn all_failures(&self) -> Vec<&RiskGateCheck> {
        self.gates
            .iter()
            .filter(|g| g.result == GateResult::Failed)
            .collect()
    }
}

// ============================================================
// SECTION 11: POSITION STRUCTURES
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub execution_id: Uuid,
    pub token_address: String,
    pub token_symbol: String,
    pub direction: Direction,
    pub entry_price: Decimal,
    pub current_price: Decimal,
    pub position_size_tokens: Decimal,
    pub position_size_usd: Decimal,
    pub remaining_size_pct: Decimal,
    pub liquidity_at_entry: Decimal,
    pub safety_score_at_entry: Decimal,
    pub holder_count_at_entry: u32,
    pub stop_loss_price: Decimal,
    pub take_profit_prices: [Decimal; 3],
    pub take_profit_hit: [bool; 3],
    pub risk_approval_id: Uuid,
    pub opened_at: DateTime<Utc>,
    pub time_stop_at: Option<DateTime<Utc>>,
    pub status: PositionStatus,
    pub trailing_stop_active: bool,
    pub trailing_stop_high: Option<Decimal>,
    pub unrealized_pnl_usd: Decimal,
    pub unrealized_pnl_pct: Decimal,
}

impl Position {
    pub fn update_price(&mut self, new_price: Decimal) {
        self.current_price = new_price;
        self.unrealized_pnl_usd = match self.direction {
            Direction::Long => {
                (new_price - self.entry_price) * self.position_size_tokens * self.remaining_size_pct
            }
            Direction::Short => {
                (self.entry_price - new_price) * self.position_size_tokens * self.remaining_size_pct
            }
        };
        self.unrealized_pnl_pct = self.unrealized_pnl_usd / self.position_size_usd;

        if self.trailing_stop_active && self.direction == Direction::Long {
            if let Some(hwm) = self.trailing_stop_high {
                if new_price > hwm {
                    self.trailing_stop_high = Some(new_price);
                }
            }
        }
    }

    pub fn stop_triggered(&self) -> bool {
        match self.direction {
            Direction::Long => self.current_price <= self.stop_loss_price,
            Direction::Short => self.current_price >= self.stop_loss_price,
        }
    }

    pub fn check_take_profits(&self) -> Option<usize> {
        for (i, &tp_price) in self.take_profit_prices.iter().enumerate() {
            if self.take_profit_hit[i] {
                continue;
            }
            let triggered = match self.direction {
                Direction::Long => self.current_price >= tp_price,
                Direction::Short => self.current_price <= tp_price,
            };
            if triggered {
                return Some(i);
            }
        }
        None
    }

    pub fn trailing_stop_triggered(&self, config: &StopLossConfig) -> bool {
        if !self.trailing_stop_active {
            return false;
        }
        if let Some(hwm) = self.trailing_stop_high {
            let trailing_price = config.trailing_stop_price(hwm);
            self.current_price <= trailing_price
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionExit {
    pub execution_id: Uuid,
    pub exit_price: Decimal,
    pub exit_type: ExitType,
    pub exit_size_pct: Decimal,
    pub realized_pnl_tokens: Decimal,
    pub realized_pnl_usd: Decimal,
    pub slippage_bps: Decimal,
    pub hold_duration_seconds: u64,
    pub liquidity_at_exit: Decimal,
    pub exited_at: DateTime<Utc>,
}

// ============================================================
// SECTION 12: BOOK MANAGEMENT
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionBook {
    pub direction: Direction,
    pub positions: Vec<Position>,
    pub total_allocation_usd: Decimal,
    pub max_allocation_usd: Decimal,
    pub max_positions: u32,
    pub realized_pnl_usd: Decimal,
    pub unrealized_pnl_usd: Decimal,
}

impl PositionBook {
    pub fn new(direction: Direction, max_allocation_usd: Decimal, max_positions: u32) -> Self {
        Self {
            direction,
            positions: Vec::with_capacity(max_positions as usize),
            total_allocation_usd: Decimal::ZERO,
            max_allocation_usd,
            max_positions,
            realized_pnl_usd: Decimal::ZERO,
            unrealized_pnl_usd: Decimal::ZERO,
        }
    }

    pub fn can_add_position(&self, size_usd: Decimal) -> bool {
        self.position_count() < self.max_positions
            && (self.total_allocation_usd + size_usd) <= self.max_allocation_usd
    }

    pub fn position_count(&self) -> u32 {
        self.positions.len() as u32
    }

    pub fn available_capacity_usd(&self) -> Decimal {
        self.max_allocation_usd - self.total_allocation_usd
    }

    pub fn update_unrealized_pnl(&mut self) {
        self.unrealized_pnl_usd = self.positions.iter().map(|p| p.unrealized_pnl_usd).sum();
    }

    pub fn has_position(&self, token_address: &str) -> bool {
        self.positions
            .iter()
            .any(|p| p.token_address == token_address && p.status == PositionStatus::Open)
    }

    pub fn get_position(&self, token_address: &str) -> Option<&Position> {
        self.positions
            .iter()
            .find(|p| p.token_address == token_address && p.status == PositionStatus::Open)
    }

    pub fn get_position_mut(&mut self, token_address: &str) -> Option<&mut Position> {
        self.positions
            .iter_mut()
            .find(|p| p.token_address == token_address && p.status == PositionStatus::Open)
    }
}

// ============================================================
// SECTION 13: PORTFOLIO STATE
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioState {
    pub total_capital_usd: Decimal,
    pub available_capital_usd: Decimal,
    pub long_book: PositionBook,
    pub short_book: PositionBook,
    pub gross_exposure_pct: Decimal,
    pub net_exposure_pct: Decimal,
    pub gross_exposure_usd: Decimal,
    pub net_exposure_usd: Decimal,
    pub daily_pnl_usd: Decimal,
    pub weekly_pnl_usd: Decimal,
    pub monthly_pnl_usd: Decimal,
    pub daily_drawdown_pct: Decimal,
    pub weekly_drawdown_pct: Decimal,
    pub monthly_drawdown_pct: Decimal,
    pub daily_high_water_mark: Decimal,
    pub weekly_high_water_mark: Decimal,
    pub monthly_high_water_mark: Decimal,
    pub state: SystemState,
    pub consecutive_failures: u32,
    pub last_updated: DateTime<Utc>,
}

impl PortfolioState {
    pub fn calculate_exposure(&mut self) {
        let long_exposure = self.long_book.total_allocation_usd;
        let short_exposure = self.short_book.total_allocation_usd;

        self.gross_exposure_usd = long_exposure + short_exposure;
        self.net_exposure_usd = long_exposure - short_exposure;

        if self.total_capital_usd > Decimal::ZERO {
            self.gross_exposure_pct = self.gross_exposure_usd / self.total_capital_usd;
            self.net_exposure_pct = self.net_exposure_usd / self.total_capital_usd;
        }

        self.available_capital_usd = self.total_capital_usd - self.gross_exposure_usd;
        self.last_updated = Utc::now();
    }

    pub fn check_drawdown_limits(&self, config: &RiskControllerConfig) -> SystemState {
        if self.monthly_drawdown_pct >= config.monthly_review_pct {
            return SystemState::EmergencyHalt;
        }
        if self.weekly_drawdown_pct >= config.weekly_drawdown_halt_pct {
            return SystemState::PausedAll;
        }
        if self.daily_drawdown_pct >= config.daily_drawdown_halt_pct {
            return SystemState::PausedAll;
        }
        SystemState::Active
    }

    pub fn net_exposure_valid(&self, config: &RiskControllerConfig) -> bool {
        self.net_exposure_pct >= config.net_exposure_min_pct
            && self.net_exposure_pct <= config.net_exposure_max_pct
    }

    pub fn update_drawdowns(&mut self) {
        let current_value = self.total_capital_usd
            + self.long_book.unrealized_pnl_usd
            + self.short_book.unrealized_pnl_usd;

        if current_value > self.daily_high_water_mark {
            self.daily_high_water_mark = current_value;
        }
        self.daily_drawdown_pct = if self.daily_high_water_mark > Decimal::ZERO {
            (self.daily_high_water_mark - current_value) / self.daily_high_water_mark
        } else {
            Decimal::ZERO
        };

        if current_value > self.weekly_high_water_mark {
            self.weekly_high_water_mark = current_value;
        }
        self.weekly_drawdown_pct = if self.weekly_high_water_mark > Decimal::ZERO {
            (self.weekly_high_water_mark - current_value) / self.weekly_high_water_mark
        } else {
            Decimal::ZERO
        };

        if current_value > self.monthly_high_water_mark {
            self.monthly_high_water_mark = current_value;
        }
        self.monthly_drawdown_pct = if self.monthly_high_water_mark > Decimal::ZERO {
            (self.monthly_high_water_mark - current_value) / self.monthly_high_water_mark
        } else {
            Decimal::ZERO
        };
    }
}

// ============================================================
// SECTION 14: AUDIT LOGGING
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryLog {
    pub execution_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub token_address: String,
    pub token_symbol: String,
    pub direction: Direction,
    pub entry_price: Decimal,
    pub position_size_tokens: Decimal,
    pub position_size_usd: Decimal,
    pub liquidity_depth_usd: Decimal,
    pub safety_score: Decimal,
    pub holder_count: u32,
    pub stop_loss_price: Decimal,
    pub take_profit_prices: [Decimal; 3],
    pub risk_approval_id: Uuid,
    pub latency_ms: u32,
    pub slippage_bps: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitLog {
    pub execution_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub exit_price: Decimal,
    pub exit_type: ExitType,
    pub exit_size_pct: Decimal,
    pub realized_pnl_tokens: Decimal,
    pub realized_pnl_usd: Decimal,
    pub slippage_bps: Decimal,
    pub hold_duration_seconds: u64,
    pub liquidity_depth_exit_usd: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectionLog {
    pub timestamp: DateTime<Utc>,
    pub token_address: String,
    pub token_symbol: String,
    pub direction: Direction,
    pub rejection_reason: String,
    pub failed_gate: String,
    pub safety_score: Option<Decimal>,
    pub liquidity_usd: Option<Decimal>,
}

// ============================================================
// SECTION 15: OPERATIONAL COMMANDS
// ============================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationalCommand {
    PauseLongs,
    PauseShorts,
    PauseAll,
    CloseLongs,
    CloseShorts,
    CloseAll,
    Resume,
    Status,
    Exposure,
    Risk,
    Health,
    Position { token: String },
    History { token: String },
    Pnl { timeframe: String },
    Rejects { timeframe: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResponse {
    pub command: String,
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub executed_at: DateTime<Utc>,
}

// ============================================================
// SECTION 16: MASTER CONFIGURATION
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrikeBoxConfig {
    pub token_validation: TokenValidationConfig,
    pub safety_scoring: SafetyScoreConfig,
    pub position_sizing: PositionSizingConfig,
    pub stop_loss: StopLossConfig,
    pub take_profit: TakeProfitConfig,
    pub time_control: TimeControlConfig,
    pub risk_controller: RiskControllerConfig,
}

impl Default for StrikeBoxConfig {
    fn default() -> Self {
        Self {
            token_validation: TokenValidationConfig::default(),
            safety_scoring: SafetyScoreConfig::default(),
            position_sizing: PositionSizingConfig::default(),
            stop_loss: StopLossConfig::default(),
            take_profit: TakeProfitConfig::default(),
            time_control: TimeControlConfig::default(),
            risk_controller: RiskControllerConfig::default(),
        }
    }
}

// ============================================================
// SECTION 17: STRIKE BOX ENGINE
// ============================================================

pub struct StrikeBoxEngine {
    pub config: StrikeBoxConfig,
    pub portfolio: PortfolioState,
    pub entry_logs: Vec<EntryLog>,
    pub exit_logs: Vec<ExitLog>,
    pub rejection_logs: Vec<RejectionLog>,
}

impl StrikeBoxEngine {
    pub fn new(config: StrikeBoxConfig, total_capital: Decimal) -> Self {
        let long_max = total_capital * config.position_sizing.long_book_max_pct;
        let short_max = total_capital * config.position_sizing.short_book_max_pct;

        Self {
            config: config.clone(),
            portfolio: PortfolioState {
                total_capital_usd: total_capital,
                available_capital_usd: total_capital,
                long_book: PositionBook::new(
                    Direction::Long,
                    long_max,
                    config.position_sizing.long_book_max_positions,
                ),
                short_book: PositionBook::new(
                    Direction::Short,
                    short_max,
                    config.position_sizing.short_book_max_positions,
                ),
                gross_exposure_pct: Decimal::ZERO,
                net_exposure_pct: Decimal::ZERO,
                gross_exposure_usd: Decimal::ZERO,
                net_exposure_usd: Decimal::ZERO,
                daily_pnl_usd: Decimal::ZERO,
                weekly_pnl_usd: Decimal::ZERO,
                monthly_pnl_usd: Decimal::ZERO,
                daily_drawdown_pct: Decimal::ZERO,
                weekly_drawdown_pct: Decimal::ZERO,
                monthly_drawdown_pct: Decimal::ZERO,
                daily_high_water_mark: total_capital,
                weekly_high_water_mark: total_capital,
                monthly_high_water_mark: total_capital,
                state: SystemState::Active,
                consecutive_failures: 0,
                last_updated: Utc::now(),
            },
            entry_logs: Vec::new(),
            exit_logs: Vec::new(),
            rejection_logs: Vec::new(),
        }
    }

    pub fn validate_entry(&self, token: &TokenSnapshot, direction: Direction) -> RiskValidation {
        let mut validation = RiskValidation::new(direction);

        match self.portfolio.state {
            SystemState::Active => {
                validation.add_gate("system_state", GateResult::Passed, None);
            }
            SystemState::PausedLongs if direction == Direction::Short => {
                validation.add_gate("system_state", GateResult::Passed, None);
            }
            SystemState::PausedShorts if direction == Direction::Long => {
                validation.add_gate("system_state", GateResult::Passed, None);
            }
            _ => {
                validation.add_gate(
                    "system_state",
                    GateResult::Failed,
                    Some(format!("System state {:?} blocks {:?} entries", self.portfolio.state, direction)),
                );
                return validation;
            }
        }

        if !token.liquidity_in_range(&self.config.token_validation) {
            validation.add_gate(
                "liquidity_range",
                GateResult::Failed,
                Some(format!(
                    "Liquidity ${} outside ${}-${} range",
                    token.liquidity_usd,
                    self.config.token_validation.liquidity_min_usd,
                    self.config.token_validation.liquidity_max_usd
                )),
            );
            return validation;
        }
        validation.add_gate("liquidity_range", GateResult::Passed, None);

        let safety = SafetyScore::calculate(
            token,
            &self.config.safety_scoring,
            &self.config.token_validation,
        );
        let score_ok = match direction {
            Direction::Long => safety.qualifies_for_long(&self.config.safety_scoring),
            Direction::Short => safety.qualifies_for_short(&self.config.safety_scoring),
        };
        if !score_ok {
            validation.add_gate(
                "safety_score",
                GateResult::Failed,
                Some(format!(
                    "Score {:.2} below {:?} threshold",
                    safety.total_score,
                    direction
                )),
            );
            return validation;
        }
        validation.add_gate("safety_score", GateResult::Passed, None);

        if token.token_age_hours < self.config.token_validation.token_age_min_hours {
            validation.add_gate(
                "token_age",
                GateResult::Failed,
                Some(format!(
                    "Token age {}h below {}h minimum",
                    token.token_age_hours, self.config.token_validation.token_age_min_hours
                )),
            );
            return validation;
        }
        validation.add_gate("token_age", GateResult::Passed, None);

        if self.config.token_validation.require_verified_contract && !token.contract_verified {
            validation.add_gate(
                "contract_verification",
                GateResult::Failed,
                Some("Contract not verified".to_string()),
            );
            return validation;
        }
        validation.add_gate("contract_verification", GateResult::Passed, None);

        if !token.holder_distribution_valid(&self.config.token_validation) {
            validation.add_gate(
                "holder_distribution",
                GateResult::Failed,
                Some(format!(
                    "Holders {} or concentration {:.1}% fails requirements",
                    token.holder_count, token.top_10_concentration_pct
                )),
            );
            return validation;
        }
        validation.add_gate("holder_distribution", GateResult::Passed, None);

        let book = match direction {
            Direction::Long => &self.portfolio.long_book,
            Direction::Short => &self.portfolio.short_book,
        };
        if book.position_count() >= book.max_positions {
            validation.add_gate(
                "book_capacity",
                GateResult::Failed,
                Some(format!("{:?} book at max {} positions", direction, book.max_positions)),
            );
            return validation;
        }
        validation.add_gate("book_capacity", GateResult::Passed, None);

        if book.has_position(&token.token_address) {
            validation.add_gate(
                "no_stacking",
                GateResult::Failed,
                Some("Position already exists for token".to_string()),
            );
            return validation;
        }
        validation.add_gate("no_stacking", GateResult::Passed, None);

        if direction == Direction::Short && token.has_squeeze_risk(&self.config.token_validation) {
            validation.add_gate(
                "squeeze_risk",
                GateResult::Failed,
                Some(format!(
                    "Largest wallet {:.1}% exceeds squeeze threshold",
                    token.largest_wallet_pct
                )),
            );
            return validation;
        }
        if direction == Direction::Short {
            validation.add_gate("squeeze_risk", GateResult::Passed, None);
        }

        if !self.portfolio.net_exposure_valid(&self.config.risk_controller) {
            validation.add_gate(
                "net_exposure",
                GateResult::Failed,
                Some("Net exposure outside bounds".to_string()),
            );
            return validation;
        }
        validation.add_gate("net_exposure", GateResult::Passed, None);

        validation
    }

    pub fn calculate_position_size(&self, token: &TokenSnapshot, direction: Direction) -> Decimal {
        let base_max_pct = LiquidityScaler::max_position_pct(token.liquidity_usd);
        let max_usd = self.portfolio.total_capital_usd * base_max_pct;
        let pool_limit_usd = LiquidityScaler::max_order_vs_pool(token.liquidity_usd, direction);
        max_usd.min(pool_limit_usd)
    }

    pub fn execute_command(&mut self, command: OperationalCommand) -> CommandResponse {
        let (success, message) = match command {
            OperationalCommand::PauseLongs => {
                self.portfolio.state = SystemState::PausedLongs;
                (true, "Long entries paused".to_string())
            }
            OperationalCommand::PauseShorts => {
                self.portfolio.state = SystemState::PausedShorts;
                (true, "Short entries paused".to_string())
            }
            OperationalCommand::PauseAll => {
                self.portfolio.state = SystemState::PausedAll;
                (true, "All entries paused".to_string())
            }
            OperationalCommand::Resume => {
                let drawdown_state = self.portfolio.check_drawdown_limits(&self.config.risk_controller);
                if drawdown_state == SystemState::Active {
                    self.portfolio.state = SystemState::Active;
                    (true, "System resumed".to_string())
                } else {
                    (false, format!("Cannot resume - drawdown limits require {:?}", drawdown_state))
                }
            }
            OperationalCommand::Status => {
                let msg = format!(
                    "State: {:?} | Longs: {}/{} | Shorts: {}/{} | Gross: {:.1}% | Net: {:.1}%",
                    self.portfolio.state,
                    self.portfolio.long_book.position_count(),
                    self.portfolio.long_book.max_positions,
                    self.portfolio.short_book.position_count(),
                    self.portfolio.short_book.max_positions,
                    self.portfolio.gross_exposure_pct * Decimal::new(100, 0),
                    self.portfolio.net_exposure_pct * Decimal::new(100, 0)
                );
                (true, msg)
            }
            OperationalCommand::Exposure => {
                let msg = format!(
                    "Gross: ${:.2} ({:.1}%) | Net: ${:.2} ({:.1}%)",
                    self.portfolio.gross_exposure_usd,
                    self.portfolio.gross_exposure_pct * Decimal::new(100, 0),
                    self.portfolio.net_exposure_usd,
                    self.portfolio.net_exposure_pct * Decimal::new(100, 0)
                );
                (true, msg)
            }
            OperationalCommand::Risk => {
                let msg = format!(
                    "Daily DD: {:.2}% | Weekly DD: {:.2}% | Monthly DD: {:.2}%",
                    self.portfolio.daily_drawdown_pct * Decimal::new(100, 0),
                    self.portfolio.weekly_drawdown_pct * Decimal::new(100, 0),
                    self.portfolio.monthly_drawdown_pct * Decimal::new(100, 0)
                );
                (true, msg)
            }
            OperationalCommand::Health => {
                let msg = format!(
                    "State: {:?} | Capital: ${:.2} | Available: ${:.2}",
                    self.portfolio.state,
                    self.portfolio.total_capital_usd,
                    self.portfolio.available_capital_usd
                );
                (true, msg)
            }
            OperationalCommand::CloseLongs => {
                let count = self.portfolio.long_book.position_count();
                (true, format!("Close {} long positions - MANUAL EXECUTION REQUIRED", count))
            }
            OperationalCommand::CloseShorts => {
                let count = self.portfolio.short_book.position_count();
                (true, format!("Close {} short positions - MANUAL EXECUTION REQUIRED", count))
            }
            OperationalCommand::CloseAll => {
                let total = self.portfolio.long_book.position_count()
                    + self.portfolio.short_book.position_count();
                self.portfolio.state = SystemState::EmergencyHalt;
                (true, format!("EMERGENCY: Close {} total positions", total))
            }
            _ => (true, "Command acknowledged".to_string()),
        };

        CommandResponse {
            command: format!("{:?}", command),
            success,
            message,
            data: None,
            executed_at: Utc::now(),
        }
    }

    pub fn log_rejection(
        &mut self,
        token: &TokenSnapshot,
        direction: Direction,
        validation: &RiskValidation,
    ) {
        if let Some(failure) = validation.first_failure() {
            let safety = SafetyScore::calculate(
                token,
                &self.config.safety_scoring,
                &self.config.token_validation,
            );

            self.rejection_logs.push(RejectionLog {
                timestamp: Utc::now(),
                token_address: token.token_address.clone(),
                token_symbol: token.token_symbol.clone(),
                direction,
                rejection_reason: failure.reason.clone().unwrap_or_default(),
                failed_gate: failure.gate_name.clone(),
                safety_score: Some(safety.total_score),
                liquidity_usd: Some(token.liquidity_usd),
            });
        }
    }
}

// ============================================================
// SECTION 18: UNIT TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_token() -> TokenSnapshot {
        TokenSnapshot {
            token_address: "0x1234567890abcdef".to_string(),
            token_symbol: "TEST".to_string(),
            liquidity_usd: Decimal::new(750_000, 0),
            bid_depth_usd: Decimal::new(375_000, 0),
            ask_depth_usd: Decimal::new(375_000, 0),
            holder_count: 60,
            top_10_concentration_pct: Decimal::new(45, 2),
            largest_wallet_pct: Decimal::new(12, 2),
            token_age_hours: 48,
            contract_verified: true,
            is_proxy_contract: false,
            deployment_timestamp: Utc::now(),
            snapshot_timestamp: Utc::now(),
        }
    }

    #[test]
    fn test_default_config_values() {
        let config = StrikeBoxConfig::default();
        assert_eq!(config.token_validation.liquidity_min_usd, Decimal::new(500_000, 0));
        assert_eq!(config.token_validation.liquidity_max_usd, Decimal::new(1_000_000, 0));
        assert_eq!(config.token_validation.holder_count_min, 25);
        assert_eq!(config.token_validation.token_age_min_hours, 24);
        assert_eq!(config.position_sizing.long_book_max_positions, 10);
        assert_eq!(config.position_sizing.short_book_max_positions, 3);
        assert_eq!(config.stop_loss.short_fixed_pct, Decimal::new(8, 2));
    }

    #[test]
    fn test_safety_score_calculation() {
        let config = SafetyScoreConfig::default();
        let validation = TokenValidationConfig::default();
        let token = create_test_token();
        let score = SafetyScore::calculate(&token, &config, &validation);
        assert!(score.total_score > Decimal::new(60, 2));
        assert!(score.qualifies_for_long(&config));
        assert!(score.qualifies_for_short(&config));
    }

    #[test]
    fn test_liquidity_scaler() {
        assert_eq!(LiquidityScaler::max_position_pct(Decimal::new(550_000, 0)), Decimal::new(5, 3));
        assert_eq!(LiquidityScaler::max_position_pct(Decimal::new(700_000, 0)), Decimal::new(1, 2));
        assert_eq!(LiquidityScaler::max_position_pct(Decimal::new(800_000, 0)), Decimal::new(15, 3));
        assert_eq!(LiquidityScaler::max_position_pct(Decimal::new(950_000, 0)), Decimal::new(2, 2));
    }

    #[test]
    fn test_engine_validation_passing() {
        let config = StrikeBoxConfig::default();
        let engine = StrikeBoxEngine::new(config, Decimal::new(1_000_000, 0));
        let token = create_test_token();
        let validation = engine.validate_entry(&token, Direction::Long);
        assert!(validation.all_passed);
    }

    #[test]
    fn test_stop_loss_calculations() {
        let config = StopLossConfig::default();
        let entry_price = Decimal::new(100, 0);
        let long_stop = config.long_stop_price(entry_price, Decimal::new(60, 2));
        assert_eq!(long_stop, Decimal::new(95, 0));
        let short_stop = config.short_stop_price(entry_price);
        assert_eq!(short_stop, Decimal::new(108, 0));
    }

    #[test]
    fn test_take_profit_calculations() {
        let config = TakeProfitConfig::default();
        let entry_price = Decimal::new(100, 0);
        let long_tps = config.long_tp_prices(entry_price);
        assert_eq!(long_tps[0], Decimal::new(115, 0));
        assert_eq!(long_tps[1], Decimal::new(130, 0));
        assert_eq!(long_tps[2], Decimal::new(150, 0));
    }

    #[test]
    fn test_operational_commands() {
        let config = StrikeBoxConfig::default();
        let mut engine = StrikeBoxEngine::new(config, Decimal::new(1_000_000, 0));
        let response = engine.execute_command(OperationalCommand::PauseAll);
        assert!(response.success);
        assert_eq!(engine.portfolio.state, SystemState::PausedAll);
    }
}

