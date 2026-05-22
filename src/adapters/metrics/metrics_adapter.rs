use std::collections::HashMap;
use std::sync::Mutex;
use crate::core::ports::metrics_port::IMetricsPort;

pub struct MetricsAdapter {
    counters: Mutex<HashMap<String, u64>>,
    histograms: Mutex<HashMap<String, Vec<f64>>>,
    gauges: Mutex<HashMap<String, f64>>,
}

impl MetricsAdapter {
    pub fn new() -> Self {
        Self {
            counters: Mutex::new(HashMap::new()),
            histograms: Mutex::new(HashMap::new()),
            gauges: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for MetricsAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl IMetricsPort for MetricsAdapter {
    fn increment_counter(&self, name: &str, _labels: &[(&str, &str)]) {
        let mut counters = self.counters.lock().unwrap();
        *counters.entry(name.to_string()).or_insert(0) += 1;
    }

    fn record_histogram(&self, name: &str, value: f64, _labels: &[(&str, &str)]) {
        let mut histograms = self.histograms.lock().unwrap();
        histograms.entry(name.to_string()).or_insert_with(Vec::new).push(value);
    }

    fn set_gauge(&self, name: &str, value: f64, _labels: &[(&str, &str)]) {
        let mut gauges = self.gauges.lock().unwrap();
        gauges.insert(name.to_string(), value);
    }
}
