//! Verifies the orchestrator/control-plane schema executes in the SurrealDB
//! engine on top of the metamodel, and that the seed "run a bench on a
//! real-world model" (Gemini Omni) round-trips. If any DEFINE/UPSERT/RELATE is
//! invalid, this fails — the schema is proven, not loosely written.

#![cfg(feature = "server")]
use surrealdb::engine::any::connect;

const METAMODEL: &str = include_str!("../schema/metamodel.surql");
const ORCHESTRATOR: &str = include_str!("../schema/orchestrator.surql");
const SEED_OMNI: &str = include_str!("../schema/seed/gemini-omni.surql");
const SEED_CLUSTERS: &str = include_str!("../schema/seed/clusters.surql");

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

#[tokio::test]
async fn multi_cloud_registry_is_data() {
    let db = fresh().await;
    db.query(METAMODEL).await.unwrap().check().unwrap();
    db.query(ORCHESTRATOR).await.unwrap().check().unwrap();
    db.query(SEED_CLUSTERS).await.expect("clusters.surql executes").check().expect("no errors");

    // Adding a cloud is a row, not code — providers span the self-hosted matrix.
    let providers: Vec<String> = db
        .query("SELECT VALUE provider FROM cluster ORDER BY provider").await.unwrap().take(0).unwrap();
    for p in ["aks", "eks", "gke", "kind"] {
        assert!(providers.contains(&p.to_string()), "missing provider {p}");
    }

    // A cluster's trust is the conformance checks it is proven_by (crawlable).
    let proofs: Vec<String> = db
        .query("SELECT VALUE out.key FROM proven_by WHERE in = cluster:`eks-us-east-1` ORDER BY out.key")
        .await.unwrap().take(0).unwrap();
    assert!(proofs.contains(&"pvc-bind".to_string()));
    assert!(proofs.contains(&"manifest-apply".to_string()));
}

#[tokio::test]
async fn run_carries_telemetry_and_trace() {
    let db = fresh().await;
    db.query(METAMODEL).await.unwrap().check().unwrap();
    db.query(ORCHESTRATOR).await.unwrap().check().unwrap();
    db.query(SEED_OMNI).await.unwrap().check().unwrap();

    // A reconciled run with an OpenTelemetry trace id + a span.
    db.query(
        "CREATE run:r1 SET workload=workload:`bench-gemini-omni`, phase='running', \
            attempt=1, trace_id='0af7651916cd43dd8448eb211c80319c', observation={ cross_modal_fidelity: 0.84 };
         CREATE span:sp1 SET run=run:r1, name='execute', phase='running', \
            attrs={ image_pull_ms: 1200 }, started_at=time::now();
         RELATE run:r1->traced_by->span:sp1;",
    ).await.unwrap().check().unwrap();

    // Telemetry joins by trace id.
    let tid: Option<String> = db
        .query("SELECT VALUE trace_id FROM run:r1").await.unwrap().take(0).unwrap();
    assert_eq!(tid.as_deref(), Some("0af7651916cd43dd8448eb211c80319c"));

    // Spans are queryable per run (Metrics plane).
    let span_name: Vec<String> = db
        .query("SELECT VALUE name FROM span WHERE run = run:r1").await.unwrap().take(0).unwrap();
    assert_eq!(span_name, vec!["execute".to_string()]);
}

#[tokio::test]
async fn workload_embedding_is_vector_searchable() {
    let db = fresh().await;
    db.query(METAMODEL).await.unwrap().check().unwrap();
    db.query(ORCHESTRATOR).await.unwrap().check().unwrap();
    db.query(SEED_OMNI).await.unwrap().check().unwrap();

    // Provider-supplied embedding (dim 1536) stored on the workload.
    db.query(
        "UPDATE workload:`bench-gemini-omni` SET embedding = array::repeat(0.01, 1536);",
    ).await.unwrap().check().unwrap();

    // KNN vector query resolves against the MTREE index — "find runs like this".
    let hits: Vec<String> = db
        .query(
            "LET $q = array::repeat(0.01, 1536); \
             SELECT VALUE id.id() FROM workload WHERE embedding <|1,COSINE|> $q;",
        ).await.unwrap().take(1).unwrap();
    assert!(hits.contains(&"bench-gemini-omni".to_string()));
}
