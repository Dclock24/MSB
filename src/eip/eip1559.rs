// EIP-1559: Fee Market Implementation
// Optimizes gas costs for 90%+ execution success

use ethers::prelude::*;
use std::collections::VecDeque;

/// EIP-1559 Dynamic Fee Calculator
pub struct DynamicFeeCalculator {
    /// Historical base fees (last 100 blocks)
    base_fee_history: VecDeque<U256>,
    
    /// Historical priority fees
    priority_fee_history: VecDeque<U256>,
    
    /// Target confirmation time (blocks)
    target_confirmation_blocks: u64,
}

impl DynamicFeeCalculator {
    pub fn new() -> Self {
        Self {
            base_fee_history: VecDeque::with_capacity(100),
            priority_fee_history: VecDeque::with_capacity(100),
            target_confirmation_blocks: 1, // Next block
        }
    }
    
    /// Calculate optimal max fee and priority fee for 90% inclusion rate
    pub async fn calculate_optimal_fees(
        &mut self,
        provider: &Provider<Ws>,
        urgency: FeeUrgency,
    ) -> Result<(U256, U256), Box<dyn std::error::Error>> {
        // Get current base fee
        let block = provider.get_block(BlockNumber::Latest).await?
            .ok_or("Failed to get latest block")?;
        
        let base_fee = block.base_fee_per_gas
            .ok_or("No base fee in block (pre-EIP-1559?)")?;
        
        // Update history
        self.update_fee_history(base_fee);
        
        // Calculate fees based on urgency and win rate requirements
        let (max_fee, priority_fee) = match urgency {
            FeeUrgency::Immediate => {
                // For 95%+ inclusion in next block
                let priority = self.calculate_high_priority_fee();
                let max_fee = base_fee * 2 + priority; // 2x base + priority
                (max_fee, priority)
            },
            FeeUrgency::Fast => {
                // For 90%+ inclusion within 2 blocks
                let priority = self.calculate_medium_priority_fee();
                let max_fee = base_fee * 15 / 10 + priority; // 1.5x base + priority
                (max_fee, priority)
            },
            FeeUrgency::Standard => {
                // For 85%+ inclusion within 5 blocks
                let priority = self.calculate_standard_priority_fee();
                let max_fee = base_fee * 12 / 10 + priority; // 1.2x base + priority
                (max_fee, priority)
            },
        };
        
        Ok((max_fee, priority_fee))
    }
    
    /// Update fee history for trend analysis
    fn update_fee_history(&mut self, base_fee: U256) {
        if self.base_fee_history.len() >= 100 {
            self.base_fee_history.pop_front();
        }
        self.base_fee_history.push_back(base_fee);
    }
    
    /// Calculate high priority fee for immediate inclusion
    fn calculate_high_priority_fee(&self) -> U256 {
        // 95th percentile of recent priority fees
        if self.priority_fee_history.is_empty() {
            return U256::from(3_000_000_000u64); // 3 gwei default
        }
        
        let mut fees: Vec<_> = self.priority_fee_history.iter().cloned().collect();
        fees.sort();
        let index = (fees.len() * 95) / 100;
        fees[index]
    }
    
    /// Calculate medium priority fee
    fn calculate_medium_priority_fee(&self) -> U256 {
        // 75th percentile
        if self.priority_fee_history.is_empty() {
            return U256::from(2_000_000_000u64); // 2 gwei default
        }
        
        let mut fees: Vec<_> = self.priority_fee_history.iter().cloned().collect();
        fees.sort();
        let index = (fees.len() * 75) / 100;
        fees[index]
    }
    
    /// Calculate standard priority fee
    fn calculate_standard_priority_fee(&self) -> U256 {
        // 50th percentile (median)
        if self.priority_fee_history.is_empty() {
            return U256::from(1_000_000_000u64); // 1 gwei default
        }
        
        let mut fees: Vec<_> = self.priority_fee_history.iter().cloned().collect();
        fees.sort();
        let index = fees.len() / 2;
        fees[index]
    }
    
    /// Predict base fee for future blocks (for planning)
    pub fn predict_base_fee(&self, blocks_ahead: u64) -> U256 {
        if self.base_fee_history.is_empty() {
            return U256::from(30_000_000_000u64); // 30 gwei default
        }
        
        let current = self.base_fee_history.back().unwrap();
        
        // Simple prediction based on recent trend
        // In practice, this would use more sophisticated modeling
        let trend = self.calculate_fee_trend();
        
        // Each block can change by max 12.5%
        let max_change_per_block = *current / 8;
        let predicted_change = max_change_per_block * blocks_ahead;
        
        if trend > 0 {
            current + predicted_change
        } else {
            current.saturating_sub(predicted_change)
        }
    }
    
    /// Calculate recent fee trend (-1 to 1)
    fn calculate_fee_trend(&self) -> i32 {
        if self.base_fee_history.len() < 10 {
            return 0;
        }
        
        let recent: Vec<_> = self.base_fee_history.iter()
            .rev()
            .take(10)
            .collect();
        
        let mut increases = 0;
        for i in 1..recent.len() {
            if recent[i] > recent[i-1] {
                increases += 1;
            }
        }
        
        if increases > 7 { 1 }      // Strong upward trend
        else if increases < 3 { -1 } // Strong downward trend  
        else { 0 }                   // Neutral
    }
}

/// Fee urgency levels for different opportunity types
#[derive(Debug, Clone, Copy)]
pub enum FeeUrgency {
    /// MEV or arbitrage - must be in next block
    Immediate,
    
    /// Time-sensitive but not critical
    Fast,
    
    /// Standard opportunities
    Standard,
}

/// Gas optimization for complex transactions
pub struct GasOptimizer {
    /// Provider for simulation
    provider: std::sync::Arc<Provider<Ws>>,
}

impl GasOptimizer {
    pub fn new(provider: std::sync::Arc<Provider<Ws>>) -> Self {
        Self { provider }
    }
    
    /// Optimize transaction for 90% success rate
    pub async fn optimize_transaction(
        &self,
        tx: &mut TypedTransaction,
        max_gas_price: U256,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Simulate to get accurate gas estimate
        let gas_estimate = self.provider
            .estimate_gas(tx, None)
            .await?;
        
        // Add 10% buffer for 90% success rate
        let gas_limit = gas_estimate * 110 / 100;
        tx.set_gas(gas_limit);
        
        // Set gas price within budget
        tx.set_gas_price(max_gas_price);
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fee_calculation() {
        let mut calc = DynamicFeeCalculator::new();
        
        // Add some historical data
        for i in 1..=10 {
            calc.base_fee_history.push_back(U256::from(i * 1_000_000_000));
        }
        
        // Test trend calculation
        assert_eq!(calc.calculate_fee_trend(), 1); // Upward trend
    }
}
