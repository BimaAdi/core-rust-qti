use std::sync::Arc;

use core_rust_qti::{core::db::init_pool, init_openapi_route, settings::get_config, AppState};
use poem::listener::TcpListener;
use tracing::Level;

#[tokio::main]
async fn main() {
    let log_level = Level::DEBUG;
    // Logging to File
    let file_appender = tracing_appender::rolling::daily("./logs", "app.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_max_level(log_level)
        .init();

    // Logging to Console
    // tracing_subscriber::fmt().with_max_level(log_level).init();

    let config = get_config();
    tracing::info!("run with config: {:?}", config);

    // Init Database Connection
    tracing::info!("Init Postgres connection on {}", config.database_url);
    let pool = init_pool(&config).await;
    // Init Redis Connection
    tracing::info!("Init Redis connection on {}", config.redis_url.clone());
    let client = redis::Client::open(config.redis_url.clone()).unwrap();
    let redis_pool = r2d2::Pool::builder().build(client).unwrap();
    // Init App State
    let app_state = Arc::new(AppState {
        db: pool,
        redis_conn: redis_pool,
    });

    let app = init_openapi_route(app_state.clone(), &config);
    tracing::info!("run server on {}:{}", config.host, config.port);
    poem::Server::new(TcpListener::bind(format!(
        "{}:{}",
        config.host, config.port
    )))
    .run(app)
    .await
    .unwrap()
}
