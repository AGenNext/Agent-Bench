//! CLEAR evaluation metrics — Cost, Latency, Efficacy, Assurance, Reliability.
//!
//! Implements the metric definitions distilled in
//! `benchmarks/reference/clear-enterprise-evaluation.md`.

use serde::{Deserialize, Serialize};

/// One per-task observation feeding the CLEAR aggregates.
#[derive(Debug, Clone, Copy)]
pub struct TaskObservation {
    /// Was the task solved (fraction of trials in `[0,1]`).
    pub success: f64,
    pub cost_usd: f64,
    pub within_sla: bool,
    pub policy_violation: bool,
    /// Whether this task involved a policy-critical action (denominator for PAS).
    pub policy_critical: bool,
}

/// Aggregate CLEAR scores for a single run.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ClearScores {
    /// Mean task success in `[0,1]` (efficacy).
    pub efficacy: f64,
    pub cost_usd: f64,
    /// Cost-Normalized Accuracy = accuracy / cost * 100.
    pub cna: f64,
    /// Cost Per Success = total cost / successful tasks.
    pub cps: f64,
    /// SLA Compliance Rate in `[0,1]`.
    pub scr: f64,
    /// Policy Adherence Score in `[0,1]`.
    pub pas: f64,
}

/// Compute CLEAR aggregates from per-task observations.
pub fn clear_scores(obs: &[TaskObservation]) -> ClearScores {
    if obs.is_empty() {
        return ClearScores::default();
    }
    let n = obs.len() as f64;

    let total_success: f64 = obs.iter().map(|o| o.success).sum();
    let total_cost: f64 = obs.iter().map(|o| o.cost_usd).sum();
    let efficacy = total_success / n;

    // Accuracy as a percentage for CNA, per the reference definition.
    let accuracy_pct = efficacy * 100.0;
    let cna = if total_cost > 0.0 {
        accuracy_pct / total_cost
    } else {
        0.0
    };

    let cps = if total_success > 0.0 {
        total_cost / total_success
    } else {
        f64::INFINITY
    };

    let within_sla = obs.iter().filter(|o| o.within_sla).count() as f64;
    let scr = within_sla / n;

    let policy_critical = obs.iter().filter(|o| o.policy_critical).count() as f64;
    let violations = obs.iter().filter(|o| o.policy_violation).count() as f64;
    let pas = if policy_critical > 0.0 {
        1.0 - violations / policy_critical
    } else {
        1.0
    };

    ClearScores {
        efficacy,
        cost_usd: total_cost,
        cna,
        cps,
        scr,
        pas,
    }
}

/// pass@k — fraction of trials achieving k consecutive successes.
///
/// `trial_successes` is the ordered per-trial outcome (true = solved).
pub fn pass_at_k(trial_successes: &[bool], k: usize) -> f64 {
    if k == 0 || trial_successes.len() < k {
        return 0.0;
    }
    let windows = trial_successes.len() - k + 1;
    let hits = (0..windows)
        .filter(|&i| trial_successes[i..i + k].iter().all(|&s| s))
        .count();
    hits as f64 / windows as f64
}

/// Weights for the composite CLEAR score. Must sum to 1.0 (validated on build).
#[derive(Debug, Clone, Copy)]
pub struct ClearWeights {
    pub cost: f64,
    pub latency: f64,
    pub efficacy: f64,
    pub assurance: f64,
    pub reliability: f64,
}

impl Default for ClearWeights {
    fn default() -> Self {
        // Equal weighting (w_i = 0.2) per the reference default.
        Self {
            cost: 0.2,
            latency: 0.2,
            efficacy: 0.2,
            assurance: 0.2,
            reliability: 0.2,
        }
    }
}

/// Composite CLEAR score: weighted sum of normalized dimensions in `[0,1]`.
///
/// `cost_norm` and `latency_norm` are expected pre-normalized to `[0,1]`
/// (min-max across the cohort) where higher = better.
pub fn clear_composite(
    w: ClearWeights,
    cost_norm: f64,
    latency_norm: f64,
    efficacy: f64,
    assurance: f64,
    reliability: f64,
) -> f64 {
    w.cost * cost_norm
        + w.latency * latency_norm
        + w.efficacy * efficacy
        + w.assurance * assurance
        + w.reliability * reliability
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obs(success: f64, cost: f64, sla: bool, viol: bool, crit: bool) -> TaskObservation {
        TaskObservation {
            success,
            cost_usd: cost,
            within_sla: sla,
            policy_violation: viol,
            policy_critical: crit,
        }
    }

    #[test]
    fn clear_basic() {
        let data = vec![
            obs(1.0, 1.0, true, false, true),
            obs(0.0, 1.0, false, true, true),
        ];
        let s = clear_scores(&data);
        assert!((s.efficacy - 0.5).abs() < 1e-9);
        assert!((s.cost_usd - 2.0).abs() < 1e-9);
        // accuracy 50% / $2 = 25.0
        assert!((s.cna - 25.0).abs() < 1e-9);
        // $2 total / 1 success = 2.0
        assert!((s.cps - 2.0).abs() < 1e-9);
        // 1 of 2 within SLA
        assert!((s.scr - 0.5).abs() < 1e-9);
        // 1 violation / 2 critical -> 0.5
        assert!((s.pas - 0.5).abs() < 1e-9);
    }

    #[test]
    fn pas_no_critical_actions_is_perfect() {
        let data = vec![obs(1.0, 0.5, true, false, false)];
        assert!((clear_scores(&data).pas - 1.0).abs() < 1e-9);
    }

    #[test]
    fn pass_at_k_consecutive() {
        assert!((pass_at_k(&[true, true, true], 3) - 1.0).abs() < 1e-9);
        assert!((pass_at_k(&[true, false, true, true], 2) - (1.0 / 3.0)).abs() < 1e-9);
        assert_eq!(pass_at_k(&[true], 3), 0.0);
    }

    #[test]
    fn composite_equal_weights() {
        let c = clear_composite(ClearWeights::default(), 1.0, 1.0, 1.0, 1.0, 1.0);
        assert!((c - 1.0).abs() < 1e-9);
    }
}
