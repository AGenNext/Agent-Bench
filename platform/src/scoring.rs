//! Bridges raw per-task results to aggregate run scores and improvement areas.
//! Pure Rust — no DB — so it is reusable by the API, the CLI, and tests.

use crate::domain::{RunScores, TaskResult};
use crate::metrics::clear::{clear_composite, clear_scores, ClearWeights, TaskObservation};
use crate::metrics::perf::{perf_scores, PerfObservation};
use crate::metrics::progress::progress_rate_continuous;

/// Compute aggregate CLEAR + progress scores for a run from its task results.
///
/// `cost_norm` / `latency_norm` are the cohort-normalized (higher = better)
/// inputs to the composite; for a standalone run we pass neutral 0.5 values.
pub fn score_run(results: &[TaskResult], weights: ClearWeights) -> RunScores {
    let obs: Vec<TaskObservation> = results
        .iter()
        .map(|r| TaskObservation {
            success: r.success,
            cost_usd: r.cost_usd,
            within_sla: r.within_sla,
            policy_violation: r.policy_violation,
            policy_critical: r.policy_critical,
        })
        .collect();

    let clear = clear_scores(&obs);

    // Performance metrics (kernel/codegen): only meaningful when baselines given.
    let perf_obs: Vec<PerfObservation> = results
        .iter()
        .map(|r| PerfObservation {
            correct: r.correct,
            baseline_latency_ms: r.baseline_latency_ms,
            kernel_latency_ms: r.latency_ms,
        })
        .collect();
    let perf = perf_scores(&perf_obs);

    // Progress rate: mean of per-task progress rates (each already a max-so-far).
    let progress_rate = if results.is_empty() {
        0.0
    } else {
        results.iter().map(|r| progress_rate_continuous(&[r.progress_rate])).sum::<f64>()
            / results.len() as f64
    };

    // pass@k proxy at run level: treat efficacy as single-trial reliability.
    let pass_at_k = clear.efficacy;

    let clear_composite = clear_composite(
        weights,
        // Neutral normalization for a standalone run; the leaderboard re-normalizes
        // cost/latency across the cohort.
        0.5,
        clear.scr,
        clear.efficacy,
        clear.pas,
        pass_at_k,
    );

    RunScores {
        clear,
        progress_rate,
        pass_at_k,
        clear_composite,
        perf,
    }
}

/// Identify an agent's weakest CLEAR dimensions (improvement areas), ordered
/// worst-first. A dimension is flagged when it falls below `threshold`.
pub fn improvement_areas(scores: &RunScores, threshold: f64) -> Vec<String> {
    let mut dims: Vec<(&str, f64)> = vec![
        ("efficacy", scores.clear.efficacy),
        ("reliability", scores.pass_at_k),
        ("assurance", scores.clear.pas),
        ("sla_compliance", scores.clear.scr),
        ("progress", scores.progress_rate),
    ];
    dims.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    dims.into_iter()
        .filter(|(_, v)| *v < threshold)
        .map(|(name, _)| name.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tr(task: &str, success: f64, cost: f64, sla: bool, viol: bool) -> TaskResult {
        TaskResult {
            task_id: task.into(),
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

    #[test]
    fn scores_a_run() {
        let results = vec![
            tr("t1", 1.0, 1.0, true, false),
            tr("t2", 0.0, 1.0, false, true),
        ];
        let s = score_run(&results, ClearWeights::default());
        assert!((s.clear.efficacy - 0.5).abs() < 1e-9);
        assert!((s.clear.cna - 25.0).abs() < 1e-9);
        assert!(s.clear_composite > 0.0 && s.clear_composite <= 1.0);
    }

    #[test]
    fn flags_weak_dimensions() {
        let results = vec![tr("t1", 0.2, 5.0, false, true)];
        let s = score_run(&results, ClearWeights::default());
        let areas = improvement_areas(&s, 0.8);
        assert!(areas.contains(&"efficacy".to_string()));
        assert!(areas.contains(&"assurance".to_string()));
    }
}
