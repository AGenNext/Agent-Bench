//! Pure-Rust evaluation metrics engine.
//!
//! No database or network dependencies — every function here is deterministic
//! and unit-tested, so the scoring core can be reused by the API, the CLI, or
//! offline analysis. Each submodule maps to a reference doc under
//! `benchmarks/reference/`.

pub mod clear;
pub mod perf;
pub mod progress;
pub mod ranking;

pub use clear::{
    clear_composite, clear_scores, pass_at_k, ClearScores, ClearWeights, TaskObservation,
};
pub use perf::{perf_scores, PerfObservation, PerfScores};
pub use progress::{
    grounding_accuracy, progress_rate_continuous, progress_rate_subgoal, success_rate,
};
pub use ranking::{
    kendall_tau, mid_range_default, mid_range_filter, pairwise_correct_prob, reduction_ratio,
    spearman, TaskDifficulty,
};
