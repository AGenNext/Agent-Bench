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

    // Storage: SurrealKV on disk when AGENTBENCH_DB_PATH is set, else in-memory.
    let store = match std::env::var("AGENTBENCH_DB_PATH") {
        Ok(path) => {
            tracing::info!(%path, "opening SurrealKV store");
            Store::surrealkv(&path).await?
        }
        Err(_) => {
            tracing::info!("opening in-memory store (set AGENTBENCH_DB_PATH to persist)");
            Store::memory().await?
        }
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
