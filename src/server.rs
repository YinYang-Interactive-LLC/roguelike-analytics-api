use std::time::{Duration};

use dotenv::dotenv;
use actix_web::{middleware, web, App, HttpServer, error, HttpResponse};
use serde::{Serialize};

use crate::app_state::{AppState};
use crate::config::{Config};
use crate::rate_limit::{cleanup_rate_limiter};
use crate::route_handlers::{
    create_session,
    ingest_event,
    get_events,
    get_sessions,
    health_check
};

#[derive(Serialize)]
struct PublicJsonError {
    pub message: String
}

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // Initialize logger
    env_logger::init();

    // Read the config from env vars
    let config = Config::from_env();

    // Create new app state
    let data = web::Data::new(AppState::init(config));

    // Spawn cleanup worker
    let rate_limiter_clone = data.rate_limiter.clone();
    let config_clone = data.config.clone();

    // Create a worker that cleans the ratelimit
    actix_web::rt::spawn(async move {
        let mut interval =
            tokio::time::interval(Duration::from_secs(config_clone.ratelimiter_cleanup_interval));
        loop {
            interval.tick().await;
            cleanup_rate_limiter(&rate_limiter_clone, &config_clone);
        }
    });

    // Create a clone for the binding
    let config_task = data.config.clone();

    let max_json_payload = config_task.max_json_payload;


    // Start server
    let server = HttpServer::new(move || {
        let json_config = web::JsonConfig::default()
            .limit(max_json_payload)
            .error_handler(|err, _req| {
                println!("{:?}", err);
                match err {
                    error::JsonPayloadError::OverflowKnownLength { limit, .. } |
                    error::JsonPayloadError::Overflow { limit } => {
                        // Handle payload too large error
                        let response = HttpResponse::PayloadTooLarge().json(
                            PublicJsonError { message: format!("Payload too large. Maximum size allowed is {} bytes", limit) }
                        );
                        error::InternalError::from_response(err, response).into()
                    },
                    _ => {
                        // Handle other JSON parsing errors
                        let response = HttpResponse::BadRequest().json(
                            PublicJsonError { message: format!("Invalid JSON: {}", err) }
                        );
                        error::InternalError::from_response(err, response).into()
                    }
                }
            });

        App::new()
            .wrap(middleware::Logger::default())
            .app_data(json_config)
            .app_data(data.clone())
            .service(
                web::resource("/create_session").route(web::post().to(create_session)),
            )
            .service(
                web::resource("/ingest_event").route(web::post().to(ingest_event)),
            )
            .service(web::resource("/get_events/{session_id}").route(web::get().to(get_events)))
            .service(web::resource("/get_sessions").route(web::get().to(get_sessions)))
            .service(web::resource("/health_check").route(web::get().to(health_check)))
    });

    println!("Listening on {:?}", (config_task.host.as_str(), config_task.port));
    
    server.bind((config_task.host.as_str(), config_task.port))?
    .run()
    .await
}
