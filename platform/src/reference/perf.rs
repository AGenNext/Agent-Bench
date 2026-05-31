//! Performance metrics for kernel / code-generation benchmarks across
//! hardware backends and DSLs (KernelBench-style).
//!
//! See `benchmarks/reference/akg-kernel-agent.md`. Agents are scored on
//! correctness *and* speedup vs. a baseline, per (benchmark, hardware, dsl).

use serde::{Deserialize, Serialize};

/// One generated-artifact outcome: correctness + latency vs. a baseline.
#[derive(Debug, Clone, Copy)]
pub struct PerfObservation {
    /// Functionally correct vs. the reference (within tolerance).
    pub correct: bool,
    /// Baseline (e.g. PyTorch Eager) latency in ms.
    pub baseline_latency_ms: f64,
    /// Generated-kernel latency in ms.
    pub kernel_latency_ms: f64,
}

impl PerfObservation {
    /// Speedup vs. baseline (>1 = faster). Defined only for correct kernels with
    /// positive latency; incorrect kernels contribute no speedup.
    pub fn speedup(&self) -> Option<f64> {
        if self.correct && self.kernel_latency_ms > 0.0 && self.baseline_latency_ms > 0.0 {
            Some(self.baseline_latency_ms / self.kernel_latency_ms)
        } else {
            None
        }
    }
}

/// Aggregate performance scores for a run.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct PerfScores {
    /// Fraction of tasks functionally correct.
    pub correctness: f64,
    /// Geometric-mean speedup over correct tasks (1.0 if none).
    pub speedup_geomean: f64,
    /// fast_1: fraction correct AND at least as fast as baseline.
    pub fast_1: f64,
}

/// Correctness rate.
pub fn correctness(obs: &[PerfObservation]) -> f64 {
    if obs.is_empty() {
        return 0.0;
    }
    obs.iter().filter(|o| o.correct).count() as f64 / obs.len() as f64
}

/// Geometric mean of speedups over correct tasks: exp(mean(ln(speedup))).
pub fn speedup_geomean(obs: &[PerfObservation]) -> f64 {
    let logs: Vec<f64> = obs.iter().filter_map(|o| o.speedup()).map(f64::ln).collect();
    if logs.is_empty() {
        return 1.0;
    }
    (logs.iter().sum::<f64>() / logs.len() as f64).exp()
}

/// fast_p: fraction of all tasks that are correct AND at least `p`× faster.
pub fn fast_p(obs: &[PerfObservation], p: f64) -> f64 {
    if obs.is_empty() {
        return 0.0;
    }
    let hits = obs
        .iter()
        .filter(|o| o.speedup().map(|s| s >= p).unwrap_or(false))
        .count();
    hits as f64 / obs.len() as f64
}

/// Compute all performance aggregates for a run.
pub fn perf_scores(obs: &[PerfObservation]) -> PerfScores {
    PerfScores {
        correctness: correctness(obs),
        speedup_geomean: speedup_geomean(obs),
        fast_1: fast_p(obs, 1.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn o(correct: bool, base: f64, kernel: f64) -> PerfObservation {
        PerfObservation { correct, baseline_latency_ms: base, kernel_latency_ms: kernel }
    }

    #[test]
    fn speedup_and_geomean() {
        // 2x and 8x speedups → geomean 4x.
        let obs = vec![o(true, 10.0, 5.0), o(true, 16.0, 2.0)];
        assert!((speedup_geomean(&obs) - 4.0).abs() < 1e-9);
        assert!((correctness(&obs) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn incorrect_excluded_from_speedup() {
        let obs = vec![o(true, 10.0, 5.0), o(false, 10.0, 1.0)];
        // Only the correct 2x counts; geomean = 2.0.
        assert!((speedup_geomean(&obs) - 2.0).abs() < 1e-9);
        assert!((correctness(&obs) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn fast_p_threshold() {
        let obs = vec![o(true, 10.0, 5.0), o(true, 10.0, 10.0), o(false, 10.0, 1.0)];
        // fast_1: correct AND >=1x → first (2x) and second (1x) = 2/3.
        assert!((fast_p(&obs, 1.0) - 2.0 / 3.0).abs() < 1e-9);
        // fast_2: only the 2x → 1/3.
        assert!((fast_p(&obs, 2.0) - 1.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn empty_is_neutral() {
        assert_eq!(speedup_geomean(&[]), 1.0);
        assert_eq!(correctness(&[]), 0.0);
    }
}
