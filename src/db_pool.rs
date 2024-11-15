use rusqlite::{Connection};
use std::sync::Once;
use std::env;

static INIT: Once = Once::new();
static DB_PATH: &str = "analytics.db";

fn initialize_database() {
    INIT.call_once(|| {
        let path = env::var("DB_PATH").unwrap_or_else(|_| DB_PATH.to_string());
        let conn = Connection::open(path).expect("Failed to open database");

        // Create tables if they do not exist
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS sessions (
                session_id TEXT PRIMARY KEY NOT NULL,
                user_id TEXT,
                start_date TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                ip_address TEXT NOT NULL,
                device_model TEXT,
                operating_system TEXT,
                screen_width INT,
                screen_height INT
            );
            CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                event_name TEXT NOT NULL,
                timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                ip_address TEXT NOT NULL,
                params TEXT,
                FOREIGN KEY(session_id) REFERENCES sessions(session_id)
            );

            CREATE INDEX IF NOT EXISTS user_idx ON sessions (user_id);
            CREATE INDEX IF NOT EXISTS session_idx ON events (session_id);
            CREATE INDEX IF NOT EXISTS session_event_name_idx ON events (session_id, event_name);
            ",
        )
        .expect("Failed to create tables");
    });
}

thread_local! {
    static DB_CONNECTION: Connection = {
        initialize_database();
        let path = env::var("DB_PATH").unwrap_or_else(|_| DB_PATH.to_string());
        let conn = Connection::open(path).expect("Failed to open database");

        // Enable WAL mode and other PRAGMAs
        conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA foreign_keys = ON;
            ",
        )
        .expect("Failed to set PRAGMAs");

        conn
    };
}

pub fn with_connection<F, R>(f: F) -> R
where
    F: FnOnce(&Connection) -> R,
{
    DB_CONNECTION.with(|conn| f(conn))
}
