// Metrics module for monitoring system
// Provides metric collection and aggregation

use super::{MetricType, MetricValue};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Metrics collector
pub struct MetricsCollector {
    metrics: Arc<RwLock<HashMap<MetricType, Vec<MetricValue>>>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn record(&self, metric_type: MetricType, value: f64) {
        let mut metrics = self.metrics.write().await;
        let entry = metrics.entry(metric_type).or_insert_with(Vec::new);
        entry.push(MetricValue {
            value,
            timestamp: std::time::SystemTime::now(),
        });
    }

    pub async fn get_latest(&self, metric_type: &MetricType) -> Option<f64> {
        let metrics = self.metrics.read().await;
        metrics.get(metric_type)
            .and_then(|values| values.last())
            .map(|v| v.value)
    }
}
