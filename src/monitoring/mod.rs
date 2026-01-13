// Real-time Monitoring Module
// Provides metrics, alerting, and system health monitoring

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

pub mod alerts;
pub mod metrics;
pub mod health;

/// System metric types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricType {
    // Trading metrics
    TradeCount,
    WinRate,
    TotalPnL,
    DailyPnL,
    AverageWin,
    AverageLoss,
    ConsecutiveWins,
    ConsecutiveLosses,
    
    // System metrics
    Latency,
    MemoryUsage,
    CPUUsage,
    APICallCount,
    ErrorCount,
    
    // Risk metrics
    Exposure,
    DrawDown,
    
    // Strike metrics
    StrikeOptimized,
    SharpeRatio,
    MaxDrawDown,
}

/// Metric value with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    pub value: f64,
    pub timestamp: SystemTime,
}

/// Time series data for metrics
#[derive(Debug, Clone)]
pub struct TimeSeries {
    pub metric_type: MetricType,
    pub values: Vec<MetricValue>,
    pub max_size: usize,
}

impl TimeSeries {
    pub fn new(metric_type: MetricType, max_size: usize) -> Self {
        Self {
            metric_type,
            values: Vec::with_capacity(max_size),
            max_size,
        }
    }

    pub fn push(&mut self, value: f64) {
        let metric_value = MetricValue {
            value,
            timestamp: SystemTime::now(),
        };
        
        self.values.push(metric_value);
        
        // Keep only the most recent values
        if self.values.len() > self.max_size {
            self.values.remove(0);
        }
    }

    pub fn latest(&self) -> Option<f64> {
        self.values.last().map(|v| v.value)
    }

    pub fn average(&self) -> Option<f64> {
        if self.values.is_empty() {
            None
        } else {
            let sum: f64 = self.values.iter().map(|v| v.value).sum();
            Some(sum / self.values.len() as f64)
        }
    }

    pub fn min(&self) -> Option<f64> {
        self.values.iter().map(|v| v.value).fold(None, |min, val| {
            match min {
                None => Some(val),
                Some(m) => Some(m.min(val)),
            }
        })
    }

    pub fn max(&self) -> Option<f64> {
        self.values.iter().map(|v| v.value).fold(None, |max, val| {
            match max {
                None => Some(val),
                Some(m) => Some(m.max(val)),
            }
        })
    }
}

/// Main monitoring system
pub struct MonitoringSystem {
    metrics: Arc<RwLock<HashMap<MetricType, TimeSeries>>>,
    alert_manager: Arc<alerts::AlertManager>,
    health_monitor: Arc<health::HealthMonitor>,
}

impl MonitoringSystem {
    pub fn new() -> Self {
        let mut metrics = HashMap::new();
        
        // Initialize all metric types
        for metric_type in [
            MetricType::TradeCount,
            MetricType::WinRate,
            MetricType::TotalPnL,
            MetricType::DailyPnL,
            MetricType::AverageWin,
            MetricType::AverageLoss,
            MetricType::ConsecutiveWins,
            MetricType::ConsecutiveLosses,
            MetricType::Latency,
            MetricType::MemoryUsage,
            MetricType::CPUUsage,
            MetricType::APICallCount,
            MetricType::ErrorCount,
            MetricType::Exposure,
            MetricType::DrawDown,
            MetricType::SharpeRatio,
            MetricType::MaxDrawDown,
        ] {
            metrics.insert(metric_type.clone(), TimeSeries::new(metric_type, 1000));
        }

        Self {
            metrics: Arc::new(RwLock::new(metrics)),
            alert_manager: Arc::new(alerts::AlertManager::new()),
            health_monitor: Arc::new(health::HealthMonitor::new()),
        }
    }

    /// Record a metric value
    pub async fn record_metric(&self, metric_type: MetricType, value: f64) {
        let mut metrics = self.metrics.write().await;
        if let Some(time_series) = metrics.get_mut(&metric_type) {
            time_series.push(value);
        }

        // Check for alerts
        self.alert_manager.check_metric(&metric_type, value).await;
    }

    /// Get current metric value
    pub async fn get_metric(&self, metric_type: &MetricType) -> Option<f64> {
        let metrics = self.metrics.read().await;
        metrics.get(metric_type).and_then(|ts| ts.latest())
    }

    /// Get metric statistics
    pub async fn get_metric_stats(&self, metric_type: &MetricType) -> Option<MetricStats> {
        let metrics = self.metrics.read().await;
        metrics.get(metric_type).map(|ts| MetricStats {
            latest: ts.latest().unwrap_or(0.0),
            average: ts.average().unwrap_or(0.0),
            min: ts.min().unwrap_or(0.0),
            max: ts.max().unwrap_or(0.0),
            count: ts.values.len(),
        })
    }

    /// Get system health status
    pub async fn get_health_status(&self) -> health::HealthStatus {
        self.health_monitor.get_status(&self.metrics).await
    }

    /// Start monitoring background tasks
    pub async fn start(&self) {
        let metrics = self.metrics.clone();
        let health_monitor = self.health_monitor.clone();
        let alert_manager = self.alert_manager.clone();

        // Health check loop
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));
            loop {
                interval.tick().await;
                
                // Perform health checks
                let status = health_monitor.get_status(&metrics).await;
                
                // Alert on critical issues
                if let health::HealthLevel::Critical = status.level {
                    alert_manager.send_alert(
                        alerts::AlertLevel::Critical,
                        "System Health Critical",
                        &status.message,
                    ).await;
                }
            }
        });
    }

    /// Export metrics snapshot
    pub async fn export_snapshot(&self) -> MetricsSnapshot {
        let metrics = self.metrics.read().await;
        let mut snapshot = HashMap::new();

        for (metric_type, time_series) in metrics.iter() {
            if let Some(stats) = self.get_metric_stats(metric_type).await {
                snapshot.insert(metric_type.clone(), stats);
            }
        }

        MetricsSnapshot {
            timestamp: SystemTime::now(),
            metrics: snapshot,
            health: self.get_health_status().await,
        }
    }
}

/// Metric statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricStats {
    pub latest: f64,
    pub average: f64,
    pub min: f64,
    pub max: f64,
    pub count: usize,
}

/// Complete metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub timestamp: SystemTime,
    pub metrics: HashMap<MetricType, MetricStats>,
    pub health: health::HealthStatus,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_time_series() {
        let mut ts = TimeSeries::new(MetricType::WinRate, 5);
        
        ts.push(0.75);
        ts.push(0.80);
        ts.push(0.85);
        
        assert_eq!(ts.latest(), Some(0.85));
        assert_eq!(ts.average(), Some(0.80));
        assert_eq!(ts.min(), Some(0.75));
        assert_eq!(ts.max(), Some(0.85));
    }

    #[tokio::test]
    async fn test_monitoring_system() {
        let monitor = MonitoringSystem::new();
        
        monitor.record_metric(MetricType::WinRate, 0.75).await;
        monitor.record_metric(MetricType::WinRate, 0.80).await;
        
        let value = monitor.get_metric(&MetricType::WinRate).await;
        assert_eq!(value, Some(0.80));
    }
}
