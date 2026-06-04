//! Verifies the orchestrator/control-plane schema executes in the SurrealDB
//! engine on top of the metamodel, and that the seed "run a bench on a
//! real-world model" (Gemini Omni) round-trips. If any DEFINE/UPSERT/RELATE is
//! invalid, this fails — the schema is proven, not loosely written.

#![cfg(feature = "server")]
use surrealdb::engine::any::connect;

const METAMODEL: &str = include_str!("../schema/metamodel.surql");
const ORCHESTRATOR: &str = include_str!("../schema/orchestrator.surql");
const SEED_OMNI: &str = include_str!("../schema/seed/gemini-omni.surql");

async fn fresh() -> surrealdb::Surreal<surrealdb::engine::any::Any> {
    let db = connect("memory").await.unwrap();
    db.use_ns("t").use_db("t").await.unwrap();
    db
}

#[tokio::test]
async fn orchestrator_executes_on_metamodel() {
    let db = fresh().await;
    db.query(METAMODEL).await.expect("metamodel.surql executes").check().expect("no errors");
    db.query(ORCHESTRATOR).await.expect("orchestrator.surql executes").check().expect("no errors");
}

#[tokio::test]
async fn workload_kinds_and_conformance_are_data() {
    let db = fresh().await;
    db.query(METAMODEL).await.unwrap().check().unwrap();
    db.query(ORCHESTRATOR).await.unwrap().check().unwrap();

    // "Anything" — kinds are rows, not code.
    let kinds: Vec<String> = db
        .query("SELECT VALUE key FROM workload_kind ORDER BY key").await.unwrap().take(0).unwrap();
    assert!(kinds.contains(&"bench".to_string()));
    assert!(kinds.contains(&"cluster-conformance".to_string()));

    // Conformance checks (Talos-style) are the swappable suite, also rows.
    let required: Vec<String> = db
        .query("SELECT VALUE key FROM conformance_check WHERE required = true")
        .await.unwrap().take(0).unwrap();
    assert!(required.contains(&"pvc-bind".to_string()));
    assert!(required.contains(&"oom-handling".to_string()));
}

#[tokio::test]
async fn bench_on_gemini_omni_round_trips() {
    let db = fresh().await;
    db.query(METAMODEL).await.unwrap().check().unwrap();
    db.query(ORCHESTRATOR).await.unwrap().check().unwrap();
    db.query(SEED_OMNI).await.expect("gemini-omni.surql executes").check().expect("no errors");

    // Subject exists as a metamodel entity.
    let name: Option<String> = db
        .query("SELECT VALUE name FROM entity:`gemini-omni`").await.unwrap().take(0).unwrap();
    assert_eq!(name.as_deref(), Some("Gemini Omni"));

    // Workload binds subject + protocol + image, kind=bench.
    let rows: Vec<serde_json::Value> = db
        .query("SELECT kind, subject.name AS subject, protocol.key AS protocol FROM workload:`bench-gemini-omni`")
        .await.unwrap().take(0).unwrap();
    assert_eq!(rows[0]["kind"], "bench");
    assert_eq!(rows[0]["subject"], "Gemini Omni");
    assert_eq!(rows[0]["protocol"], "OMNI-001");

    // Protocol references metrics with thresholds — no formula stored here.
    let metric_count: Option<i64> = db
        .query("SELECT VALUE array::len(metrics) FROM protocol:`OMNI-001:1`")
        .await.unwrap().take(0).unwrap();
    assert_eq!(metric_count, Some(4));

    // Graph: workload -> its kind.
    let kind: Vec<String> = db
        .query("SELECT VALUE out.key FROM has_kind WHERE in = workload:`bench-gemini-omni`")
        .await.unwrap().take(0).unwrap();
    assert_eq!(kind, vec!["bench".to_string()]);
}
