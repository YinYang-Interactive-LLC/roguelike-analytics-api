use rusqlite::{Connection};
use std::sync::Once;

static INIT: Once = Once::new();
static DB_PATH: &str = "analytics.db";

fn initialize_database() {
    INIT.call_once(|| {
        let conn = Connection::open(DB_PATH).expect("Failed to open database");

        // Create tables if they do not exist
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS sessions (
                session_row_id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT UNIQUE NOT NULL,
                start_date INTEGER NOT NULL,
                ip_address TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_row_id TEXT NOT NULL,
                event_name TEXT NOT NULL,
                time INTEGER NOT NULL,
                ip_address TEXT NOT NULL,
                params TEXT,
                FOREIGN KEY(session_row_id) REFERENCES sessions(session_row_id)
            );
            ",
        )
        .expect("Failed to create tables");
    });
}

thread_local! {
    static DB_CONNECTION: Connection = {
        initialize_database();
        let conn = Connection::open(DB_PATH).expect("Failed to open database");

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
