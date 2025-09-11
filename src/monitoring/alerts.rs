// Alert Management System
// Handles alert rules, notifications, and alert history

use super::MetricType;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertLevel {
    Info,
    Warning,
    Error,
    Critical,
}

/// Alert record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub level: AlertLevel,
    pub title: String,
    pub message: String,
    pub timestamp: SystemTime,
    pub acknowledged: bool,
}

/// Alert rule for automatic monitoring
#[derive(Debug, Clone)]
pub struct AlertRule {
    pub metric_type: MetricType,
    pub condition: AlertCondition,
    pub level: AlertLevel,
    pub title: String,
    pub message_template: String,
}

/// Alert conditions
#[derive(Debug, Clone)]
pub enum AlertCondition {
    Above(f64),
    Below(f64),
    Equal(f64),
    OutsideRange(f64, f64),
    InsideRange(f64, f64),
    ChangePercent(f64), // Percent change from previous
}

/// Alert manager
pub struct AlertManager {
    alerts: Arc<RwLock<VecDeque<Alert>>>,
    rules: Arc<RwLock<Vec<AlertRule>>>,
    max_alerts: usize,
}

impl AlertManager {
    pub fn new() -> Self {
        let mut manager = Self {
            alerts: Arc::new(RwLock::new(VecDeque::new())),
            rules: Arc::new(RwLock::new(Vec::new())),
            max_alerts: 1000,
        };

        // Initialize default alert rules
        manager.setup_default_rules();
        manager
    }

    /// Setup default monitoring rules
    fn setup_default_rules(&mut self) {
        let default_rules = vec![
            // Trading performance alerts
            AlertRule {
                metric_type: MetricType::WinRate,
                condition: AlertCondition::Below(0.60),
                level: AlertLevel::Warning,
                title: "Low Win Rate".to_string(),
                message_template: "Win rate dropped to {value:.1}%".to_string(),
            },
            AlertRule {
                metric_type: MetricType::WinRate,
                condition: AlertCondition::Below(0.50),
                level: AlertLevel::Critical,
                title: "Critical Win Rate".to_string(),
                message_template: "Win rate critically low at {value:.1}%".to_string(),
            },
            AlertRule {
                metric_type: MetricType::ConsecutiveLosses,
                condition: AlertCondition::Above(5.0),
                level: AlertLevel::Error,
                title: "Consecutive Losses".to_string(),
                message_template: "{value} consecutive losses detected".to_string(),
            },
            AlertRule {
                metric_type: MetricType::DailyPnL,
                condition: AlertCondition::Below(-1000.0),
                level: AlertLevel::Error,
                title: "Daily Loss Limit".to_string(),
                message_template: "Daily P&L at ${value:.2}".to_string(),
            },
            
            // Risk alerts
            AlertRule {
                metric_type: MetricType::DrawDown,
                condition: AlertCondition::Above(0.10),
                level: AlertLevel::Warning,
                title: "High Drawdown".to_string(),
                message_template: "Drawdown at {value:.1}%".to_string(),
            },
            AlertRule {
                metric_type: MetricType::Exposure,
                condition: AlertCondition::Above(50000.0),
                level: AlertLevel::Warning,
                title: "High Exposure".to_string(),
                message_template: "Total exposure at ${value:.2}".to_string(),
            },
            
            // System alerts
            AlertRule {
                metric_type: MetricType::ErrorCount,
                condition: AlertCondition::Above(10.0),
                level: AlertLevel::Error,
                title: "High Error Rate".to_string(),
                message_template: "{value} errors in monitoring period".to_string(),
            },
            AlertRule {
                metric_type: MetricType::Latency,
                condition: AlertCondition::Above(1000.0),
                level: AlertLevel::Warning,
                title: "High Latency".to_string(),
                message_template: "API latency at {value:.0}ms".to_string(),
            },
            AlertRule {
                metric_type: MetricType::APICallCount,
                condition: AlertCondition::Above(55.0), // Per minute
                level: AlertLevel::Warning,
                title: "Rate Limit Warning".to_string(),
                message_template: "API calls at {value}/min (limit: 60)".to_string(),
            },
        ];

        let rules = self.rules.clone();
        tokio::spawn(async move {
            let mut rules_guard = rules.write().await;
            rules_guard.extend(default_rules);
        });
    }

    /// Check metric against rules
    pub async fn check_metric(&self, metric_type: &MetricType, value: f64) {
        let rules = self.rules.read().await;
        
        for rule in rules.iter() {
            if &rule.metric_type != metric_type {
                continue;
            }

            let triggered = match &rule.condition {
                AlertCondition::Above(threshold) => value > *threshold,
                AlertCondition::Below(threshold) => value < *threshold,
                AlertCondition::Equal(target) => (value - target).abs() < f64::EPSILON,
                AlertCondition::OutsideRange(min, max) => value < *min || value > *max,
                AlertCondition::InsideRange(min, max) => value >= *min && value <= *max,
                AlertCondition::ChangePercent(percent) => {
                    // TODO: Implement change detection
                    false
                },
            };

            if triggered {
                let message = rule.message_template.replace("{value}", &format!("{}", value));
                self.send_alert(rule.level, &rule.title, &message).await;
            }
        }
    }

    /// Send an alert
    pub async fn send_alert(&self, level: AlertLevel, title: &str, message: &str) {
        let alert = Alert {
            id: format!("{:?}-{}", SystemTime::now(), title),
            level,
            title: title.to_string(),
            message: message.to_string(),
            timestamp: SystemTime::now(),
            acknowledged: false,
        };

        // Log the alert
        match level {
            AlertLevel::Info => log::info!("ALERT: {} - {}", title, message),
            AlertLevel::Warning => log::warn!("ALERT: {} - {}", title, message),
            AlertLevel::Error => log::error!("ALERT: {} - {}", title, message),
            AlertLevel::Critical => log::error!("CRITICAL ALERT: {} - {}", title, message),
        }

        // Store alert
        let mut alerts = self.alerts.write().await;
        alerts.push_back(alert.clone());
        
        // Maintain max size
        while alerts.len() > self.max_alerts {
            alerts.pop_front();
        }

        // In production, you would send notifications here:
        // - Email
        // - SMS
        // - Slack/Discord
        // - PagerDuty
        // - Webhook
    }

    /// Get recent alerts
    pub async fn get_alerts(&self, limit: usize) -> Vec<Alert> {
        let alerts = self.alerts.read().await;
        alerts.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get unacknowledged alerts
    pub async fn get_unacknowledged_alerts(&self) -> Vec<Alert> {
        let alerts = self.alerts.read().await;
        alerts.iter()
            .filter(|a| !a.acknowledged)
            .cloned()
            .collect()
    }

    /// Acknowledge an alert
    pub async fn acknowledge_alert(&self, alert_id: &str) {
        let mut alerts = self.alerts.write().await;
        for alert in alerts.iter_mut() {
            if alert.id == alert_id {
                alert.acknowledged = true;
                break;
            }
        }
    }

    /// Clear all alerts
    pub async fn clear_alerts(&self) {
        let mut alerts = self.alerts.write().await;
        alerts.clear();
    }

    /// Add custom alert rule
    pub async fn add_rule(&self, rule: AlertRule) {
        let mut rules = self.rules.write().await;
        rules.push(rule);
    }

    /// Remove alert rule
    pub async fn remove_rule(&self, metric_type: &MetricType) {
        let mut rules = self.rules.write().await;
        rules.retain(|r| &r.metric_type != metric_type);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_alert_manager() {
        let manager = AlertManager::new();
        
        // Send test alert
        manager.send_alert(
            AlertLevel::Warning,
            "Test Alert",
            "This is a test alert message",
        ).await;

        // Check alerts
        let alerts = manager.get_alerts(10).await;
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].title, "Test Alert");
        assert!(!alerts[0].acknowledged);

        // Acknowledge alert
        let alert_id = alerts[0].id.clone();
        manager.acknowledge_alert(&alert_id).await;

        // Verify acknowledged
        let unack_alerts = manager.get_unacknowledged_alerts().await;
        assert_eq!(unack_alerts.len(), 0);
    }
}
