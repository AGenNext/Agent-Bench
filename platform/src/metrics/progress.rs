//! Fine-grained progress-rate metric (AgentBoard).
//!
//! Implements the methods distilled in `benchmarks/reference/agentboard.md`:
//! progress rate is the highest matching score toward the goal achieved so far,
//! capturing incremental advancement where binary success rate is uninformative.

/// Progress rate from a continuous matching score `f(state, goal) -> [0,1]`,
/// evaluated over the trajectory of visited states. Returns the maximum
/// (monotone non-decreasing) score, clamped to `[0,1]`.
pub fn progress_rate_continuous(matching_scores: &[f64]) -> f64 {
    matching_scores
        .iter()
        .copied()
        .fold(0.0_f64, f64::max)
        .clamp(0.0, 1.0)
}

/// Subgoal-based progress rate: fraction of the `k` ordered subgoals completed.
/// `completed` is the count of satisfied subgoals; `k` is the total.
pub fn progress_rate_subgoal(completed: usize, k: usize) -> f64 {
    if k == 0 {
        return 0.0;
    }
    (completed as f64 / k as f64).clamp(0.0, 1.0)
}

/// Success rate across a set of task outcomes (proportion fully completed).
pub fn success_rate(outcomes: &[bool]) -> f64 {
    if outcomes.is_empty() {
        return 0.0;
    }
    outcomes.iter().filter(|&&o| o).count() as f64 / outcomes.len() as f64
}

/// Grounding accuracy: fraction of issued actions that were valid.
pub fn grounding_accuracy(valid_actions: usize, total_actions: usize) -> f64 {
    if total_actions == 0 {
        return 0.0;
    }
    valid_actions as f64 / total_actions as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_takes_max() {
        assert!((progress_rate_continuous(&[0.0, 0.25, 0.1, 0.5]) - 0.5).abs() < 1e-9);
        assert_eq!(progress_rate_continuous(&[]), 0.0);
    }

    #[test]
    fn subgoal_fraction() {
        assert!((progress_rate_subgoal(2, 4) - 0.5).abs() < 1e-9);
        assert_eq!(progress_rate_subgoal(0, 0), 0.0);
    }

    #[test]
    fn rates() {
        assert!((success_rate(&[true, false, true, true]) - 0.75).abs() < 1e-9);
        assert!((grounding_accuracy(58, 100) - 0.58).abs() < 1e-9);
    }
}
