pub mod coffee;
pub mod conf;
pub mod dbms;
pub mod facade;
pub mod jsons;
pub mod preset;
pub mod sql;
pub mod status;
pub mod results;
pub mod zero;

use crate::erx;
use redis;

// use deadpool_redis::{
//     redis::{
//         FromRedisValue,
//     },
//     Config as DeadPConfig,
//     Runtime as DeadPRuntime,
// };

use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::sync::RwLock;
use std::time::Duration;
use tokio::sync::OnceCell;
use tracing::{info, span, warn};

use crate::conf::{Backend, BackendKind, Dict};
use crate::web::url;


/// get shared DatabaseConnection
/// panic if error
pub fn shared_must() -> &'static DatabaseConnection {
    SHARED_DB_CONNECTION.get().expect("SHARED_DB_CONNECTION get failed")
}

/// get shared DatabaseConnection
pub fn shared() -> erx::ResultE<&'static DatabaseConnection> {
    SHARED_DB_CONNECTION.get().ok_or("SHARED_DB_CONNECTION get failed".into())
}

/// For async connections, connection pooling isn't necessary, unless blocking commands are used.
/// The MultiplexedConnection is cloneable and can be used safely from multiple threads, so a single connection can be easily reused.
/// For automatic reconnections consider using ConnectionManager with the connection-manager feature.
/// Async cluster connections also don't require pooling and are thread-safe and reusable.
pub fn make_redis_client() -> erx::ResultE<redis::Client> {
    let s = SHARED_REDIS_CONNECT_STRING.read().map_err(erx::smp)?.clone();
    redis::Client::open(s).map_err(erx::smp)
}

// get redis connection from pool
// pub async fn get_redis_client() -> erx::ResultE<deadpool_redis::Connection> {
//     let pool = SHARED_REDIS_POOL.get_or_init(|| async {
//         deadpool_redis::Config::from_url(SHARED_REDIS_CONNECT_STRING.read().unwrap().clone())
//             .create_pool(Some(DeadPRuntime::Tokio1))
//             .unwrap()
//     }).await;
//
//     Ok(pool.get().await.map_err(erx::smp)?)
// }

/// initialize model connection
/// call once when application initialized
pub async fn initialize_model_connection(backends: Dict<Backend>) {
    let span = span!(tracing::Level::INFO, "INITIALIZE MODEL");
    let _guard = span.enter();

    if backends.is_empty() {
        warn!("No backends configured, pass init_model.");
        return;
    }

    async fn postgre(backend: Backend) {
        info!("Connecting to postgres: {:?}", backend.connect);
        SHARED_DB_CONNECTION.get_or_init(|| async { new_database_connection(backend).await }).await;
    }

    async fn redis(backend: Backend) {
        let connect_string = backend.connect.clone();

        let mut conn = SHARED_REDIS_CONNECT_STRING.write().unwrap();
        *conn = connect_string.clone();

        info!("Connecting to redis: {:?}", connect_string);
        let cli = redis::Client::open(connect_string.as_str()).expect("Redis connection failed.");
        info!("Connected to redis: {:?}", cli.get_connection_info());
    }

    for (backend_name, backend) in backends {
        if backend.connect.len() < 1 {
            warn!("Backend '{}' connect string is empty, pass", backend_name);
            continue;
        }

        match backend.kind {
            BackendKind::Redis => redis(backend).await,
            BackendKind::Postgres => postgre(backend).await,
        }
    }
}

/// new database connection
pub async fn new_database_connection(backend: Backend) -> DatabaseConnection {
    match backend.kind {
        BackendKind::Redis => {
            panic!("Redis Backend use new_database_connection is not supported yet");
        },
        BackendKind::Postgres => (),
    };

    const MAX_CONNECTIONS: u32 = 100;
    const MIN_CONNECTIONS: u32 = 2;
    const CONNECT_TIMEOUT: Duration = Duration::from_secs(8);
    const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(8);
    const IDLE_TIMEOUT: Duration = Duration::from_secs(60 * 5);
    const MAX_LIFETIME: Duration = Duration::from_secs(60 * 60 * 1);

    let connection_string = backend.connect.clone();
    let mut opt = ConnectOptions::new(connection_string.clone());

    use log;

    opt.max_connections(MAX_CONNECTIONS)
        .min_connections(MIN_CONNECTIONS)
        .connect_timeout(CONNECT_TIMEOUT)
        .acquire_timeout(ACQUIRE_TIMEOUT)
        .idle_timeout(IDLE_TIMEOUT)
        .max_lifetime(MAX_LIFETIME)
        .connect_lazy(false)
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Info)
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Info)
        .sqlx_slow_statements_logging_settings(log::LevelFilter::Warn, Duration::from_secs(2));

    if backend.kind == BackendKind::Postgres && connection_string.contains("currentSchema") {
        let params = url::parse_url_query(connection_string.as_str());
        if let Some(schema) = params.get("currentSchema") {
            info!("Using Postgres schema: {}", schema);
            opt.set_schema_search_path(schema);
        }
    }

    Database::connect(opt).await.expect("Database connection failed")
}

/// shared database connection
static SHARED_DB_CONNECTION: OnceCell<DatabaseConnection> = OnceCell::const_new();

/// shared redis connection string
/// can be changed by code, for example, when config changed
/// if changed, please call make_redis_client() to get new client
static SHARED_REDIS_CONNECT_STRING: RwLock<String> = RwLock::new(String::new());

// static SHARED_REDIS_POOL: OnceCell<deadpool_redis::Pool> = OnceCell::const_new();
