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

#[derive(Deserialize, Debug)]
pub struct IngestEventRequest {
    session_id: String,
    event_name: String,
    data: Option<Value>
}

#[derive(Deserialize)]
pub struct CreateSessionRequest {
    user_id: Option<String>,
    device_model: Option<String>,
    operating_system: Option<String>,
    screen_width: Option<u64>,
    screen_height: Option<u64>
}

#[derive(Serialize)]
struct CreateSessionResponse {
    session_id: String,
    user_id: String,
}

#[derive(Serialize)]
struct Event {
    id: i64,
    event_name: String,
    time: i64,
    data: Option<Value>,
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

fn get_user_agent<'a>(req: &'a HttpRequest) -> Option<&'a str> {
    req.headers().get("user-agent")?.to_str().ok()
}

pub async fn create_session(req: HttpRequest, data: web::Data<AppState>, payload: web::Json<CreateSessionRequest>) -> impl Responder {
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

    let user_agent = get_user_agent(&req);

    let session_id = Uuid::new_v4().to_string();

    let user_id = payload.user_id.clone()
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let execution = db_pool::with_connection(|conn| {
        conn.execute(
            "INSERT INTO sessions (session_id, user_id, start_date, ip_address, device_model, operating_system, screen_width, screen_height, user_agent) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![session_id, user_id, now(), ip, payload.device_model, payload.operating_system, payload.screen_width, payload.screen_height, user_agent],
        )
    });


    match execution {
        // ToDo: notify REDIS channel that sessionId was created
        Ok(_) => HttpResponse::Ok().json(CreateSessionResponse {
            session_id,
            user_id,
        }),

        Err(e) => HttpResponse::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("Session not created: {}", e)
        })
    }
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

    let execution = db_pool::with_connection(|conn| {
        conn.execute(
            "INSERT INTO events (session_id, timestamp, event_name, ip_address, params) VALUES (?1, ?2, ?3, ?4, json(?5))",
            params![
                payload.session_id,
                now(),
                payload.event_name,
                ip,
                payload.data.as_ref().map(|x| x.to_string())
            ],
        )
    });


    match execution {
        Ok(_) => HttpResponse::Ok().json(ApiResponse {
            // ToDo: notify REDIS channel that sessionId was updated
            success: true,
            message: "Event ingested".to_string()
        }),

        Err(e) => HttpResponse::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("Event not ingested: {}", e)
        })
    }
}

pub fn compare_secrets(secret_header: Option<&HeaderValue>, config: &Config) -> bool {
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
                    data: serde_json::from_str(&params_str).unwrap_or(None),
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

pub async fn health_check() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().finish())
}
