use std::time::Instant;

pub struct TelemetryDecorator {
    pub component: String,
}

impl TelemetryDecorator {
    pub fn new(component: impl Into<String>) -> Self {
        Self { component: component.into() }
    }
    pub fn measure<F, T>(&self, operation: &str, f: F) -> T
    where F: FnOnce() -> T {
        let start = Instant::now();
        let result = f();
        let elapsed = start.elapsed();
        tracing::debug!(
            component = %self.component,
            operation = %operation,
            duration_us = elapsed.as_micros(),
            "operation completed"
        );
        result
    }
    pub fn measure_result<F, T, E>(&self, operation: &str, f: F) -> Result<T, E>
    where F: FnOnce() -> Result<T, E>, E: std::fmt::Debug {
        let start = Instant::now();
        let result = f();
        let elapsed = start.elapsed();
        match &result {
            Ok(_) => {
                tracing::debug!(
                    component = %self.component,
                    operation = %operation,
                    duration_us = elapsed.as_micros(),
                    "operation succeeded"
                );
            }
            Err(e) => {
                tracing::warn!(
                    component = %self.component,
                    operation = %operation,
                    duration_us = elapsed.as_micros(),
                    error = ?e,
                    "operation failed"
                );
            }
        }
        result
    }
}
