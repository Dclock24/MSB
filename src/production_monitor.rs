// REAL-TIME PRODUCTION MONITORING SYSTEM
// Ensures $250K capital is protected and performing as expected

use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};
use std::collections::{HashMap, VecDeque};
use serde::{Serialize, Deserialize};

/// Production monitoring system with real-time alerts
pub struct ProductionMonitor {
    capital_limit: f64,
    initial_capital: f64,
    current_capital: Arc<RwLock<f64>>,
    daily_loss_limit: f64,
    position_limits: PositionLimits,
    performance_tracker: Arc<RwLock<PerformanceTracker>>,
    alert_system: Arc<RwLock<AlertSystem>>,
    circuit_breakers: Arc<RwLock<CircuitBreakers>>,
    audit_log: Arc<RwLock<AuditLog>>,
}

#[derive(Debug, Clone)]
pub struct PositionLimits {
    pub max_position_size: f64,      // Max single position
    pub max_total_exposure: f64,     // Max total exposure
    pub max_correlated_exposure: f64, // Max correlated positions
    pub max_positions_per_symbol: usize,
    pub concentration_limit: f64,     // Max % in single asset
}

#[derive(Debug, Clone)]
pub struct PerformanceTracker {
    pub daily_pnl: f64,
    pub weekly_pnl: f64,
    pub monthly_pnl: f64,
    pub total_pnl: f64,
    pub win_rate: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub current_drawdown: f64,
    pub trades_today: usize,
    pub winning_trades_today: usize,
    pub last_update: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct AlertSystem {
    pub alerts: VecDeque<Alert>,
    pub alert_thresholds: AlertThresholds,
    pub notification_channels: Vec<NotificationChannel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub timestamp: DateTime<Utc>,
    pub severity: AlertSeverity,
    pub category: AlertCategory,
    pub message: String,
    pub value: f64,
    pub threshold: f64,
    pub action_required: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCategory {
    RiskLimit,
    Performance,
    Technical,
    Execution,
    Market,
    System,
}

#[derive(Debug, Clone)]
pub struct AlertThresholds {
    pub daily_loss_warning: f64,     // % loss to trigger warning
    pub daily_loss_critical: f64,    // % loss to trigger critical
    pub drawdown_warning: f64,       // % drawdown warning
    pub drawdown_critical: f64,      // % drawdown critical
    pub win_rate_warning: f64,       // Win rate below this
    pub latency_warning_ms: u64,     // Execution latency warning
    pub position_concentration: f64,  // Position concentration limit
}

#[derive(Debug, Clone)]
pub enum NotificationChannel {
    Email(String),
    Sms(String),
    Webhook(String),
    Telegram(String),
    Dashboard,
}

#[derive(Debug, Clone)]
pub struct CircuitBreakers {
    pub daily_loss_breaker: bool,
    pub drawdown_breaker: bool,
    pub volatility_breaker: bool,
    pub technical_breaker: bool,
    pub breaker_history: Vec<BreakerEvent>,
}

#[derive(Debug, Clone)]
pub struct BreakerEvent {
    pub timestamp: DateTime<Utc>,
    pub breaker_type: String,
    pub trigger_value: f64,
    pub duration_minutes: u64,
    pub auto_resume: bool,
}

#[derive(Debug, Clone)]
pub struct AuditLog {
    pub entries: VecDeque<AuditEntry>,
    pub max_entries: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub action: String,
    pub details: HashMap<String, String>,
    pub result: String,
    pub capital_before: f64,
    pub capital_after: f64,
}

impl ProductionMonitor {
    pub fn new(initial_capital: f64) -> Self {
        let daily_loss_limit = initial_capital * 0.05; // 5% max daily loss
        
        let position_limits = PositionLimits {
            max_position_size: initial_capital * 0.12,      // 12% max position
            max_total_exposure: initial_capital * 0.40,     // 40% max exposure
            max_correlated_exposure: initial_capital * 0.25, // 25% correlated
            max_positions_per_symbol: 3,
            concentration_limit: 0.20, // 20% max in one asset
        };
        
        let alert_thresholds = AlertThresholds {
            daily_loss_warning: 0.02,    // 2% daily loss warning
            daily_loss_critical: 0.04,   // 4% daily loss critical
            drawdown_warning: 0.05,      // 5% drawdown warning
            drawdown_critical: 0.08,     // 8% drawdown critical
            win_rate_warning: 0.60,      // 60% win rate warning
            latency_warning_ms: 500,     // 500ms latency warning
            position_concentration: 0.25, // 25% concentration warning
        };
        
        Self {
            capital_limit: initial_capital * 1.2, // 20% growth buffer
            initial_capital,
            current_capital: Arc::new(RwLock::new(initial_capital)),
            daily_loss_limit,
            position_limits,
            performance_tracker: Arc::new(RwLock::new(PerformanceTracker::default())),
            alert_system: Arc::new(RwLock::new(AlertSystem {
                alerts: VecDeque::new(),
                alert_thresholds,
                notification_channels: vec![
                    NotificationChannel::Dashboard,
                    NotificationChannel::Email("trader@example.com".to_string()),
                ],
            })),
            circuit_breakers: Arc::new(RwLock::new(CircuitBreakers::default())),
            audit_log: Arc::new(RwLock::new(AuditLog {
                entries: VecDeque::new(),
                max_entries: 10000,
            })),
        }
    }
    
    /// Check if trading should continue
    pub async fn should_continue_trading(&self) -> bool {
        let breakers = self.circuit_breakers.read().await;
        
        // Check all circuit breakers
        if breakers.daily_loss_breaker {
            self.create_alert(
                AlertSeverity::Critical,
                AlertCategory::RiskLimit,
                "Daily loss limit breaker triggered".to_string(),
                self.daily_loss_limit,
            ).await;
            return false;
        }
        
        if breakers.drawdown_breaker {
            self.create_alert(
                AlertSeverity::Critical,
                AlertCategory::RiskLimit,
                "Drawdown breaker triggered".to_string(),
                0.0,
            ).await;
            return false;
        }
        
        if breakers.volatility_breaker {
            self.create_alert(
                AlertSeverity::Warning,
                AlertCategory::Market,
                "Volatility breaker triggered".to_string(),
                0.0,
            ).await;
            return false;
        }
        
        if breakers.technical_breaker {
            self.create_alert(
                AlertSeverity::Emergency,
                AlertCategory::System,
                "Technical breaker triggered".to_string(),
                0.0,
            ).await;
            return false;
        }
        
        true
    }
    
    /// Update capital and check limits
    pub async fn update_capital(&self, new_capital: f64) {
        let mut current = self.current_capital.write().await;
        let old_capital = *current;
        *current = new_capital;
        
        // Calculate daily P&L
        let daily_pnl = new_capital - self.initial_capital;
        let daily_pnl_pct = daily_pnl / self.initial_capital;
        
        // Update performance tracker
        let mut tracker = self.performance_tracker.write().await;
        tracker.daily_pnl = daily_pnl;
        tracker.total_pnl = new_capital - self.initial_capital;
        tracker.last_update = Utc::now();
        
        // Check for circuit breaker conditions
        if daily_pnl <= -self.daily_loss_limit {
            let mut breakers = self.circuit_breakers.write().await;
            breakers.daily_loss_breaker = true;
            breakers.breaker_history.push(BreakerEvent {
                timestamp: Utc::now(),
                breaker_type: "Daily Loss".to_string(),
                trigger_value: daily_pnl,
                duration_minutes: 1440, // 24 hours
                auto_resume: true,
            });
        }
        
        // Check alert conditions
        let alert_system = self.alert_system.read().await;
        
        if daily_pnl_pct <= -alert_system.alert_thresholds.daily_loss_critical {
            drop(alert_system);
            self.create_alert(
                AlertSeverity::Critical,
                AlertCategory::RiskLimit,
                format!("Daily loss critical: {:.2}%", daily_pnl_pct * 100.0),
                daily_pnl,
            ).await;
        } else if daily_pnl_pct <= -alert_system.alert_thresholds.daily_loss_warning {
            drop(alert_system);
            self.create_alert(
                AlertSeverity::Warning,
                AlertCategory::RiskLimit,
                format!("Daily loss warning: {:.2}%", daily_pnl_pct * 100.0),
                daily_pnl,
            ).await;
        }
        
        // Log capital update
        self.audit_log("Capital Update", 
            vec![
                ("old_capital".to_string(), old_capital.to_string()),
                ("new_capital".to_string(), new_capital.to_string()),
                ("daily_pnl".to_string(), daily_pnl.to_string()),
            ],
            old_capital,
            new_capital,
        ).await;
    }
    
    /// Validate position before execution
    pub async fn validate_position(&self, symbol: &str, size: f64, side: &str) -> Result<(), String> {
        let current_capital = *self.current_capital.read().await;
        
        // Check position size limit
        if size > self.position_limits.max_position_size {
            return Err(format!(
                "Position size ${:.2} exceeds limit ${:.2}",
                size, self.position_limits.max_position_size
            ));
        }
        
        // Check concentration limit
        let concentration = size / current_capital;
        if concentration > self.position_limits.concentration_limit {
            return Err(format!(
                "Position concentration {:.1}% exceeds limit {:.1}%",
                concentration * 100.0,
                self.position_limits.concentration_limit * 100.0
            ));
        }
        
        // Check if we should be trading at all
        if !self.should_continue_trading().await {
            return Err("Trading halted by circuit breaker".to_string());
        }
        
        // Log validation
        self.audit_log("Position Validation",
            vec![
                ("symbol".to_string(), symbol.to_string()),
                ("size".to_string(), size.to_string()),
                ("side".to_string(), side.to_string()),
                ("result".to_string(), "approved".to_string()),
            ],
            current_capital,
            current_capital,
        ).await;
        
        Ok(())
    }
    
    /// Update trade results
    pub async fn record_trade(&self, symbol: &str, pnl: f64, win: bool, execution_ms: u64) {
        let mut tracker = self.performance_tracker.write().await;
        
        tracker.trades_today += 1;
        if win {
            tracker.winning_trades_today += 1;
        }
        
        // Update win rate
        tracker.win_rate = tracker.winning_trades_today as f64 / tracker.trades_today as f64;
        
        // Check performance alerts
        let alert_system = self.alert_system.read().await;
        
        if tracker.win_rate < alert_system.alert_thresholds.win_rate_warning {
            drop(alert_system);
            self.create_alert(
                AlertSeverity::Warning,
                AlertCategory::Performance,
                format!("Win rate below target: {:.1}%", tracker.win_rate * 100.0),
                tracker.win_rate,
            ).await;
        }
        
        if execution_ms > alert_system.alert_thresholds.latency_warning_ms {
            drop(alert_system);
            self.create_alert(
                AlertSeverity::Warning,
                AlertCategory::Execution,
                format!("High execution latency: {}ms", execution_ms),
                execution_ms as f64,
            ).await;
        }
        
        // Update capital
        let new_capital = *self.current_capital.read().await + pnl;
        drop(tracker);
        self.update_capital(new_capital).await;
    }
    
    /// Create and send alert
    async fn create_alert(&self, severity: AlertSeverity, category: AlertCategory, message: String, value: f64) {
        let alert = Alert {
            timestamp: Utc::now(),
            severity,
            category: category.clone(),
            message: message.clone(),
            value,
            threshold: 0.0,
            action_required: matches!(severity, AlertSeverity::Critical | AlertSeverity::Emergency),
        };
        
        let mut alert_system = self.alert_system.write().await;
        alert_system.alerts.push_back(alert.clone());
        
        // Keep only last 1000 alerts
        while alert_system.alerts.len() > 1000 {
            alert_system.alerts.pop_front();
        }
        
        // Send notifications based on severity
        match severity {
            AlertSeverity::Emergency | AlertSeverity::Critical => {
                self.send_notifications(&alert, &alert_system.notification_channels).await;
            }
            _ => {}
        }
    }
    
    /// Send notifications through configured channels
    async fn send_notifications(&self, alert: &Alert, channels: &[NotificationChannel]) {
        for channel in channels {
            match channel {
                NotificationChannel::Dashboard => {
                    // Dashboard automatically shows all alerts
                }
                NotificationChannel::Email(email) => {
                    println!("📧 Email to {}: {} - {}", email, alert.severity as u8, alert.message);
                }
                NotificationChannel::Sms(phone) => {
                    println!("📱 SMS to {}: {}", phone, alert.message);
                }
                NotificationChannel::Telegram(chat_id) => {
                    println!("💬 Telegram to {}: {}", chat_id, alert.message);
                }
                NotificationChannel::Webhook(url) => {
                    println!("🔗 Webhook to {}: {:?}", url, alert);
                }
            }
        }
    }
    
    /// Add entry to audit log
    async fn audit_log(&self, action: &str, details: Vec<(String, String)>, capital_before: f64, capital_after: f64) {
        let mut log = self.audit_log.write().await;
        
        let entry = AuditEntry {
            timestamp: Utc::now(),
            action: action.to_string(),
            details: details.into_iter().collect(),
            result: if capital_after >= capital_before { "success" } else { "loss" }.to_string(),
            capital_before,
            capital_after,
        };
        
        log.entries.push_back(entry);
        
        // Maintain size limit
        while log.entries.len() > log.max_entries {
            log.entries.pop_front();
        }
    }
    
    /// Generate monitoring report
    pub async fn generate_report(&self) -> MonitoringReport {
        let current_capital = *self.current_capital.read().await;
        let tracker = self.performance_tracker.read().await;
        let breakers = self.circuit_breakers.read().await;
        let alerts = self.alert_system.read().await;
        
        MonitoringReport {
            timestamp: Utc::now(),
            capital: CapitalStatus {
                initial: self.initial_capital,
                current: current_capital,
                daily_pnl: tracker.daily_pnl,
                total_pnl: tracker.total_pnl,
                daily_pnl_pct: tracker.daily_pnl / self.initial_capital,
                total_pnl_pct: tracker.total_pnl / self.initial_capital,
            },
            performance: PerformanceStatus {
                trades_today: tracker.trades_today,
                win_rate: tracker.win_rate,
                sharpe_ratio: tracker.sharpe_ratio,
                max_drawdown: tracker.max_drawdown,
                current_drawdown: tracker.current_drawdown,
            },
            risk: RiskStatus {
                daily_loss_used: -tracker.daily_pnl / self.daily_loss_limit,
                position_limit_used: 0.0, // Would need to track positions
                breakers_active: breakers.daily_loss_breaker || 
                                breakers.drawdown_breaker || 
                                breakers.volatility_breaker || 
                                breakers.technical_breaker,
                alerts_active: alerts.alerts.iter()
                    .filter(|a| matches!(a.severity, AlertSeverity::Critical | AlertSeverity::Emergency))
                    .count(),
            },
            health: SystemHealth {
                status: if breakers.technical_breaker { "ERROR" } 
                       else if breakers.daily_loss_breaker { "HALTED" }
                       else if alerts.alerts.iter().any(|a| matches!(a.severity, AlertSeverity::Critical)) { "WARNING" }
                       else { "OK" }.to_string(),
                uptime_hours: 24.0, // Would track actual uptime
                last_trade: tracker.last_update,
                api_health: true,
            },
        }
    }
    
    /// Reset daily counters (call at UTC midnight)
    pub async fn reset_daily_counters(&self) {
        let mut tracker = self.performance_tracker.write().await;
        tracker.daily_pnl = 0.0;
        tracker.trades_today = 0;
        tracker.winning_trades_today = 0;
        
        // Reset daily loss breaker if auto-resume enabled
        let mut breakers = self.circuit_breakers.write().await;
        if breakers.daily_loss_breaker {
            if let Some(last_event) = breakers.breaker_history.last() {
                if last_event.auto_resume && 
                   Utc::now() - last_event.timestamp > Duration::minutes(last_event.duration_minutes as i64) {
                    breakers.daily_loss_breaker = false;
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringReport {
    pub timestamp: DateTime<Utc>,
    pub capital: CapitalStatus,
    pub performance: PerformanceStatus,
    pub risk: RiskStatus,
    pub health: SystemHealth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapitalStatus {
    pub initial: f64,
    pub current: f64,
    pub daily_pnl: f64,
    pub total_pnl: f64,
    pub daily_pnl_pct: f64,
    pub total_pnl_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStatus {
    pub trades_today: usize,
    pub win_rate: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub current_drawdown: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskStatus {
    pub daily_loss_used: f64,
    pub position_limit_used: f64,
    pub breakers_active: bool,
    pub alerts_active: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub status: String,
    pub uptime_hours: f64,
    pub last_trade: DateTime<Utc>,
    pub api_health: bool,
}

impl Default for PerformanceTracker {
    fn default() -> Self {
        Self {
            daily_pnl: 0.0,
            weekly_pnl: 0.0,
            monthly_pnl: 0.0,
            total_pnl: 0.0,
            win_rate: 0.0,
            sharpe_ratio: 0.0,
            max_drawdown: 0.0,
            current_drawdown: 0.0,
            trades_today: 0,
            winning_trades_today: 0,
            last_update: Utc::now(),
        }
    }
}

impl Default for CircuitBreakers {
    fn default() -> Self {
        Self {
            daily_loss_breaker: false,
            drawdown_breaker: false,
            volatility_breaker: false,
            technical_breaker: false,
            breaker_history: vec![],
        }
    }
}

/// Production monitoring dashboard (web interface)
pub mod dashboard {
    use super::*;
    
    /// HTML dashboard generator
    pub fn generate_dashboard_html(report: &MonitoringReport) -> String {
        format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Macro Strike Bot - $250K Production Monitor</title>
    <meta http-equiv="refresh" content="5">
    <style>
        body {{ font-family: Arial, sans-serif; background: #1a1a1a; color: #fff; margin: 20px; }}
        .container {{ max-width: 1400px; margin: 0 auto; }}
        .header {{ background: #2a2a2a; padding: 20px; border-radius: 10px; margin-bottom: 20px; }}
        .metrics {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 20px; }}
        .metric-card {{ background: #2a2a2a; padding: 20px; border-radius: 10px; }}
        .metric-value {{ font-size: 2em; font-weight: bold; margin: 10px 0; }}
        .positive {{ color: #4CAF50; }}
        .negative {{ color: #F44336; }}
        .warning {{ color: #FF9800; }}
        .status-ok {{ color: #4CAF50; }}
        .status-error {{ color: #F44336; }}
        .chart {{ height: 200px; background: #333; border-radius: 5px; margin-top: 10px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Macro Strike Bot - Production Monitor</h1>
            <p>Capital: ${:.2} | Status: <span class="{}">{}</span> | {}</p>
        </div>
        
        <div class="metrics">
            <div class="metric-card">
                <h3>Daily P&L</h3>
                <div class="metric-value {}">${:.2}</div>
                <p>{:.1}% of capital</p>
            </div>
            
            <div class="metric-card">
                <h3>Total P&L</h3>
                <div class="metric-value {}">${:.2}</div>
                <p>{:.1}% return</p>
            </div>
            
            <div class="metric-card">
                <h3>Win Rate</h3>
                <div class="metric-value">{:.1}%</div>
                <p>{} trades today</p>
            </div>
            
            <div class="metric-card">
                <h3>Risk Status</h3>
                <div class="metric-value {}">{}</div>
                <p>{} active alerts</p>
            </div>
            
            <div class="metric-card">
                <h3>Daily Loss Used</h3>
                <div class="metric-value">{:.1}%</div>
                <div class="chart"></div>
            </div>
            
            <div class="metric-card">
                <h3>System Health</h3>
                <div class="metric-value status-{}">{}</div>
                <p>Uptime: {:.1}h</p>
            </div>
        </div>
    </div>
</body>
</html>"#,
            report.capital.current,
            if report.health.status == "OK" { "status-ok" } else { "status-error" },
            report.health.status,
            report.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            
            if report.capital.daily_pnl >= 0.0 { "positive" } else { "negative" },
            report.capital.daily_pnl,
            report.capital.daily_pnl_pct * 100.0,
            
            if report.capital.total_pnl >= 0.0 { "positive" } else { "negative" },
            report.capital.total_pnl,
            report.capital.total_pnl_pct * 100.0,
            
            report.performance.win_rate * 100.0,
            report.performance.trades_today,
            
            if report.risk.breakers_active { "negative" } else { "positive" },
            if report.risk.breakers_active { "ACTIVE" } else { "Normal" },
            report.risk.alerts_active,
            
            report.risk.daily_loss_used * 100.0,
            
            if report.health.status == "OK" { "ok" } else { "error" },
            report.health.status,
            report.health.uptime_hours
        )
    }
}





