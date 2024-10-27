use std::env;

#[derive(Clone)]
pub struct Config {
    pub secret_key: String,
    pub max_events_per_second: u64,
    pub host: String,
    pub port: u16,
    pub max_ratelimit_entries: usize,
    pub ratelimiter_cleanup_interval: u64,
    pub ratelimit_cache_entry_lifetime: u64,
    pub create_session_cost: u64,
    pub ingest_event_cost: u64,
    pub token_bucket_size: u64,
}

impl Config {
    pub fn from_env() -> Config {
        Config {
            secret_key: env::var("SECRET_KEY").unwrap_or_else(|_| "your_shared_secret".to_string()),
            max_events_per_second: env::var("MAX_EVENTS_PER_SECOND")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            max_ratelimit_entries: env::var("MAX_RATELIMIT_ENTRIES")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000),
            ratelimiter_cleanup_interval: env::var("RATE_LIMITER_CLEANUP_INTERVAL")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .unwrap_or(60),
            ratelimit_cache_entry_lifetime: env::var("RATELIMIT_CACHE_ENTRY_LIFETIME")
                .unwrap_or_else(|_| "300".to_string())
                .parse()
                .unwrap_or(300),
            create_session_cost: env::var("CREATE_SESSION_COST")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            ingest_event_cost: env::var("INGEST_EVENT_COST")
                .unwrap_or_else(|_| "1".to_string())
                .parse()
                .unwrap_or(1),
            token_bucket_size: env::var("TOKEN_BUCKET_SIZE")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
        }      
    }
}
