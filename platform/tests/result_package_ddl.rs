//! Verifies the result-package / reproducibility schema (Bench's core OUTPUT)
//! executes and round-trips: a run's score is packaged into an immutable,
//! content-addressed artifact bound to the manifest that reproduces it.
//! "If performance cannot be reproduced, it cannot be benchmarked."

#![cfg(feature = "server")]
use surrealdb::engine::any::connect;

const METAMODEL: &str = include_str!("../schema/metamodel.surql");
const ORCHESTRATOR: &str = include_str!("../schema/orchestrator.surql");
const SEED_OMNI: &str = include_str!("../schema/seed/gemini-omni.surql");
const RESULT_PACKAGE: &str = include_str!("../schema/result_package.surql");

async fn fresh() -> surrealdb::Surreal<surrealdb::engine::any::Any> {
    let db = connect("memory").await.unwrap();
    db.use_ns("t").use_db("t").await.unwrap();
    db
}

async fn loaded() -> surrealdb::Surreal<surrealdb::engine::any::Any> {
    let db = fresh().await;
    db.query(METAMODEL).await.unwrap().check().unwrap();
    db.query(ORCHESTRATOR).await.unwrap().check().unwrap();
    db.query(SEED_OMNI).await.unwrap().check().unwrap();
    db.query(RESULT_PACKAGE).await.expect("result_package.surql executes").check().expect("no errors");
    db
}

#[tokio::test]
async fn result_package_schema_executes() {
    loaded().await;
}

#[tokio::test]
async fn run_packages_into_reproducible_artifact() {
    let db = loaded().await;

    // A scored run gets packaged with the manifest that reproduces it.
    db.query(
        "CREATE attribute_score:sc SET entity=entity:`gemini-omni`, attribute=attribute:`multimodal`, \
             protocol=protocol:`OMNI-001:1`, benchmark=benchmark:omni, \
             metric_scores=[{ metric_key:'cross_modal_fidelity', value:0.84, passed:true }], \
             grade=0.84, passed=true;
         CREATE run:rp SET workload=workload:`bench-gemini-omni`, phase='succeeded', \
             score=attribute_score:sc, observation={ cross_modal_fidelity: 0.84 };
         CREATE repro_manifest:m1 SET \
             image_digest='sha256:0000000000000000000000000000000000000000000000000000000000000000', \
             protocol=protocol:`OMNI-001:1`, inputs_hash='sha256:abc', trials=3, seed=42, \
             hardware={ provider:'gke', region:'us-central1' }, evaluator='harness-multimodal@1.0';
         CREATE result_package:pkg SET subject=entity:`gemini-omni`, run=run:rp, \
             score=attribute_score:sc, manifest=repro_manifest:m1, \
             digest='sha256:1111111111111111111111111111111111111111111111111111111111111111', \
             category='artifact_generation', visibility='public', published_at=time::now();
         RELATE run:rp->packaged_as->result_package:pkg;
         RELATE result_package:pkg->reproduced_by->repro_manifest:m1;
         RELATE result_package:pkg->published->entity:`gemini-omni`;",
    ).await.unwrap().check().unwrap();

    // The package binds the score to its subject.
    let rows: Vec<serde_json::Value> = db
        .query("SELECT subject.name AS subject, score.grade AS grade, visibility FROM result_package:pkg")
        .await.unwrap().take(0).unwrap();
    assert_eq!(rows[0]["subject"], "Gemini Omni");
    assert_eq!(rows[0]["grade"], 0.84);
    assert_eq!(rows[0]["visibility"], "public");

    // Reproducibility: the manifest carries the exact inputs to replay the run.
    let repro: Vec<serde_json::Value> = db
        .query("SELECT manifest.image_digest AS image, manifest.trials AS trials, \
                manifest.seed AS seed, manifest.hardware.provider AS provider \
                FROM result_package:pkg")
        .await.unwrap().take(0).unwrap();
    assert!(repro[0]["image"].as_str().unwrap().starts_with("sha256:"));
    assert_eq!(repro[0]["trials"], 3);
    assert_eq!(repro[0]["seed"], 42);
    assert_eq!(repro[0]["provider"], "gke");

    // Graph crawl: subject -> its published packages.
    let pubd: Vec<String> = db
        .query("SELECT VALUE in.digest FROM published WHERE out = entity:`gemini-omni`")
        .await.unwrap().take(0).unwrap();
    assert!(pubd[0].starts_with("sha256:"));
}

#[tokio::test]
async fn package_digest_is_unique() {
    let db = loaded().await;
    db.query(
        "CREATE run:r1 SET workload=workload:`bench-gemini-omni`, phase='succeeded';
         CREATE repro_manifest:mm SET image_digest='sha256:aa', protocol=protocol:`OMNI-001:1`, inputs_hash='h';
         CREATE attribute_score:ss SET entity=entity:`gemini-omni`, attribute=attribute:`multimodal`, \
             protocol=protocol:`OMNI-001:1`, benchmark=benchmark:omni, metric_scores=[], grade=0.8, passed=true;
         CREATE result_package:p1 SET subject=entity:`gemini-omni`, run=run:r1, score=attribute_score:ss, \
             manifest=repro_manifest:mm, digest='sha256:dup';",
    ).await.unwrap().check().unwrap();

    // Same digest must be rejected — packages are immutable, content-addressed.
    let dup = db.query(
        "CREATE result_package:p2 SET subject=entity:`gemini-omni`, run=run:r1, score=attribute_score:ss, \
             manifest=repro_manifest:mm, digest='sha256:dup';",
    ).await.unwrap().check();
    assert!(dup.is_err(), "duplicate package digest must be rejected by the unique index");
}
