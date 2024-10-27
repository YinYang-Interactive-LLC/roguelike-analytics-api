use std::sync::Arc;
use std::time::{Instant, Duration};
use std::collections::HashMap;

use parking_lot::Mutex;

use crate::app_state::{AppState};
use crate::config::{Config};

pub struct RateLimitInfo {
    pub tokens: u64,
    pub last_refill: Instant,
    pub last_access: Instant,
}

pub fn check_rate_limit(state: &AppState, ip: &str, cost: u64) -> bool {
    let mut rate_limiter = state.rate_limiter.lock();

    let now = Instant::now();
    let rate_limit = state.config.max_events_per_second;
    let token_bucket_size = state.config.token_bucket_size;
    let max_tokens = rate_limit * token_bucket_size;

    let entry = rate_limiter.entry(ip.to_string()).or_insert_with(|| RateLimitInfo {
        tokens: max_tokens,
        last_refill: now,
        last_access: now,
    });

    // Refill tokens
    let elapsed_secs = now.duration_since(entry.last_refill).as_secs();
    if elapsed_secs >= 1 {
        let refill_tokens = elapsed_secs * rate_limit;
        entry.tokens = (entry.tokens + refill_tokens).min(max_tokens);
        entry.last_refill = now;
    }

    entry.last_access = now;

    if entry.tokens >= cost {
        // Consume tokens and allow the request
        entry.tokens -= cost;
        true
    } else {
        // Rate limit exceeded
        false
    }
}

pub fn cleanup_rate_limiter(
    rate_limiter: &Arc<Mutex<HashMap<String, RateLimitInfo>>>,
    config: &Arc<Config>,
) {
    let mut rate_limiter = rate_limiter.lock();
    let now = Instant::now();
    let lifetime = Duration::from_secs(config.ratelimit_cache_entry_lifetime);

    // Remove old entries
    rate_limiter.retain(|_, entry| now.duration_since(entry.last_access) <= lifetime);

    // If rate limiter has more than max_ratelimit_entries, remove oldest entries
    if rate_limiter.len() > config.max_ratelimit_entries {
        let mut entries: Vec<_> = rate_limiter.iter().collect();
        // Sort entries by last_access time, oldest first
        entries.sort_by_key(|&(_, entry)| entry.last_access);

        // Compute how many entries to remove
        let num_to_remove = rate_limiter.len() - config.max_ratelimit_entries;

        // Collect keys to remove
        let keys_to_remove: Vec<String> = entries
            .iter()
            .take(num_to_remove)
            .map(|(ip, _)| (*ip).clone())
            .collect();

        // Remove entries
        for ip in keys_to_remove {
            rate_limiter.remove(&ip);
        }
    }
}
