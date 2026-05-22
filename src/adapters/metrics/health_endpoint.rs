use crate::core::ports::health_port::IHealthReporter;

pub struct HealthReporter {
    healthy: std::sync::atomic::AtomicBool,
}

impl HealthReporter {
    pub fn new() -> Self {
        Self {
            healthy: std::sync::atomic::AtomicBool::new(true),
        }
    }

    pub fn set_healthy(&self, healthy: bool) {
        self.healthy.store(healthy, std::sync::atomic::Ordering::Relaxed);
    }
}

impl Default for HealthReporter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl IHealthReporter for HealthReporter {
    fn is_healthy(&self) -> bool {
        self.healthy.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn get_status(&self) -> String {
        if self.is_healthy() {
            "healthy".to_string()
        } else {
            "unhealthy".to_string()
        }
    }
}
