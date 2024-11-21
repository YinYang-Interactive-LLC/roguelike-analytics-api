![Roguelike Game Analytics Server](__assets/roguelike-analytics-api.jpg)

## Roguelike Game Analytics Server

An HTTP server designed for ingesting and retrieving events from roguelike games, optimized for high-speed data ingestion into an SQLite database while ensuring compliance with data privacy laws.

## Features

*   **High-Speed Ingestion**: Efficiently writes game event data into an SQLite database.
*   **Session Management**: Create new sessions and ingest custom events via HTTP endpoints.
*   **Data Retrieval**: Access session IDs and event data using a shared API secret.
*   **Rate Limiting**: Configurable rate limiter to control request flow and prevent abuse.
*   **Configurable**: Configuration supported using environment variables and .env files.
*   **IP Recording**: Logs the IP address for each session and event.
*   **Data Privacy Compliance**: Suitable for self-hosted analytics with respect to data privacy regulations.
*   **Redis Connection**: Notify subscribers on Redis instance if a session is created/touched for real time use cases.
*   **Future UI Development**: Plans to develop a user interface for data examination.

## Endpoints

*   `POST /create_session`: Create a new session. These parameters can be specified.
  - `user_id`: `string` (will be generated if not supplied)
  - `device_model`: `string` (default: `null`)
  - `operating_system`: `string` (default: `null`)
  - `screen_width`: `u64` (default: `null`)
  - `screen_height`: `u64` (default: `null`)
*   `POST /ingest_event`: Ingest a custom event into a session.
  - `session_id`: `string` (mandatory)
  - `event_name`: `string` (mandatory)
  - `data`: `object` (default: `{}`)
*   `GET /get_sessions`: Retrieve all session IDs (requires shared secret).
*   `GET /get_events/{session_id}`: Retrieve all events for a specific session (requires shared secret).

## Configuration

The server can be configured via environment variables:

`DB_PATH`: Path to the database (default `analytics.db`, relative from current working directory)

`SECRET_KEY:` Shared secret key for authenticated endpoints (no default value)

`MAX_EVENTS_PER_SECOND:` Maximum number of events per second (default: `5`)

`HOST:` Server host address (default: `"127.0.0.1"`)

`PORT:` Server port (default: `8080`)

`MAX_RATELIMIT_ENTRIES:` Maximum number of rate limit entries (default: `1000`)

`RATE_LIMITER_CLEANUP_INTERVAL:` Interval in seconds for rate limiter cleanup (default: `60`)

`RATELIMIT_CACHE_ENTRY_LIFETIME:` Lifetime in seconds for rate limit cache entries (default: `300`)

`CREATE_SESSION_COST:` Token cost for creating a session (default: `5`)

`INGEST_EVENT_COST:` Token cost for ingesting an event (default: `1`)

`TOKEN_BUCKET_SIZE:` Size of the token bucket for rate limiting (default: `10`)

`MAX_JSON_PAYLOAD:` Maximum allowed JSON payload size (default `4096` [bytes])

`ALLOWED_ORIGINS:` List of allowed base URLs that are allowed to request this api endpoint (default: `[]`)

`TRUST_PROXY:` extract IP from proxy headers if set to 1 (default: `0`)

`REDIS_HOSTNAME:` Hostname of redis instance that is reachable from the same network (default: `None` (connection disabled))

`REDIS_PORT:` The port of the listening Redis instance (default: `6379`)

`REDIS_USERNAME:` Username of redis connection (default: `None`)

`REDIS_PASSWORD:` Password of redis connection (default: `None`)

`REDIS_PROTOCOL:` Specify either `resp2` or `resp3` (default: `resp3`)

`REDIS_DATABASE:` The database instance number on the Redis client (default: `0`)

`REDIS_USE_TLS:` Whether or not to use TLS over TCP (default: `false`)

## Installation

1.  **Clone the Repository**
    
    `git clone https://github.com/yourusername/roguelike-analytics-server.git`
    
2.  **Navigate to the Project Directory**
    
    `cd roguelike-analytics-server`
    
3.  **Set Environment Variables**
    
    Configure the server by setting the necessary environment variables as per your requirements.
    
4.  **Build the Project**
    
    `cargo build --release`
    
5.  **Run the Server**
    
    `cargo run --release`
    

## Usage

*   **Create a New Session**
    
    Send a `POST` request to `/create_session`. The response will include a session ID.
    
*   **Ingest an Event**
    
    Send a `POST` request to `/ingest_event` with the session ID and event data in the request body.
    
*   **Retrieve All Sessions**
    
    Send a `GET` request to `/get_sessions` with the shared secret provided in the `Authorization` header.
    
*   **Retrieve Events for a Session**
    
    Send a `GET` request to `/get_events/{session_id}` with the shared secret provided in the `Authorization` header.

```bash
# Replace {session_id} with the actual session ID whose events you want to retrieve. 
# Note that YOUR_SECRET_KEY will be the key that you set using the environment variable SECRET_KEY.
curl -X GET http://localhost:8080/get_events/{session_id} \
     -H "X-RLA-KEY: YOUR_SECRET_KEY"
```
    

## Redis PUB/SUB

This project can optionally connect to a Redis instance to publish significant events (using PUB) when specific data changes occur.

Subscribers can subscribe to the following message types:

- `evt_session_created <sessionId>`: A session was just created

- `evt_session_updated <sessionId>`: A session was updated (event was posted)

> Note that the Redis connection will only be enabled if a hostname is specified using `REDIS_HOSTNAME` and a connection test will be made on startup. Make sure the redis instance is available during the launch of the API server.

## Dependencies

*   **Rust**
*   **Actix Web**: Web framework backend.
*   **SQLite**: Database for storing event data.

## Future Development

*   **User Interface**: A web-based UI will be developed to examine and visualize the collected data.
*   **Enhanced Analytics**: Additional analytical tools tailored for roguelike games.
*   **Extended Data Privacy Features**: Further compliance enhancements with data privacy laws.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request for any improvements or features.

## License

This project is licensed under the Apache 2.0 License.

## Contact

For support or inquiries, please contact \[hello@yinyang-interactive.com\].
