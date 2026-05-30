//! End-to-end test of the multi-tenant store: submit runs (measured metric
//! values + protocol thresholds) for two agents on a benchmark and verify the
//! leaderboard ranks them on the generic attribute grade and flags improvement
//! areas. Exercises embedded SurrealDB, migrations, namespaces, and the generic
//! metamodel — Bench computes no metric formula.

#![cfg(feature = "server")]

use agentbench_platform::db::Store;
use agentbench_platform::domain::{Agent, Benchmark, SubmitRun, SubmittedMetric};
use agentbench_platform::evaluation::{MetricDirection, Threshold};

/// A measured metric value with its protocol pass threshold. `value` doubles as
/// the normalized [0,1] score so the weighted grade differentiates agents.
fn metric(key: &str, value: f64, threshold: f64) -> SubmittedMetric {
    SubmittedMetric {
        key: key.into(),
        direction: MetricDirection::HigherIsBetter,
        value,
        normalized_score: Some(value),
        threshold: Some(Threshold::Gte(threshold)),
        weight: None,
    }
}

#[tokio::test]
async fn submit_and_rank() {
    let store = Store::memory().await.expect("open store");
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

    let strong = store
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

    let weak = store
        .create_agent(
            tenant,
            Agent {
                id: None,
                name: "Weak".into(),
                scaffold: "react".into(),
                model: "small".into(),
                version: "1".into(),
            },
        )
        .await
        .unwrap();

    // Strong agent: all metrics above threshold.
    store
        .submit_run(
            tenant,
            SubmitRun {
                agent_id: strong.id.clone().unwrap(),
                benchmark_id: "swe-lite".into(),
                attribute: "trajectory".into(),
                protocol: "TRAJ-001@0.1.0".into(),
                hardware: "gpu-a100".into(),
                dsl: String::new(),
                trials: 1,
                metrics: vec![
                    metric("efficacy", 0.95, 0.70),
                    metric("reliability", 0.90, 0.70),
                    metric("assurance", 1.00, 0.70),
                ],
            },
        )
        .await
        .unwrap();

    // Weak agent: most metrics below threshold.
    store
        .submit_run(
            tenant,
            SubmitRun {
                agent_id: weak.id.clone().unwrap(),
                benchmark_id: "swe-lite".into(),
                attribute: "trajectory".into(),
                protocol: "TRAJ-001@0.1.0".into(),
                hardware: "gpu-a100".into(),
                dsl: String::new(),
                trials: 1,
                metrics: vec![
                    metric("efficacy", 0.20, 0.70),
                    metric("reliability", 0.00, 0.70),
                    metric("assurance", 0.80, 0.70),
                ],
            },
        )
        .await
        .unwrap();

    let board = store.leaderboard(tenant, "swe-lite", None).await.unwrap();
    assert_eq!(board.len(), 2, "two agents on the board");

    // Strong agent ranks first on the generic grade.
    assert_eq!(board[0].rank, 1);
    assert_eq!(board[0].agent_name, "Strong");
    assert_eq!(board[1].agent_name, "Weak");
    assert!(board[0].grade > board[1].grade);
    assert!(board[0].passed, "strong agent passed all thresholds");
    assert!(!board[1].passed, "weak agent failed thresholds");

    // The weak agent should have improvement areas flagged, worst-first.
    let weak_areas: Vec<&str> = board[1]
        .improvement_areas
        .iter()
        .map(|a| a.metric_key.as_str())
        .collect();
    assert!(
        weak_areas.contains(&"reliability") && weak_areas.contains(&"efficacy"),
        "weak agent flagged: {weak_areas:?}"
    );
    assert!(board[0].improvement_areas.len() < board[1].improvement_areas.len());
}

#[tokio::test]
async fn tenants_are_isolated() {
    let store = Store::memory().await.unwrap();
    store
        .create_agent(
            "tenant_a",
            Agent {
                id: None,
                name: "OnlyA".into(),
                scaffold: "react".into(),
                model: "m".into(),
                version: "1".into(),
            },
        )
        .await
        .unwrap();

    // tenant_b sees none of tenant_a's agents.
    let b_agents = store.list_agents("tenant_b").await.unwrap();
    assert!(b_agents.is_empty(), "namespace isolation");

    let a_agents = store.list_agents("tenant_a").await.unwrap();
    assert_eq!(a_agents.len(), 1);
    assert_eq!(a_agents[0].name, "OnlyA");
}

#[tokio::test]
async fn multi_hardware_leaderboard() {
    let store = Store::memory().await.unwrap();
    let tenant = "kernelco";
    store
        .upsert_benchmark(
            tenant,
            Benchmark {
                id: None,
                benchmark_id: "kernelbench".into(),
                name: "KernelBench".into(),
                domain: "kernel_synthesis".into(),
                task_count: 2,
            },
        )
        .await
        .unwrap();

    let agent = store
        .create_agent(
            tenant,
            Agent {
                id: None,
                name: "AKG".into(),
                scaffold: "designer-coder-verifier".into(),
                model: "m".into(),
                version: "1".into(),
            },
        )
        .await
        .unwrap();

    // Same agent, two hardware backends: strong on GPU, weak on NPU. Rankings
    // are hardware-specific, so a leaderboard is only meaningful within one.
    store
        .submit_run(
            tenant,
            SubmitRun {
                agent_id: agent.id.clone().unwrap(),
                benchmark_id: "kernelbench".into(),
                attribute: "performance".into(),
                protocol: "PERF-001@0.1.0".into(),
                hardware: "gpu-a100".into(),
                dsl: "triton".into(),
                trials: 1,
                metrics: vec![metric("correctness", 1.0, 0.5), metric("speedup", 0.95, 0.5)],
            },
        )
        .await
        .unwrap();
    store
        .submit_run(
            tenant,
            SubmitRun {
                agent_id: agent.id.clone().unwrap(),
                benchmark_id: "kernelbench".into(),
                attribute: "performance".into(),
                protocol: "PERF-001@0.1.0".into(),
                hardware: "npu".into(),
                dsl: "triton".into(),
                trials: 1,
                metrics: vec![metric("correctness", 0.5, 0.5), metric("speedup", 0.20, 0.5)],
            },
        )
        .await
        .unwrap();

    let gpu = store
        .leaderboard(tenant, "kernelbench", Some("gpu-a100"))
        .await
        .unwrap();
    assert_eq!(gpu.len(), 1);
    assert_eq!(gpu[0].hardware, "gpu-a100");
    assert!(gpu[0].passed);

    let npu = store
        .leaderboard(tenant, "kernelbench", Some("npu"))
        .await
        .unwrap();
    assert_eq!(npu.len(), 1);
    assert!(npu[0].grade < gpu[0].grade, "same agent worse on NPU");

    // Unsliced board sees both runs.
    let all = store.leaderboard(tenant, "kernelbench", None).await.unwrap();
    assert_eq!(all.len(), 2);
}
