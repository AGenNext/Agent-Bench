//! Verifies migration 0004 collapses the live store onto one coherent model:
//! a freshly-migrated tenant has the generic metamodel + orchestrator +
//! result-package tables (with their seed data), migration 0004 is recorded as
//! applied, and the existing run/leaderboard service path still works unbroken.

#![cfg(feature = "server")]

use agentbench_platform::db::Store;
use agentbench_platform::domain::{Agent, Benchmark, SubmitRun, SubmittedMetric};
use agentbench_platform::evaluation::{MetricDirection, Threshold};

#[tokio::test]
async fn collapse_migration_applies_and_installs_generic_model() {
    let store = Store::memory().await.expect("open store");
    let tenant = "acme";

    // 0004 is wired into the migration chain and applied on first tenant touch.
    // (enter_tenant now applies migrations with .check(), so a broken 0004 would
    // fail loudly here rather than be silently recorded.)
    let applied = store.applied_migrations(tenant).await.unwrap();
    assert!(
        applied.contains(&"0004_collapse_generic".to_string()),
        "0004 must be applied: {applied:?}"
    );

    // Orchestrator + generic seed data is present — proving the big DDL block ran.
    assert_eq!(store.table_count(tenant, "workload_kind").await.unwrap(), 5);
    assert_eq!(
        store
            .table_count(tenant, "conformance_check")
            .await
            .unwrap(),
        7
    );
    assert_eq!(store.table_count(tenant, "level_scheme").await.unwrap(), 9);

    // The new tables are live and queryable (empty until used).
    for t in [
        "entity",
        "workload",
        "cluster",
        "result_package",
        "repro_manifest",
    ] {
        assert_eq!(
            store.table_count(tenant, t).await.unwrap(),
            0,
            "`{t}` should exist and be empty on a fresh tenant"
        );
    }
}

#[tokio::test]
async fn service_path_survives_collapse() {
    // The legacy agent/benchmark/run/leaderboard HTTP contract must keep working
    // against the collapsed schema (new run/benchmark fields are optional).
    let store = Store::memory().await.unwrap();
    let tenant = "acme";

    store
        .upsert_benchmark(
            tenant,
            Benchmark {
                id: None,
                benchmark_id: "swe-lite".into(),
                name: "SWE Lite".into(),
                domain: "software".into(),
                task_count: 3,
            },
        )
        .await
        .unwrap();

    let agent = store
        .create_agent(
            tenant,
            Agent {
                id: None,
                name: "Strong".into(),
                scaffold: "plan-execute".into(),
                model: "opus".into(),
                version: "1".into(),
            },
        )
        .await
        .unwrap();

    store
        .submit_run(
            tenant,
            SubmitRun {
                agent_id: agent.id.clone().unwrap(),
                benchmark_id: "swe-lite".into(),
                attribute: "trajectory".into(),
                protocol: "TRAJ-001@0.1.0".into(),
                hardware: "gpu-a100".into(),
                dsl: String::new(),
                trials: 1,
                metrics: vec![SubmittedMetric {
                    key: "efficacy".into(),
                    direction: MetricDirection::HigherIsBetter,
                    value: 0.95,
                    normalized_score: Some(0.95),
                    threshold: Some(Threshold::Gte(0.70)),
                    weight: None,
                }],
            },
        )
        .await
        .expect("submit_run still works after collapse");

    let board = store.leaderboard(tenant, "swe-lite", None).await.unwrap();
    assert_eq!(board.len(), 1);
    assert_eq!(board[0].agent_name, "Strong");
    assert!(board[0].passed);
}
