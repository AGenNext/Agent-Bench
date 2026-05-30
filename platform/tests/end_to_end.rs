//! End-to-end test of the multi-tenant store: submit runs for two agents on a
//! benchmark and verify the leaderboard ranks them and flags improvement areas.
//! Exercises embedded SurrealDB, migrations, namespaces, and the scoring engine.

#![cfg(feature = "server")]

use agentbench_platform::db::Store;
use agentbench_platform::domain::{Agent, Benchmark, SubmitRun, TaskResult};

fn task(id: &str, success: f64, cost: f64, sla: bool, viol: bool) -> TaskResult {
    TaskResult {
        task_id: id.into(),
        success,
        progress_rate: success,
        cost_usd: cost,
        latency_ms: 100.0,
        within_sla: sla,
        policy_violation: viol,
        policy_critical: true,
        correct: success >= 0.5,
        baseline_latency_ms: 0.0,
    }
}

/// A kernel-style perf task: correct + a baseline latency to compute speedup.
fn perf_task(id: &str, correct: bool, baseline_ms: f64, kernel_ms: f64) -> TaskResult {
    TaskResult {
        task_id: id.into(),
        success: if correct { 1.0 } else { 0.0 },
        progress_rate: 1.0,
        cost_usd: 0.1,
        latency_ms: kernel_ms,
        within_sla: true,
        policy_violation: false,
        policy_critical: false,
        correct,
        baseline_latency_ms: baseline_ms,
    }
}

#[tokio::test]
async fn submit_and_rank() {
    let store = Store::memory().await.expect("open store");
    let tenant = "acme";

    // Two enterprises' namespaces are isolated; we use one here.
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

    // Strong agent: high success, cheap, compliant.
    store
        .submit_run(
            tenant,
            SubmitRun {
                agent_id: strong.id.clone().unwrap(),
                benchmark_id: "swe-lite".into(),
                hardware: "gpu-a100".into(),
                dsl: String::new(),
                trials: 1,
                results: vec![
                    task("t1", 1.0, 0.2, true, false),
                    task("t2", 1.0, 0.2, true, false),
                    task("t3", 0.8, 0.2, true, false),
                ],
            },
        )
        .await
        .unwrap();

    // Weak agent: low success, expensive, policy violations.
    store
        .submit_run(
            tenant,
            SubmitRun {
                agent_id: weak.id.clone().unwrap(),
                benchmark_id: "swe-lite".into(),
                hardware: "gpu-a100".into(),
                dsl: String::new(),
                trials: 1,
                results: vec![
                    task("t1", 0.2, 2.0, false, true),
                    task("t2", 0.0, 2.0, false, true),
                    task("t3", 0.3, 2.0, true, false),
                ],
            },
        )
        .await
        .unwrap();

    let board = store
        .leaderboard(tenant, "swe-lite", None)
        .await
        .unwrap();
    assert_eq!(board.len(), 2, "two agents on the board");

    // Strong agent ranks first.
    assert_eq!(board[0].rank, 1);
    assert_eq!(board[0].agent_name, "Strong");
    assert_eq!(board[1].agent_name, "Weak");
    assert!(board[0].clear_composite > board[1].clear_composite);

    // The weak agent should have improvement areas flagged; the strong one fewer.
    assert!(
        board[1].improvement_areas.contains(&"efficacy".to_string()),
        "weak agent flagged on efficacy: {:?}",
        board[1].improvement_areas
    );
    assert!(board[0].improvement_areas.len() <= board[1].improvement_areas.len());
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
async fn multi_hardware_kernel_leaderboard() {
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

    // Same agent, two hardware backends: fast on GPU (4x), slow on NPU (~1x).
    store
        .submit_run(
            tenant,
            SubmitRun {
                agent_id: agent.id.clone().unwrap(),
                benchmark_id: "kernelbench".into(),
                hardware: "gpu-a100".into(),
                dsl: "triton".into(),
                trials: 1,
                results: vec![
                    perf_task("matmul", true, 10.0, 5.0),
                    perf_task("attn", true, 16.0, 2.0),
                ],
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
                hardware: "npu".into(),
                dsl: "triton".into(),
                trials: 1,
                results: vec![
                    perf_task("matmul", true, 10.0, 10.0),
                    perf_task("attn", false, 16.0, 4.0),
                ],
            },
        )
        .await
        .unwrap();

    // GPU board: correct + ~4x geomean speedup.
    let gpu = store
        .leaderboard(tenant, "kernelbench", Some("gpu-a100"))
        .await
        .unwrap();
    assert_eq!(gpu.len(), 1);
    assert_eq!(gpu[0].hardware, "gpu-a100");
    assert!((gpu[0].correctness - 1.0).abs() < 1e-9);
    assert!((gpu[0].speedup_geomean - 4.0).abs() < 1e-6, "gpu geomean {}", gpu[0].speedup_geomean);

    // NPU board for the SAME agent is worse — rankings are hardware-specific.
    let npu = store
        .leaderboard(tenant, "kernelbench", Some("npu"))
        .await
        .unwrap();
    assert_eq!(npu.len(), 1);
    assert!(npu[0].correctness < gpu[0].correctness);
    assert!(npu[0].speedup_geomean < gpu[0].speedup_geomean);

    // Unsliced board sees both runs.
    let all = store.leaderboard(tenant, "kernelbench", None).await.unwrap();
    assert_eq!(all.len(), 2);
}
