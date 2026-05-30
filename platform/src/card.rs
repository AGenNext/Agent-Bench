//! Card updates.
//!
//! After evaluating a subject, Agent-Bench **writes its results into the
//! subject's card** — the metadata descriptor (model-card / agent-card style)
//! held in the registry, addressed by the subject's identity (DID). This module
//! builds the evaluation block and the JSON patch the registry applies.
//!
//! The card *schema* is owned by the registry; Agent-Bench owns the evaluation
//! values it contributes.

use serde::{Deserialize, Serialize};

use crate::attributes::memory::MemoryVerdict;

/// The evaluation block Agent-Bench writes onto a subject's card, per attribute.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CardEval {
    /// Subject identity (DID) the card belongs to.
    pub subject_did: String,
    /// Attribute evaluated, e.g. "memory".
    pub attribute: String,
    /// Protocol the measurement used, e.g. "AMB-001@0.1.0".
    pub protocol: String,
    /// Q1: how good — overall grade in [0,1].
    pub grade: f64,
    pub passed: bool,
    /// Per-metric scores (attribute-specific shape).
    pub metrics: serde_json::Value,
    /// Q2: what to do next.
    pub improvement_areas: Vec<String>,
    /// RFC3339 timestamp of the evaluation.
    pub evaluated_at: String,
}

impl CardEval {
    /// Build the card evaluation block from a memory verdict.
    pub fn from_memory(subject_did: &str, protocol: &str, v: &MemoryVerdict) -> Self {
        CardEval {
            subject_did: subject_did.to_string(),
            attribute: "memory".to_string(),
            protocol: protocol.to_string(),
            grade: v.grade,
            passed: v.passed,
            metrics: serde_json::to_value(&v.scores).unwrap_or(serde_json::Value::Null),
            improvement_areas: v
                .improvement_areas
                .iter()
                .map(|g| g.metric.clone())
                .collect(),
            evaluated_at: now_rfc3339(),
        }
    }

    /// The JSON patch the registry applies to the card: results are keyed by
    /// attribute under an `evaluations` object, so repeated runs update in place.
    ///
    /// ```json
    /// { "evaluations": { "memory": { ...CardEval... } } }
    /// ```
    pub fn as_card_patch(&self) -> serde_json::Value {
        serde_json::json!({
            "evaluations": { self.attribute.clone(): self }
        })
    }
}

fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attributes::memory::{evaluate_amb_001, MemoryQueryCategory, MemoryQueryResult};

    fn results() -> Vec<MemoryQueryResult> {
        (0..10)
            .map(|i| MemoryQueryResult {
                category: MemoryQueryCategory::IdentityRecall,
                correct: i < 8, // 80% recall
                gap_handled: None,
                conflict_score: None,
                recall_latency_ms: 100.0,
            })
            .collect()
    }

    #[test]
    fn builds_card_eval_from_memory_verdict() {
        let v = evaluate_amb_001(&results(), 1000.0, 0);
        let card = CardEval::from_memory("did:agent:strong", "AMB-001@0.1.0", &v);

        assert_eq!(card.subject_did, "did:agent:strong");
        assert_eq!(card.attribute, "memory");
        assert_eq!(card.protocol, "AMB-001@0.1.0");
        assert!(card.passed);
        assert!((card.grade - 1.0).abs() < 1e-9);
        // metrics carry the recall accuracy.
        assert!((card.metrics["recall_accuracy"].as_f64().unwrap() - 0.8).abs() < 1e-9);
        assert!(!card.evaluated_at.is_empty());
    }

    #[test]
    fn patch_is_keyed_by_attribute() {
        let v = evaluate_amb_001(&results(), 1000.0, 0);
        let card = CardEval::from_memory("did:agent:x", "AMB-001@0.1.0", &v);
        let patch = card.as_card_patch();
        // Updates the memory slot in place under evaluations.
        assert_eq!(patch["evaluations"]["memory"]["attribute"], "memory");
        assert_eq!(patch["evaluations"]["memory"]["protocol"], "AMB-001@0.1.0");
    }
}
