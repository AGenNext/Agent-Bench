//! LLM-as-a-judge interface (scaffold).
//!
//! The article's judge paradigms: **pointwise** (score one output against
//! criteria) and **pairwise** (compare two). A real judge wraps an LLM at
//! runtime — which this sandbox cannot call (network-blocked) — so the live
//! impl belongs in the runtime. Here we define the *interface* and a
//! deterministic, dependency-free stub so callers can be written and tested.

use std::cmp::Ordering;

/// A judge scores agent outputs against natural-language criteria.
pub trait Judge {
    /// Score one output against `criteria`, in `[0,1]`.
    fn pointwise(&self, output: &str, criteria: &str) -> f64;

    /// Compare two outputs against `criteria` (which is better).
    fn pairwise(&self, a: &str, b: &str, criteria: &str) -> Ordering;
}

/// Deterministic, no-LLM stub. **Not** a real judge — it lets code that depends
/// on a `Judge` be written and tested offline; the runtime supplies a real
/// LLM-backed impl. Scoring here is a transparent length/keyword heuristic, not
/// a quality judgement.
pub struct DeterministicJudge;

impl Judge for DeterministicJudge {
    fn pointwise(&self, output: &str, criteria: &str) -> f64 {
        if output.trim().is_empty() {
            return 0.0;
        }
        // Fraction of criteria keywords present — transparent, reproducible.
        let keys: Vec<&str> = criteria.split_whitespace().filter(|w| w.len() > 3).collect();
        if keys.is_empty() {
            return 0.5;
        }
        let lower = output.to_lowercase();
        let hits = keys.iter().filter(|k| lower.contains(&k.to_lowercase())).count();
        (hits as f64 / keys.len() as f64).clamp(0.0, 1.0)
    }

    fn pairwise(&self, a: &str, b: &str, criteria: &str) -> Ordering {
        self.pointwise(a, criteria)
            .partial_cmp(&self.pointwise(b, criteria))
            .unwrap_or(Ordering::Equal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pointwise_scores_keyword_coverage() {
        let j = DeterministicJudge;
        assert_eq!(j.pointwise("", "anything"), 0.0);
        // 2 of 2 criteria keywords present.
        assert!((j.pointwise("clear and correct", "clear correct") - 1.0).abs() < 1e-9);
    }

    #[test]
    fn pairwise_picks_better_coverage() {
        let j = DeterministicJudge;
        assert_eq!(j.pairwise("clear correct answer", "vague", "clear correct"), Ordering::Greater);
    }
}
