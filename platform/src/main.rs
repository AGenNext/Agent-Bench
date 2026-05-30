//! Entry point for the Agent-Bench evaluation platform server.

use std::net::SocketAddr;

use agentbench_platform::api;
use agentbench_platform::db::Store;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,agentbench_platform=debug".into()),
        )
        .init();

    // Storage selection (in priority order):
    //   AGENTBENCH_DB_URL  — any endpoint: memory | surrealkv://path | ws://host:8000
    //   AGENTBENCH_DB_PATH — shorthand for surrealkv://<path>
    //   (neither)          — in-memory
    // Remote (ws/http) endpoints use AGENTBENCH_DB_USER / AGENTBENCH_DB_PASS.
    let store = if let Ok(url) = std::env::var("AGENTBENCH_DB_URL") {
        let user = std::env::var("AGENTBENCH_DB_USER").unwrap_or_else(|_| "root".into());
        let pass = std::env::var("AGENTBENCH_DB_PASS").unwrap_or_else(|_| "root".into());
        tracing::info!(%url, "connecting to SurrealDB");
        Store::connect(&url, Some((&user, &pass))).await?
    } else if let Ok(path) = std::env::var("AGENTBENCH_DB_PATH") {
        tracing::info!(%path, "opening SurrealKV store");
        Store::surrealkv(&path).await?
    } else {
        tracing::info!("opening in-memory store (set AGENTBENCH_DB_URL to use a server)");
        Store::memory().await?
    };

    let app = api::router(store);

    let addr: SocketAddr = std::env::var("AGENTBENCH_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8080".into())
        .parse()?;
    tracing::info!(%addr, "agentbench-platform listening");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("shutting down");
}
