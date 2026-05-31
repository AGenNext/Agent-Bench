//! Domain models shared across the API and storage layers.
//!
//! Bench consumes measured metric *values* (computed against Agent-Metrics
//! definitions elsewhere) plus the protocol's thresholds, and produces a
//! generic [`AttributeScore`](crate::evaluation::AttributeScore). It does not
//! compute any metric formula.

use serde::{Deserialize, Serialize};

use crate::evaluation::{MetricDirection, Threshold};

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

/// One measured metric for a run: the value (computed against an Agent-Metrics
/// definition) plus the protocol-supplied evaluation rule (direction,
/// threshold, weight). Bench scores it; it does not compute the value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmittedMetric {
    pub key: String,
    pub direction: MetricDirection,
    pub value: f64,
    /// Optional normalized [0,1] score; when present it drives the weighted grade.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub normalized_score: Option<f64>,
    /// Protocol-supplied pass threshold for this metric.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threshold: Option<Threshold>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
}

/// Request body to submit a run: an agent + benchmark + the attribute/protocol
/// it was evaluated under + measured metric values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitRun {
    pub agent_id: String,
    pub benchmark_id: String,
    /// Attribute key being evaluated, e.g. "memory", "trajectory".
    pub attribute: String,
    /// Protocol label that supplied the metrics/thresholds, e.g. "AMB-001@0.1.0".
    pub protocol: String,
    /// Hardware backend the run targeted (e.g. "gpu-a100", "npu", "cpu").
    #[serde(default)]
    pub hardware: String,
    /// DSL / kernel language (e.g. "triton", "cuda-c", "tilelang").
    #[serde(default)]
    pub dsl: String,
    #[serde(default = "default_one")]
    pub trials: u32,
    /// Measured metric values + their protocol thresholds.
    pub metrics: Vec<SubmittedMetric>,
}

fn default_one() -> u32 {
    1
}

/// A scored run as stored and returned by the API: the generic AttributeScore.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    pub id: String,
    pub agent_id: String,
    pub benchmark_id: String,
    pub hardware: String,
    pub dsl: String,
    pub status: String,
    pub trials: u32,
    pub score: crate::evaluation::AttributeScore,
}

/// A leaderboard row for one agent on one benchmark — ranked on the generic grade.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub rank: u32,
    pub agent_id: String,
    pub agent_name: String,
    pub scaffold: String,
    /// Hardware backend this entry's run targeted.
    pub hardware: String,
    pub dsl: String,
    /// Generic attribute grade in [0,1] — the ranking key.
    pub grade: f64,
    pub passed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
    /// Improvement areas (worst-first), from the generic metamodel.
    pub improvement_areas: Vec<crate::evaluation::ImprovementArea>,
}
