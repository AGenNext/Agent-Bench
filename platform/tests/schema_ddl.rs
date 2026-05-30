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
async fn trajectory_thresholds_seeded_and_step_efficiency() {
    let db = fresh().await;
    db.query(SCHEMA_TRAJECTORY).await.unwrap().check().unwrap();
    let tca: Option<f64> = db
        .query("SELECT VALUE tool_call_accuracy FROM trajectory_thresholds:`TRAJ-001@0.1.0`")
        .await.unwrap().take(0).unwrap();
    assert!((tca.unwrap() - 0.80).abs() < 1e-9);
    // step_efficiency caps at 1.0 when actual < optimal.
    let se: Option<f64> = db.query("RETURN fn::step_efficiency(8, 4)").await.unwrap().take(0).unwrap();
    assert!((se.unwrap() - 1.0).abs() < 1e-9);
}

#[tokio::test]
async fn metric_functions_compute_correctly() {
    let db = fresh().await;
    db.query(SCHEMA_METRICS).await.unwrap().check().unwrap();

    // Each formula must agree with the Rust reference.
    // CNA: 50% accuracy / $2 -> 25.0
    let cna: Option<f64> = db.query("RETURN fn::cna(0.5, 2.0)").await.unwrap().take(0).unwrap();
    assert!((cna.unwrap() - 25.0).abs() < 1e-9);

    // PAS: 1 violation / 2 critical -> 0.5
    let pas: Option<f64> = db.query("RETURN fn::pas(1, 2)").await.unwrap().take(0).unwrap();
    assert!((pas.unwrap() - 0.5).abs() < 1e-9);

    // Geomean of [2,8] -> 4
    let gm: Option<f64> = db.query("RETURN fn::geomean([2.0, 8.0])").await.unwrap().take(0).unwrap();
    assert!((gm.unwrap() - 4.0).abs() < 1e-9);

    // Speedup 10/5 -> 2
    let sp: Option<f64> = db.query("RETURN fn::speedup(10.0, 5.0)").await.unwrap().take(0).unwrap();
    assert!((sp.unwrap() - 2.0).abs() < 1e-9);

    // Reduction ratio 4 total, 2 selected -> 0.5
    let rr: Option<f64> = db.query("RETURN fn::reduction_ratio(4, 2)").await.unwrap().take(0).unwrap();
    assert!((rr.unwrap() - 0.5).abs() < 1e-9);

    // Progress over [0,0.25,0.1,0.5] -> 0.5
    let pr: Option<f64> = db
        .query("RETURN fn::progress_continuous([0.0, 0.25, 0.1, 0.5])")
        .await.unwrap().take(0).unwrap();
    assert!((pr.unwrap() - 0.5).abs() < 1e-9);
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
