//! Verifies the DDL/DSL definitions actually execute in the SurrealDB engine —
//! so the definitions are correct, not loosely written prose. If a `DEFINE`
//! statement or function formula is invalid, this test fails.

#![cfg(feature = "server")]

use surrealdb::engine::any::connect;

const SCHEMA_METRICS: &str = include_str!("../schema/metrics.surql");
const SCHEMA_MEMORY: &str = include_str!("../schema/memory_attribute.surql");
const SCHEMA_TRAJECTORY: &str = include_str!("../schema/trajectory_attribute.surql");

async fn fresh() -> surrealdb::Surreal<surrealdb::engine::any::Any> {
    let db = connect("memory").await.expect("embedded surreal");
    db.use_ns("test").use_db("test").await.unwrap();
    db
}

#[tokio::test]
async fn schema_files_execute() {
    let db = fresh().await;
    db.query(SCHEMA_METRICS).await.expect("metrics.surql executes").check().expect("no errors");
    db.query(SCHEMA_MEMORY).await.expect("memory_attribute.surql executes").check().expect("no errors");
    db.query(SCHEMA_TRAJECTORY).await.expect("trajectory_attribute.surql executes").check().expect("no errors");
}

#[tokio::test]
async fn trajectory_thresholds_are_seeded() {
    let db = fresh().await;
    db.query(SCHEMA_TRAJECTORY).await.unwrap().check().unwrap();
    let tca: Option<f64> = db
        .query("SELECT VALUE tool_call_accuracy FROM trajectory_thresholds:`TRAJ-001@0.1.0`")
        .await.unwrap().take(0).unwrap();
    assert!((tca.unwrap() - 0.80).abs() < 1e-9);
}

#[tokio::test]
async fn metrics_are_referenced_not_defined() {
    // Bench does not define metric formulas. metrics.surql is a reference
    // manifest pointing at Agent-Metrics; assert it loads and resolves ids.
    let db = fresh().await;
    db.query(SCHEMA_METRICS).await.unwrap().check().unwrap();

    // Every referenced metric carries a canonical Agent-Metrics id.
    let refs: Vec<String> = db
        .query("SELECT VALUE ref FROM metric_ref WHERE ref = NONE")
        .await.unwrap().take(0).unwrap();
    assert!(refs.is_empty(), "every metric_ref must carry an Agent-Metrics id");

    // CLEAR dimensions resolve to Agent-Metrics, not to a local formula.
    let cna: Option<String> = db
        .query("SELECT VALUE ref FROM metric_ref:cna")
        .await.unwrap().take(0).unwrap();
    assert_eq!(cna.as_deref(), Some("agent-metrics:cna@1.0.0"));

    // No formula functions are defined by Bench (would error if called).
    let called: Result<surrealdb::Response, _> = db.query("RETURN fn::cna(0.5, 2.0)").await;
    assert!(
        called.is_err() || called.unwrap().check().is_err(),
        "Bench must not define fn::cna — it lives in Agent-Metrics"
    );
}

#[tokio::test]
async fn amb_001_thresholds_are_seeded() {
    let db = fresh().await;
    db.query(SCHEMA_MEMORY).await.unwrap().check().unwrap();
    let recall: Option<f64> = db
        .query("SELECT VALUE recall_accuracy FROM memory_thresholds:`AMB-001@0.1.0`")
        .await.unwrap().take(0).unwrap();
    assert!((recall.unwrap() - 0.70).abs() < 1e-9);
}
