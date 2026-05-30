//! HTTP API: multi-tenant agent submission, run scoring, and leaderboards.

use std::collections::HashMap;

use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};

use crate::db::Store;
use crate::domain::{Agent, Benchmark, LeaderboardEntry, Run, SubmitRun};
use crate::error::AppResult;
use crate::tenancy::Tenant;

/// Build the application router.
pub fn router(store: Store) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/agents", post(create_agent).get(list_agents))
        .route("/v1/benchmarks", post(upsert_benchmark).get(list_benchmarks))
        .route("/v1/runs", post(submit_run))
        .route("/v1/leaderboard/:benchmark_id", get(leaderboard))
        .with_state(store)
}

async fn health() -> &'static str {
    "ok"
}

async fn create_agent(
    State(store): State<Store>,
    tenant: Tenant,
    Json(agent): Json<Agent>,
) -> AppResult<Json<Agent>> {
    Ok(Json(store.create_agent(&tenant.0, agent).await?))
}

async fn list_agents(
    State(store): State<Store>,
    tenant: Tenant,
) -> AppResult<Json<Vec<Agent>>> {
    Ok(Json(store.list_agents(&tenant.0).await?))
}

async fn upsert_benchmark(
    State(store): State<Store>,
    tenant: Tenant,
    Json(b): Json<Benchmark>,
) -> AppResult<Json<Benchmark>> {
    Ok(Json(store.upsert_benchmark(&tenant.0, b).await?))
}

async fn list_benchmarks(
    State(store): State<Store>,
    tenant: Tenant,
) -> AppResult<Json<Vec<Benchmark>>> {
    Ok(Json(store.list_benchmarks(&tenant.0).await?))
}

async fn submit_run(
    State(store): State<Store>,
    tenant: Tenant,
    Json(req): Json<SubmitRun>,
) -> AppResult<Json<Run>> {
    Ok(Json(store.submit_run(&tenant.0, req).await?))
}

async fn leaderboard(
    State(store): State<Store>,
    tenant: Tenant,
    Path(benchmark_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> AppResult<Json<Vec<LeaderboardEntry>>> {
    // Optional ?hardware=gpu-a100 slices the board to one backend.
    let hardware = params.get("hardware").map(String::as_str);
    Ok(Json(
        store.leaderboard(&tenant.0, &benchmark_id, hardware).await?,
    ))
}
