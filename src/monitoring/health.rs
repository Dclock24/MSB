// Health Monitoring Module
// Monitors system health and provides health status

use super::{MetricType, TimeSeries};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Health status levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthLevel {
    Healthy,
    Degraded,
    Unhealthy,
    Critical,
}

/// System health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub level: HealthLevel,
    pub score: f64, // 0-100
    pub message: String,
    pub components: Vec<ComponentHealth>,
    pub issues: Vec<HealthIssue>,
}

/// Component health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthLevel,
    pub details: String,
}

/// Health issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthIssue {
    pub severity: HealthLevel,
    pub component: String,
    pub description: String,
    pub recommendation: String,
}

/// Health monitor
pub struct HealthMonitor {
    thresholds: HealthThresholds,
}

#[derive(Debug, Clone)]
struct HealthThresholds {
    win_rate_critical: f64,
    win_rate_warning: f64,
    error_rate_warning: u32,
    error_rate_critical: u32,
    latency_warning: f64,
    latency_critical: f64,
    memory_warning: f64,
    memory_critical: f64,
}

impl Default for HealthThresholds {
    fn default() -> Self {
        Self {
            win_rate_critical: 0.50,
            win_rate_warning: 0.65,
            error_rate_warning: 10,
            error_rate_critical: 50,
            latency_warning: 500.0,
            latency_critical: 1000.0,
            memory_warning: 80.0,
            memory_critical: 90.0,
        }
    }
}

impl HealthMonitor {
    pub fn new() -> Self {
        Self {
            thresholds: HealthThresholds::default(),
        }
    }

    /// Get current health status
    pub async fn get_status(
        &self,
        metrics: &Arc<RwLock<HashMap<MetricType, TimeSeries>>>,
    ) -> HealthStatus {
        let metrics_guard = metrics.read().await;
        let mut components = Vec::new();
        let mut issues = Vec::new();
        let mut total_score: f64 = 100.0;

        // Check trading performance
        if let Some(component) = self.check_trading_health(&metrics_guard, &mut issues) {
            if component.status != HealthLevel::Healthy {
                total_score -= match component.status {
                    HealthLevel::Degraded => 10.0,
                    HealthLevel::Unhealthy => 25.0,
                    HealthLevel::Critical => 50.0,
                    _ => 0.0,
                };
            }
            components.push(component);
        }

        // Check system performance
        if let Some(component) = self.check_system_health(&metrics_guard, &mut issues) {
            if component.status != HealthLevel::Healthy {
                total_score -= match component.status {
                    HealthLevel::Degraded => 5.0,
                    HealthLevel::Unhealthy => 15.0,
                    HealthLevel::Critical => 30.0,
                    _ => 0.0,
                };
            }
            components.push(component);
        }

        // Check risk metrics
        if let Some(component) = self.check_risk_health(&metrics_guard, &mut issues) {
            if component.status != HealthLevel::Healthy {
                total_score -= match component.status {
                    HealthLevel::Degraded => 10.0,
                    HealthLevel::Unhealthy => 20.0,
                    HealthLevel::Critical => 40.0,
                    _ => 0.0,
                };
            }
            components.push(component);
        }

        // Determine overall health level
        let level = if total_score >= 90.0 {
            HealthLevel::Healthy
        } else if total_score >= 70.0 {
            HealthLevel::Degraded
        } else if total_score >= 50.0 {
            HealthLevel::Unhealthy
        } else {
            HealthLevel::Critical
        };

        let message = match level {
            HealthLevel::Healthy => "All systems operating normally".to_string(),
            HealthLevel::Degraded => "System performance degraded".to_string(),
            HealthLevel::Unhealthy => "Multiple issues detected".to_string(),
            HealthLevel::Critical => "Critical issues require immediate attention".to_string(),
        };

        HealthStatus {
            level,
            score: total_score.max(0.0),
            message,
            components,
            issues,
        }
    }

    /// Check trading health
    fn check_trading_health(
        &self,
        metrics: &HashMap<MetricType, TimeSeries>,
        issues: &mut Vec<HealthIssue>,
    ) -> Option<ComponentHealth> {
        let mut status = HealthLevel::Healthy;
        let mut details = Vec::new();

        // Check win rate
        if let Some(win_rate_ts) = metrics.get(&MetricType::WinRate) {
            if let Some(win_rate) = win_rate_ts.latest() {
                if win_rate < self.thresholds.win_rate_critical {
                    status = HealthLevel::Critical;
                    issues.push(HealthIssue {
                        severity: HealthLevel::Critical,
                        component: "Trading".to_string(),
                        description: format!("Win rate critically low at {:.1}%", win_rate * 100.0),
                        recommendation: "Stop trading and review strategy".to_string(),
                    });
                } else if win_rate < self.thresholds.win_rate_warning {
                    status = status.max(HealthLevel::Unhealthy);
                    issues.push(HealthIssue {
                        severity: HealthLevel::Unhealthy,
                        component: "Trading".to_string(),
                        description: format!("Win rate below target at {:.1}%", win_rate * 100.0),
                        recommendation: "Reduce position sizes and monitor closely".to_string(),
                    });
                }
                details.push(format!("Win rate: {:.1}%", win_rate * 100.0));
            }
        }

        // Check consecutive losses
        if let Some(losses_ts) = metrics.get(&MetricType::ConsecutiveLosses) {
            if let Some(losses) = losses_ts.latest() {
                if losses > 5.0 {
                    status = status.max(HealthLevel::Unhealthy);
                    issues.push(HealthIssue {
                        severity: HealthLevel::Unhealthy,
                        component: "Trading".to_string(),
                        description: format!("{} consecutive losses", losses as u32),
                        recommendation: "Consider pausing trading to break the streak".to_string(),
                    });
                }
                details.push(format!("Consecutive losses: {}", losses as u32));
            }
        }

        Some(ComponentHealth {
            name: "Trading Performance".to_string(),
            status,
            details: details.join(", "),
        })
    }

    /// Check system health
    fn check_system_health(
        &self,
        metrics: &HashMap<MetricType, TimeSeries>,
        issues: &mut Vec<HealthIssue>,
    ) -> Option<ComponentHealth> {
        let mut status = HealthLevel::Healthy;
        let mut details = Vec::new();

        // Check error rate
        if let Some(error_ts) = metrics.get(&MetricType::ErrorCount) {
            if let Some(errors) = error_ts.latest() {
                if errors > self.thresholds.error_rate_critical as f64 {
                    status = HealthLevel::Critical;
                    issues.push(HealthIssue {
                        severity: HealthLevel::Critical,
                        component: "System".to_string(),
                        description: format!("High error rate: {} errors", errors as u32),
                        recommendation: "Check logs and fix critical errors".to_string(),
                    });
                } else if errors > self.thresholds.error_rate_warning as f64 {
                    status = status.max(HealthLevel::Degraded);
                    issues.push(HealthIssue {
                        severity: HealthLevel::Degraded,
                        component: "System".to_string(),
                        description: format!("Elevated error rate: {} errors", errors as u32),
                        recommendation: "Monitor error logs".to_string(),
                    });
                }
                details.push(format!("Errors: {}", errors as u32));
            }
        }

        // Check latency
        if let Some(latency_ts) = metrics.get(&MetricType::Latency) {
            if let Some(latency) = latency_ts.latest() {
                if latency > self.thresholds.latency_critical {
                    status = status.max(HealthLevel::Unhealthy);
                    issues.push(HealthIssue {
                        severity: HealthLevel::Unhealthy,
                        component: "System".to_string(),
                        description: format!("High API latency: {:.0}ms", latency),
                        recommendation: "Check network and API health".to_string(),
                    });
                } else if latency > self.thresholds.latency_warning {
                    status = status.max(HealthLevel::Degraded);
                }
                details.push(format!("Latency: {:.0}ms", latency));
            }
        }

        Some(ComponentHealth {
            name: "System Performance".to_string(),
            status,
            details: details.join(", "),
        })
    }

    /// Check risk health
    fn check_risk_health(
        &self,
        metrics: &HashMap<MetricType, TimeSeries>,
        issues: &mut Vec<HealthIssue>,
    ) -> Option<ComponentHealth> {
        let mut status = HealthLevel::Healthy;
        let mut details = Vec::new();

        // Check drawdown
        if let Some(dd_ts) = metrics.get(&MetricType::DrawDown) {
            if let Some(drawdown) = dd_ts.latest() {
                if drawdown > 0.20 {
                    status = HealthLevel::Critical;
                    issues.push(HealthIssue {
                        severity: HealthLevel::Critical,
                        component: "Risk".to_string(),
                        description: format!("Severe drawdown: {:.1}%", drawdown * 100.0),
                        recommendation: "Reduce all positions immediately".to_string(),
                    });
                } else if drawdown > 0.10 {
                    status = status.max(HealthLevel::Unhealthy);
                    issues.push(HealthIssue {
                        severity: HealthLevel::Unhealthy,
                        component: "Risk".to_string(),
                        description: format!("High drawdown: {:.1}%", drawdown * 100.0),
                        recommendation: "Reduce risk exposure".to_string(),
                    });
                }
                details.push(format!("Drawdown: {:.1}%", drawdown * 100.0));
            }
        }

        // Check exposure
        if let Some(exposure_ts) = metrics.get(&MetricType::Exposure) {
            if let Some(exposure) = exposure_ts.latest() {
                if exposure > 100_000.0 {
                    status = status.max(HealthLevel::Degraded);
                    details.push(format!("Exposure: ${:.0}", exposure));
                }
            }
        }

        Some(ComponentHealth {
            name: "Risk Management".to_string(),
            status,
            details: details.join(", "),
        })
    }
}

impl HealthLevel {
    fn max(self, other: Self) -> Self {
        match (self as u8, other as u8) {
            (a, b) if a > b => self,
            _ => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_monitor() {
        let monitor = HealthMonitor::new();
        let metrics = Arc::new(RwLock::new(HashMap::new()));

        // Add test metrics
        {
            let mut metrics_guard = metrics.write().await;
            
            let mut win_rate_ts = TimeSeries::new(MetricType::WinRate, 100);
            win_rate_ts.push(0.75);
            metrics_guard.insert(MetricType::WinRate, win_rate_ts);

            let mut error_ts = TimeSeries::new(MetricType::ErrorCount, 100);
            error_ts.push(5.0);
            metrics_guard.insert(MetricType::ErrorCount, error_ts);
        }

        let status = monitor.get_status(&metrics).await;
        assert_eq!(status.level, HealthLevel::Healthy);
        assert!(status.score > 80.0);
    }
}
