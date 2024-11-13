use actix_web::{web, Error, HttpRequest, HttpResponse, Responder, http::header::HeaderValue};
use serde::{Deserialize, Serialize};
use rusqlite::params;
use serde_json::Value;
use uuid::Uuid;
use chrono::{Utc};

use crate::db_pool;
use crate::config::{Config};
use crate::app_state::{AppState};
use crate::rate_limit::{check_rate_limit};

#[derive(Deserialize)]
pub struct IngestEventRequest {
    session_id: String,
    event_name: String,
    data: Value,
}

#[derive(Serialize)]
struct CreateSessionResponse {
    session_id: String
}

#[derive(Serialize)]
struct Event {
    id: i64,
    event_name: String,
    time: i64,
    data: Value,
}

#[derive(Serialize)]
struct SessionInfo {
    session_id: String,
    start_date: i64,
}

#[derive(Serialize)]
pub struct ApiResponse {
    success: bool,
    message: String,
}

pub fn now() -> i64 {
    let now = Utc::now();

    let millis_since_epoch = now.timestamp_millis();

    millis_since_epoch
}

pub async fn create_session(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    // Rate limiting per IP address
    let ip = req
        .peer_addr()
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    if !check_rate_limit(&data, &ip, data.config.create_session_cost) {
        return HttpResponse::TooManyRequests().json(ApiResponse {
            success: false,
            message: "Rate limit exceeded".to_string()
        });
    }

    let session_id = Uuid::new_v4().to_string();

    db_pool::with_connection(|conn| {
        conn.execute(
            "INSERT INTO sessions (session_id, start_date, ip_address, device_model, operating_system, screen_width, screen_height) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![session_id, now(), ip, ],
        )
        .unwrap();
    });

    HttpResponse::Ok().json(CreateSessionResponse {
        session_id,
    })
}

pub async fn ingest_event(
    req: HttpRequest,
    data: web::Data<AppState>,
    payload: web::Json<IngestEventRequest>,
) -> impl Responder {
    // Rate limiting per IP address
    let ip = req
        .peer_addr()
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    if !check_rate_limit(&data, &ip, data.config.ingest_event_cost) {
        return HttpResponse::TooManyRequests().json(ApiResponse {
            success: false,
            message: "Rate limit exceeded".to_string()
        })        
    }

    db_pool::with_connection(|conn| {
        conn.execute(
            "INSERT INTO events (session_id, timestamp, event_name, ip_address, params) VALUES (?1, ?2, ?3, ?4, json(?5))",
            params![
                payload.session_id,
                now(),
                payload.event_name,
                ip,
                payload.data.to_string()
            ],
        )
        .unwrap();
    });

    HttpResponse::Ok().json(ApiResponse {
        success: true,
        message: "Event ingested".to_string()
    })
}

pub fn compare_secrets(secret_header: Option<&HeaderValue>, config: &Config) -> bool {
    println!("S {:?} {:?}", secret_header, config.secret_key);
    if secret_header.is_none() || config.secret_key.is_none() {
        return false;
    }
    secret_header.unwrap().to_str().unwrap() == config.secret_key.as_ref().unwrap()
}

pub async fn get_events(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, Error> {
    // Check for shared secret
    let secret = req.headers().get("X-RLA-KEY");
    if !compare_secrets(secret, &data.config) {
        return Ok(HttpResponse::Unauthorized().json(ApiResponse {
            success: false,
            message: "Insufficient permissions".to_string()
        }))             
    }

    let session_id = path.into_inner();

    let events = db_pool::with_connection(|conn| {
        let mut stmt = conn
            .prepare_cached(
                "SELECT
                    id,
                    event_name,
                    timestamp,
                    params
                FROM events
                WHERE session_id = ?1",
            )
            .unwrap();

        let events_iter = stmt
            .query_map(params![session_id], |row| {
                let params_str: String = row.get(3)?;
                Ok(Event {
                    id: row.get(0)?,
                    event_name: row.get(1)?,
                    time: row.get(2)?,
                    data: serde_json::from_str(&params_str).unwrap_or(Value::Null),
                })
            })
            .unwrap();

        events_iter.map(|event| event.unwrap()).collect::<Vec<Event>>()
    });

    Ok(HttpResponse::Ok().json(events))
}

pub async fn get_sessions(req: HttpRequest, data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    // Check for shared secret
    let secret = req.headers().get("X-RLA-KEY");
    if !compare_secrets(secret, &data.config) {
        return Ok(HttpResponse::Unauthorized().json(ApiResponse {
            success: false,
            message: "Insufficient permissions".to_string()
        }));
    }

    let sessions = db_pool::with_connection(|conn| {
        let mut stmt = conn
            .prepare_cached("SELECT session_id, start_date FROM sessions")
            .unwrap();

        let sessions_iter = stmt
            .query_map([], |row| {
                Ok(SessionInfo {
                    session_id: row.get(0)?,
                    start_date: row.get(1)?,
                })
            })
            .unwrap();

        sessions_iter
            .map(|session| session.unwrap())
            .collect::<Vec<SessionInfo>>()
    });

    Ok(HttpResponse::Ok().json(sessions))
}
