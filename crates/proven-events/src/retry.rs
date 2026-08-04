//! Retry with exponential backoff + jitter for publish/handler paths.

use std::future::Future;
use std::time::Duration;

use tokio::time::sleep;
use tracing::{debug, warn};

use crate::error::EventError;

/// Retry policy for NATS publish and handler execution.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_backoff: Duration,
    pub max_backoff: Duration,
    pub multiplier: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            initial_backoff: Duration::from_millis(50),
            max_backoff: Duration::from_secs(5),
            multiplier: 2.0,
        }
    }
}

impl RetryPolicy {
    pub fn backoff_for_attempt(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::ZERO;
        }
        let exp = self.multiplier.powi((attempt.saturating_sub(1)) as i32);
        let ms = (self.initial_backoff.as_millis() as f64 * exp) as u64;
        let capped = ms.min(self.max_backoff.as_millis() as u64);
        // Deterministic half-jitter from attempt number (no RNG dependency).
        let jitter = capped / 4 * ((attempt % 3) as u64 + 1) / 3;
        Duration::from_millis(capped.saturating_sub(jitter).max(1))
    }
}

/// Run `op` with retries. `op` receives the 1-based attempt number.
pub async fn retry_with_backoff<F, Fut, T>(
    policy: &RetryPolicy,
    operation: &str,
    mut op: F,
) -> Result<T, EventError>
where
    F: FnMut(u32) -> Fut,
    Fut: Future<Output = Result<T, EventError>>,
{
    let mut last_err = EventError::Internal("retry: no attempts".into());
    for attempt in 1..=policy.max_attempts {
        if attempt > 1 {
            let delay = policy.backoff_for_attempt(attempt);
            debug!(
                operation = operation,
                attempt,
                delay_ms = delay.as_millis() as u64,
                "retrying after backoff"
            );
            sleep(delay).await;
        }

        match op(attempt).await {
            Ok(value) => return Ok(value),
            Err(err) => {
                warn!(
                    operation = operation,
                    attempt,
                    max_attempts = policy.max_attempts,
                    error = %err,
                    "operation failed"
                );
                last_err = err;
            }
        }
    }

    Err(EventError::RetryExhausted {
        attempts: policy.max_attempts,
        message: last_err.to_string(),
    })
}
