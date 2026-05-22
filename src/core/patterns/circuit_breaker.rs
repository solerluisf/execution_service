use std::sync::Mutex;
use std::time::{Duration, Instant};
use crate::core::infrastructure::mutex_ext::MutexExt;

#[derive(Debug, Clone, PartialEq)]
enum State { Closed, Open { until: Instant }, HalfOpen }

struct Inner {
    state: State,
    failures: u32,
    failure_threshold: u32,
    cooldown: Duration,
}

pub struct CircuitBreaker {
    broker_id: String,
    inner: Mutex<Inner>,
}

impl CircuitBreaker {
    pub fn new(broker_id: impl Into<String>, failure_threshold: u32, cooldown_secs: u64) -> Self {
        Self {
            broker_id: broker_id.into(),
            inner: Mutex::new(Inner { state: State::Closed, failures: 0, failure_threshold, cooldown: Duration::from_secs(cooldown_secs) }),
        }
    }
    pub fn call<F, T>(&self, f: F) -> Result<T, String>
    where F: FnOnce() -> Result<T, String> {
        {
            let mut g = self.inner.safe_lock();
            match &g.state {
                State::Open { until } => {
                    if Instant::now() < *until {
                        let msg = format!("circuit_breaker.rejected broker={}", self.broker_id);
                        tracing::warn!("{}", msg);
                        return Err(msg);
                    } else {
                        g.state = State::HalfOpen;
                        tracing::info!("circuit_breaker.half_open broker={}", self.broker_id);
                    }
                }
                State::Closed | State::HalfOpen => {}
            }
        }
        match f() {
            Ok(v) => { self.record_success(); Ok(v) }
            Err(e) => { self.record_failure(&e); Err(e) }
        }
    }
    pub fn record_success(&self) {
        let mut g = self.inner.safe_lock();
        let was_half_open = g.state == State::HalfOpen;
        g.failures = 0;
        g.state = State::Closed;
        if was_half_open { tracing::info!("circuit_breaker.closed broker={}", self.broker_id); }
    }
    pub fn record_failure<E: std::fmt::Display>(&self, err: &E) {
        let mut g = self.inner.safe_lock();
        g.failures += 1;
        tracing::warn!("circuit_breaker.failure broker={} failures={} err={}", self.broker_id, g.failures, err);
        if g.failures >= g.failure_threshold || g.state == State::HalfOpen {
            let cooldown = g.cooldown.max(Duration::from_millis(1));
            let until = Instant::now() + cooldown;
            g.state = State::Open { until };
            tracing::error!("circuit_breaker.opened broker={} cooldown_secs={}", self.broker_id, g.cooldown.as_secs());
        }
    }
    pub fn is_open(&self) -> bool {
        let g = self.inner.safe_lock();
        matches!(&g.state, State::Open { until } if Instant::now() < *until)
    }
}
