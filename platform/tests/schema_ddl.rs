//! Verifies the DDL/DSL definitions actually execute in the SurrealDB engine —
//! so the definitions are correct, not loosely written prose. If a `DEFINE`
//! statement or function formula is invalid, this test fails.

#![cfg(feature = "server")]

use surrealdb::engine::any::connect;

const SCHEMA_METRICS: &str = include_str!("../schema/metrics.surql");
const SCHEMA_MEMORY: &str = include_str!("../schema/memory_attribute.surql");
const SCHEMA_PAGES: &str = include_str!("../schema/pages.surql");
const LOGIC_REFERENCE_MD: &str = include_str!("../schema/pages/agent-bench-logic-reference.md");

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
async fn logic_reference_page_stored_and_served_from_surrealdb() {
    let db = fresh().await;
    db.query(SCHEMA_PAGES).await.unwrap().check().unwrap();

    // Store the page document in SurrealDB (multimodel).
    db.query(
        "UPSERT type::thing('page', $slug) SET slug = $slug, title = $title, \
         format = $format, content = $content",
    )
    .bind(("slug", "agent-bench-logic-reference"))
    .bind(("title", "Agent-Bench — Logic Reference"))
    .bind(("format", "markdown"))
    .bind(("content", LOGIC_REFERENCE_MD))
    .await
    .unwrap()
    .check()
    .unwrap();

    // Served back by the same query the SurrealDB HTTP API would run.
    let title: Option<String> = db
        .query("SELECT VALUE title FROM page WHERE slug = $slug")
        .bind(("slug", "agent-bench-logic-reference"))
        .await
        .unwrap()
        .take(0)
        .unwrap();
    assert_eq!(title.as_deref(), Some("Agent-Bench — Logic Reference"));

    let content: Option<String> = db
        .query("SELECT VALUE content FROM page:`agent-bench-logic-reference`")
        .await
        .unwrap()
        .take(0)
        .unwrap();
    assert!(content.unwrap().contains("How good is this agent"));
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
