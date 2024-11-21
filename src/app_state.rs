use parking_lot::Mutex;
use deadpool_redis::{
    Config as RedisConfig,
    Pool as RedisPool,
    ProtocolVersion as RedisProtocolVersion,
    ConnectionAddr as RedisConnectionAddr,
    Runtime,
    PoolError
};

use std::sync::Arc;
use std::collections::HashMap;

use crate::rate_limit::{RateLimitInfo};
use crate::config::{Config};

#[derive(Clone)]
pub struct AppState {
    pub rate_limiter: Arc<Mutex<HashMap<String, RateLimitInfo>>>,
    pub config: Arc<Config>,
    pub redis_pool: Option<Arc<RedisPool>>,
}

#[derive(Debug)]
pub struct ConnectionTestError {
    error: PoolError
}

impl From<PoolError> for ConnectionTestError {
    fn from(error: PoolError) -> ConnectionTestError {
        ConnectionTestError {
            error
        }
    }
}

impl From<ConnectionTestError> for std::io::Error {
    fn from(err: ConnectionTestError) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::ConnectionAborted, format!("{:?}", err.error))
    }    
}

fn parse_redis_protocol(protocol: Option<String>) -> RedisProtocolVersion {
    match protocol.as_deref() {
        Some("resp2") => RedisProtocolVersion::RESP2,
        Some("resp3") => RedisProtocolVersion::RESP3,
        None => RedisProtocolVersion::RESP3,
        _ => panic!("Invalid REDIS_PROTOCOL specified")
    }
}

fn get_redis_connection_addr(hostname: &str, port: u16, use_tls: bool) -> RedisConnectionAddr {
    match use_tls {
        false => RedisConnectionAddr::Tcp(hostname.to_string(), port),
        true => RedisConnectionAddr::TcpTls {
            host: hostname.to_string(),
            port: port,
            insecure: false,
        }
    }
}

impl AppState {
    pub fn init(config: Config) -> AppState {
        let redis_pool = config.redis_connection_hostname.as_ref().map(|hostname| {
            let cfg = &config;

            let mut redis_cfg = RedisConfig::default();

            if let Some(ref mut connection) = redis_cfg.connection {
                connection.redis.db = cfg.redis_connection_db;
                connection.redis.username = cfg.redis_connection_username.clone();
                connection.redis.password = cfg.redis_connection_password.clone();
                connection.redis.protocol = parse_redis_protocol(cfg.redis_connection_protocol.clone());

                connection.addr = get_redis_connection_addr(&hostname, cfg.redis_connection_port, cfg.redis_connection_use_tls);
            }

            let pool = redis_cfg.create_pool(Some(Runtime::Tokio1)).expect("Unable to create redis pool");

            Arc::new(pool)
        });

        AppState {
            rate_limiter: Arc::new(Mutex::new(HashMap::new())),
            config: Arc::new(config),
            redis_pool: redis_pool
        }
    }

    pub async fn test_connection(&self) -> Result<(), ConnectionTestError> {
        if let Some(redis_pool) = self.redis_pool.as_ref() {
            redis_pool.get().await?;
        }
        Ok(())
    }
}
