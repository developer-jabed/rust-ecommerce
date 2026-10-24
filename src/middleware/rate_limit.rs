//! Simple in-memory rate-limit scaffolding.
//! Production deployments should prefer a reverse-proxy or Redis-backed limiter.
//! Full implementation is deferred until measurable need; the skeleton is here
//! so routes can be annotated later.

use crate::errors::AppError;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use once_cell::sync::Lazy;

static RATE_MAP: Lazy<Mutex<HashMap<String, (u32, Instant)>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Extremely simple fixed-window limiter (per-key, in-process).
/// Not suitable for multi-instance production without shared storage.
pub fn check_rate_limit(key: &str, limit: u32, window: Duration) -> Result<(), AppError> {
    let mut map = RATE_MAP.lock().map_err(|_| AppError::Internal)?;
    let now = Instant::now();

    match map.get_mut(key) {
        Some((count, start)) => {
            if now.duration_since(*start) > window {
                *count = 1;
                *start = now;
                Ok(())
            } else if *count >= limit {
                Err(AppError::RateLimited)
            } else {
                *count += 1;
                Ok(())
            }
        }
        None => {
            map.insert(key.to_string(), (1, now));
            Ok(())
        }
    }
}