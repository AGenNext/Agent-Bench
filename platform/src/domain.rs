//! Domain models shared across the API and storage layers.

use serde::{Deserialize, Serialize};

use crate::metrics::clear::ClearScores;

/// An agent submitted by a tenant for evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    /// Harness/framework wrapping the model (ReAct, Plan-Execute, ...).
    pub scaffold: String,
    #[serde(default)]
    pub model: String,
    #[serde(default = "default_version")]
    pub version: String,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

/// A benchmark suite in the catalogue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Benchmark {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub benchmark_id: String,
    pub name: String,
    pub domain: String,
    #[serde(default)]
    pub task_count: u32,
}

/// A single per-task outcome submitted for a run (a performance-matrix cell).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: String,
    /// Fraction of trials solved, in `[0,1]`.
    pub success: f64,
    #[serde(default)]
    pub progress_rate: f64,
    #[serde(default)]
    pub cost_usd: f64,
    #[serde(default)]
    pub latency_ms: f64,
    #[serde(default = "default_true")]
    pub within_sla: bool,
    #[serde(default)]
    pub policy_violation: bool,
    #[serde(default)]
    pub policy_critical: bool,
}

fn default_true() -> bool {
    true
}

/// Request body to submit a run: an agent + benchmark + per-task results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitRun {
    pub agent_id: String,
    pub benchmark_id: String,
    #[serde(default = "default_one")]
    pub trials: u32,
    pub results: Vec<TaskResult>,
}

fn default_one() -> u32 {
    1
}

/// A scored run as stored and returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    pub id: String,
    pub agent_id: String,
    pub benchmark_id: String,
    pub status: String,
    pub trials: u32,
    pub scores: RunScores,
}

/// Aggregate run scores: CLEAR dimensions plus progress rate and composite.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RunScores {
    #[serde(flatten)]
    pub clear: ClearScores,
    pub progress_rate: f64,
    pub pass_at_k: f64,
    pub clear_composite: f64,
}

/// A leaderboard row for one agent on one benchmark.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub rank: u32,
    pub agent_id: String,
    pub agent_name: String,
    pub scaffold: String,
    pub efficacy: f64,
    pub cna: f64,
    pub clear_composite: f64,
    /// Improvement areas identified for this agent (lowest-scoring dimensions).
    pub improvement_areas: Vec<String>,
}
