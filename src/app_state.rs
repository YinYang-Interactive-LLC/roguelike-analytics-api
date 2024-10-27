use parking_lot::Mutex;

use std::sync::Arc;
use std::collections::HashMap;

use crate::rate_limit::{RateLimitInfo};
use crate::config::{Config};

#[derive(Clone)]
pub struct AppState {
    pub rate_limiter: Arc<Mutex<HashMap<String, RateLimitInfo>>>,
    pub config: Arc<Config>,
}

impl AppState {
    pub fn init(config: Config) -> AppState {
        AppState {
            rate_limiter: Arc::new(Mutex::new(HashMap::new())),
            config: Arc::new(config),
        }
    }
}
