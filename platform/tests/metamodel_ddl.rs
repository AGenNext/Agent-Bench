//! Verifies the generic metamodel schema executes in the SurrealDB engine and
//! round-trips an evaluation -> outcome, matching src/evaluation.rs.

#![cfg(feature = "server")]
use surrealdb::engine::any::connect;

const METAMODEL: &str = include_str!("../schema/metamodel.surql");

async fn fresh() -> surrealdb::Surreal<surrealdb::engine::any::Any> {
    let db = connect("memory").await.unwrap();
    db.use_ns("t").use_db("t").await.unwrap();
    db
}

#[tokio::test]
async fn metamodel_executes() {
    let db = fresh().await;
    db.query(METAMODEL).await.expect("metamodel.surql executes").check().expect("no errors");
}

#[tokio::test]
async fn evaluation_round_trips() {
    let db = fresh().await;
    db.query(METAMODEL).await.unwrap().check().unwrap();

    db.query(
        "CREATE entity:agent_x SET did='did:agnext:agent/x', entity_type='agent', name='X';
         CREATE attribute:memory SET key='memory';
         CREATE protocol:amb001 SET key='AMB-001', version='0.1.0', attribute='memory';
         CREATE benchmark:memqa SET key='MemoryQA', version='1';
         CREATE evaluation:e1 SET entity=entity:agent_x, attribute=attribute:memory, \
             protocol=protocol:amb001, benchmark=benchmark:memqa, \
             conditions={ sample_size: 100, trials: 1, evaluator_version: '0.1.0' };
         CREATE attribute_score:s1 SET evaluation=evaluation:e1, entity=entity:agent_x, \
             attribute=attribute:memory, protocol=protocol:amb001, benchmark=benchmark:memqa, \
             metric_scores=[{ metric_key:'recall_accuracy', value:0.82, passed:true, weight:1.0 }], \
             grade=0.83, passed=true, improvement_areas=[];
         RELATE entity:agent_x->has_attribute->attribute:memory;
         RELATE evaluation:e1->produces->attribute_score:s1;",
    ).await.unwrap().check().unwrap();

    // Outcome is queryable and linked.
    let grade: Option<f64> = db
        .query("SELECT VALUE grade FROM attribute_score:s1").await.unwrap().take(0).unwrap();
    assert!((grade.unwrap() - 0.83).abs() < 1e-9);

    // Graph crawl: entity -> its evaluations' outcomes.
    let rows: Vec<serde_json::Value> = db
        .query("SELECT entity.name AS name, attribute.key AS attr, grade FROM attribute_score")
        .await.unwrap().take(0).unwrap();
    assert_eq!(rows[0]["attr"], "memory");
    assert_eq!(rows[0]["name"], "X");
}
