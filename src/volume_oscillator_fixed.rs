// Production-Ready Volume Oscillator
// Fixed with proper error handling, validation, and bounds checking

use crate::errors::{TradingResult, TradingError, validate_non_negative, safe_divide};
use std::collections::VecDeque;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};

const MAX_HISTORY_SIZE: usize = 10000;
const MIN_WINDOW_SIZE: usize = 20;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeOscillatorFixed {
    window_size: usize,
    max_history_size: usize,
    volume_history: VecDeque<f64>,
    oscillator_history: VecDeque<f64>,
    velocity_history: VecDeque<f64>,
    acceleration_history: VecDeque<f64>,
    last_update: DateTime<Utc>,
    update_count: u64,
}

impl VolumeOscillatorFixed {
    pub fn new(window_size: usize) -> TradingResult<Self> {
        if window_size < MIN_WINDOW_SIZE {
            return Err(TradingError::InvalidInput(
                format!("Window size must be at least {}, got {}", MIN_WINDOW_SIZE, window_size)
            ));
        }
        
        Ok(Self {
            window_size,
            max_history_size: window_size * 10,
            volume_history: VecDeque::with_capacity(window_size * 10),
            oscillator_history: VecDeque::with_capacity(window_size),
            velocity_history: VecDeque::with_capacity(window_size),
            acceleration_history: VecDeque::with_capacity(window_size),
            last_update: Utc::now(),
            update_count: 0,
        })
    }

    pub fn update(&mut self, volume: f64) -> TradingResult<OscillatorSignal> {
        // Validate input
        let volume = validate_non_negative(volume, "volume")?;
        
        // Enforce memory bounds
        if self.volume_history.len() >= self.max_history_size {
            // Remove oldest 10% when limit reached
            let remove_count = self.max_history_size / 10;
            for _ in 0..remove_count {
                self.volume_history.pop_front();
            }
            warn!("Volume history limit reached, removed {} oldest entries", remove_count);
        }

        self.volume_history.push_back(volume);
        self.update_count += 1;
        self.last_update = Utc::now();

        // Calculate oscillator with error handling
        let oscillator = self.calculate_oscillator()?;
        
        // Enforce bounds on oscillator history
        self.oscillator_history.push_back(oscillator);
        if self.oscillator_history.len() > self.window_size {
            self.oscillator_history.pop_front();
        }

        // Calculate velocity with error handling
        let velocity = self.calculate_velocity()?;
        self.velocity_history.push_back(velocity);
        if self.velocity_history.len() > self.window_size {
            self.velocity_history.pop_front();
        }

        // Calculate acceleration with error handling
        let acceleration = self.calculate_acceleration()?;
        self.acceleration_history.push_back(acceleration);
        if self.acceleration_history.len() > self.window_size {
            self.acceleration_history.pop_front();
        }

        // Generate signal
        self.generate_signal(oscillator, velocity, acceleration, volume)
    }

    fn calculate_oscillator(&self) -> TradingResult<f64> {
        if self.volume_history.len() < MIN_WINDOW_SIZE {
            return Ok(0.0);
        }

        let recent_volumes: Vec<f64> = self.volume_history
            .iter()
            .rev()
            .take(MIN_WINDOW_SIZE)
            .copied()
            .collect();
        
        // Validate all volumes are finite
        for (i, v) in recent_volumes.iter().enumerate() {
            if !v.is_finite() {
                return Err(TradingError::InvalidInput(
                    format!("Volume at index {} is not finite: {}", i, v)
                ));
            }
        }
        
        let sum: f64 = recent_volumes.iter().sum();
        let ma = safe_divide(sum, MIN_WINDOW_SIZE as f64, "moving average")?;
        
        let variance = recent_volumes.iter()
            .map(|v| {
                let diff = v - ma;
                diff * diff
            })
            .sum::<f64>();
        let variance = safe_divide(variance, MIN_WINDOW_SIZE as f64, "variance")?;
        
        let std_dev = variance.sqrt();
        
        if std_dev == 0.0 || !std_dev.is_finite() {
            return Ok(0.0);
        }

        let current_volume = *self.volume_history.back()
            .ok_or_else(|| TradingError::InvalidInput("No volume data available".to_string()))?;
        
        let oscillator = safe_divide(current_volume - ma, std_dev, "oscillator")?;
        
        Ok(oscillator)
    }

    fn calculate_velocity(&self) -> TradingResult<f64> {
        if self.oscillator_history.len() < 2 {
            return Ok(0.0);
        }

        let current = *self.oscillator_history.back()
            .ok_or_else(|| TradingError::InvalidInput("No oscillator data".to_string()))?;
        
        let idx = self.oscillator_history.len() - 2;
        let previous = self.oscillator_history.get(idx)
            .ok_or_else(|| TradingError::InvalidInput("Previous oscillator data missing".to_string()))?;
        
        Ok(current - previous)
    }

    fn calculate_acceleration(&self) -> TradingResult<f64> {
        if self.velocity_history.len() < 2 {
            return Ok(0.0);
        }

        let current = *self.velocity_history.back()
            .ok_or_else(|| TradingError::InvalidInput("No velocity data".to_string()))?;
        
        let idx = self.velocity_history.len() - 2;
        let previous = self.velocity_history.get(idx)
            .ok_or_else(|| TradingError::InvalidInput("Previous velocity data missing".to_string()))?;
        
        Ok(current - previous)
    }

    fn generate_signal(
        &self,
        oscillator: f64,
        velocity: f64,
        acceleration: f64,
        volume: f64,
    ) -> TradingResult<OscillatorSignal> {
        // Validate inputs
        if !oscillator.is_finite() || !velocity.is_finite() || !acceleration.is_finite() {
            return Err(TradingError::InvalidInput(
                "Non-finite values in signal generation".to_string()
            ));
        }

        let ma_volume = if self.volume_history.len() >= 50 {
            let sum: f64 = self.volume_history.iter().rev().take(50).sum();
            safe_divide(sum, 50.0, "50-period MA")?
        } else {
            volume.max(1.0) // Prevent division by zero
        };
        
        let volume_ratio = safe_divide(volume, ma_volume, "volume ratio")?;
        
        // Strike signal calculation with weighted components
        let strike_signal = 0.5 * velocity + 0.3 * acceleration + 0.2 * volume_ratio;
        
        // Determine signal type based on conditions
        let signal_type = if oscillator < -2.0 && velocity > 0.5 && volume_ratio > 1.2 {
            SignalType::StrongLong
        } else if oscillator < -1.5 && velocity > 0.3 && volume_ratio > 1.0 {
            SignalType::Long
        } else if oscillator > 2.0 && velocity < -0.5 && volume_ratio > 1.2 {
            SignalType::StrongShort
        } else if oscillator > 1.5 && velocity < -0.3 && volume_ratio > 1.0 {
            SignalType::Short
        } else {
            SignalType::Neutral
        };

        Ok(OscillatorSignal {
            oscillator_value: oscillator,
            velocity,
            acceleration,
            volume_ratio,
            strike_signal,
            signal_type,
            timestamp: Utc::now(),
        })
    }

    pub fn get_stats(&self) -> OscillatorStats {
        OscillatorStats {
            history_size: self.volume_history.len(),
            update_count: self.update_count,
            last_update: self.last_update,
            memory_usage_percent: (self.volume_history.len() as f64 / self.max_history_size as f64) * 100.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscillatorSignal {
    pub oscillator_value: f64,
    pub velocity: f64,
    pub acceleration: f64,
    pub volume_ratio: f64,
    pub strike_signal: f64,
    pub signal_type: SignalType,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SignalType {
    StrongLong,
    Long,
    Neutral,
    Short,
    StrongShort,
}

#[derive(Debug, Clone)]
pub struct OscillatorStats {
    pub history_size: usize,
    pub update_count: u64,
    pub last_update: DateTime<Utc>,
    pub memory_usage_percent: f64,
}
