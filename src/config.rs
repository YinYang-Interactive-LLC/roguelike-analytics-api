use std::env;

#[derive(Clone)]
pub struct Config {
    pub secret_key: Option<String>,
    pub max_events_per_second: u64,
    pub host: String,
    pub port: u16,
    pub max_ratelimit_entries: usize,
    pub ratelimiter_cleanup_interval: u64,
    pub ratelimit_cache_entry_lifetime: u64,
    pub create_session_cost: u64,
    pub ingest_event_cost: u64,
    pub token_bucket_size: u64,
    pub trust_proxy: u64,
    pub max_json_payload: usize,
    pub cors_origins: Option<String>,

    pub redis_connection_hostname: Option<String>,
    pub redis_connection_db: i64,
    pub redis_connection_port: u16,
    pub redis_connection_use_tls: bool,
    pub redis_connection_username: Option<String>,
    pub redis_connection_password: Option<String>,
    pub redis_connection_protocol: Option<String>,
}

fn parse_bool(input: Option<String>, default_value: bool) -> bool {
    match input.as_deref() {
        Some("") => default_value,

        Some("true") | Some("True") | Some("TRUE") => true,
        Some("yes") | Some("y") => true,
        Some("YES") | Some("Y") => true,
        Some("1") => true,

        Some("false") | Some("False") | Some("FALSE") => false,
        Some("no") | Some("n") => false,
        Some("NO") | Some("N") => false,
        Some("0") => false,

        _ => default_value
    }
}

impl Config {
    pub fn from_env() -> Config {
        Config {
            secret_key: env::var("SECRET_KEY").ok(),
            max_events_per_second: env::var("MAX_EVENTS_PER_SECOND")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .expect("Invalid value provided for MAX_EVENTS_PER_SECOND"),
            host: env::var("HOST")
                .unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .expect("Invalid value provided for PORT"),
            max_ratelimit_entries: env::var("MAX_RATELIMIT_ENTRIES")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .expect("Invalid value provided for MAX_RATELIMIT_ENTRIES"),
            ratelimiter_cleanup_interval: env::var("RATE_LIMITER_CLEANUP_INTERVAL")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .expect("Invalid value provided for RATE_LIMITER_CLEANUP_INTERVAL"),
            ratelimit_cache_entry_lifetime: env::var("RATELIMIT_CACHE_ENTRY_LIFETIME")
                .unwrap_or_else(|_| "300".to_string())
                .parse()
                .expect("Invalid value provided for RATELIMIT_CACHE_ENTRY_LIFETIME"),
            create_session_cost: env::var("CREATE_SESSION_COST")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .expect("Invalid value provided for CREATE_SESSION_COST"),
            ingest_event_cost: env::var("INGEST_EVENT_COST")
                .unwrap_or_else(|_| "1".to_string())
                .parse()
                .expect("Invalid value provided for INGEST_EVENT_COST"),
            token_bucket_size: env::var("TOKEN_BUCKET_SIZE")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .expect("Invalid value provided for TOKEN_BUCKET_SIZE"),
            max_json_payload: env::var("MAX_JSON_PAYLOAD")
                .unwrap_or_else(|_| "4096".to_string())
                .parse()
                .expect("Invalid value provided for MAX_JSON_PAYLOAD"),
            cors_origins: env::var("ALLOWED_ORIGINS").ok(),
            trust_proxy: env::var("TRUST_PROXY")
                .unwrap_or_else(|_| "0".to_string())
                .parse()
                .expect("Trust proxy must either be zero or non-zero"),

            redis_connection_hostname: env::var("REDIS_HOSTNAME").ok(),
            redis_connection_username: env::var("REDIS_USERNAME").ok(),
            redis_connection_password: env::var("REDIS_PASSWORD").ok(),
            redis_connection_protocol: env::var("REDIS_PROTOCOL").ok(),
            redis_connection_port: env::var("REDIS_PORT")
                .unwrap_or_else(|_| "6379".to_string())
                .parse()
                .expect("Invalid number specified for REDIS_PORT"),            
            redis_connection_db: env::var("REDIS_DATABASE")
                .unwrap_or_else(|_| "0".to_string())
                .parse()
                .expect("Invalid number specified for REDIS_DATABASE"),
            redis_connection_use_tls: parse_bool(env::var("REDIS_USE_TLS").ok(), false),            
        }      
    }
}
