//! Trajectory attribute — score the agent's *execution path*, not just the final
//! answer. Code-based metrics over tool calls, steps, and plan adherence
//! (grounded in the MLflow / AgentBoard references). Answers the same two
//! questions: how good is the trajectory, and what to improve.

use serde::{Deserialize, Serialize};

use crate::evaluation::{
    default_attribute_grade, default_improvement_areas, default_passed, AttributeRef,
    AttributeScore, BenchmarkRef, EntityRef, MetricDirection, MetricScore, MetricSpec, ProtocolRef,
    Threshold,
};

/// Raw counts observed over an agent run's trajectory.
#[derive(Debug, Clone, Copy)]
pub struct TrajectoryInput {
    pub correct_tool_calls: u32,
    pub total_tool_calls: u32,
    /// Fewest steps a competent agent needs (reference/optimal).
    pub optimal_steps: u32,
    /// Steps the agent actually took.
    pub actual_steps: u32,
    /// Steps that followed the intended plan.
    pub adhered_steps: u32,
    pub planned_steps: u32,
    /// Valid (well-formed, executable) actions vs. all issued.
    pub valid_actions: u32,
    pub total_actions: u32,
}

fn ratio(num: u32, den: u32) -> f64 {
    if den == 0 { 0.0 } else { (num as f64 / den as f64).clamp(0.0, 1.0) }
}

/// Aggregate trajectory scores, all in `[0,1]`.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct TrajectoryScores {
    /// Correct tool invocations / total.
    pub tool_call_accuracy: f64,
    /// Optimal / actual steps (1.0 if at or below optimal).
    pub step_efficiency: f64,
    /// Steps following the plan / planned.
    pub plan_adherence: f64,
    /// Valid actions / total (well-formed, executable).
    pub grounding_accuracy: f64,
}

pub fn score(i: TrajectoryInput) -> TrajectoryScores {
    let step_efficiency = if i.actual_steps == 0 {
        0.0
    } else {
        (i.optimal_steps as f64 / i.actual_steps as f64).min(1.0)
    };
    TrajectoryScores {
        tool_call_accuracy: ratio(i.correct_tool_calls, i.total_tool_calls),
        step_efficiency,
        plan_adherence: ratio(i.adhered_steps, i.planned_steps),
        grounding_accuracy: ratio(i.valid_actions, i.total_actions),
    }
}

/// Supplied protocol: pass thresholds.
#[derive(Debug, Clone, Copy)]
pub struct TrajectoryThresholds {
    pub tool_call_accuracy: f64,
    pub step_efficiency: f64,
    pub plan_adherence: f64,
    pub grounding_accuracy: f64,
}

impl Default for TrajectoryThresholds {
    fn default() -> Self {
        Self {
            tool_call_accuracy: 0.80,
            step_efficiency: 0.60,
            plan_adherence: 0.70,
            grounding_accuracy: 0.80,
        }
    }
}

/// A shortfall: metric, value, threshold, normalized severity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrajectoryGap {
    pub metric: String,
    pub value: f64,
    pub threshold: f64,
    pub severity: f64,
}

/// The two-question verdict for the trajectory attribute.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryVerdict {
    pub scores: TrajectoryScores,
    pub grade: f64,
    pub passed: bool,
    pub improvement_areas: Vec<TrajectoryGap>,
}

pub fn evaluate(s: &TrajectoryScores, t: TrajectoryThresholds) -> TrajectoryVerdict {
    let checks: [(&str, f64, f64); 4] = [
        ("tool_call_accuracy", s.tool_call_accuracy, t.tool_call_accuracy),
        ("step_efficiency", s.step_efficiency, t.step_efficiency),
        ("plan_adherence", s.plan_adherence, t.plan_adherence),
        ("grounding_accuracy", s.grounding_accuracy, t.grounding_accuracy),
    ];
    let mut passed = 0;
    let mut gaps = Vec::new();
    for (metric, value, threshold) in checks {
        if value >= threshold {
            passed += 1;
        } else {
            let severity = if threshold > 0.0 { ((threshold - value) / threshold).clamp(0.0, 1.0) } else { 1.0 };
            gaps.push(TrajectoryGap { metric: metric.into(), value, threshold, severity });
        }
    }
    gaps.sort_by(|a, b| b.severity.partial_cmp(&a.severity).unwrap());
    TrajectoryVerdict {
        scores: s.clone(),
        grade: passed as f64 / checks.len() as f64,
        passed: gaps.is_empty(),
        improvement_areas: gaps,
    }
}

/// Score + evaluate against default thresholds.
pub fn evaluate_default(i: TrajectoryInput) -> TrajectoryVerdict {
    evaluate(&score(i), TrajectoryThresholds::default())
}

// --- Trajectory as a protocol implementation under the generic metamodel ----

/// Emit the generic `AttributeScore` for the trajectory attribute.
pub fn attribute_score(entity: EntityRef, s: &TrajectoryScores, t: TrajectoryThresholds) -> AttributeScore {
    let spec = |key: &str, threshold| MetricSpec {
        key: key.into(),
        direction: MetricDirection::HigherIsBetter,
        formula: None,
        threshold: Some(Threshold::Gte(threshold)),
        weight: None,
    };
    let specs = [
        spec("tool_call_accuracy", t.tool_call_accuracy),
        spec("step_efficiency", t.step_efficiency),
        spec("plan_adherence", t.plan_adherence),
        spec("grounding_accuracy", t.grounding_accuracy),
    ];
    let values = [s.tool_call_accuracy, s.step_efficiency, s.plan_adherence, s.grounding_accuracy];
    let metric_scores: Vec<MetricScore> = specs
        .iter()
        .zip(values)
        .map(|(spec, v)| MetricScore::from_spec(spec, v, None))
        .collect();
    AttributeScore {
        entity,
        attribute: AttributeRef { key: "trajectory".into(), name: Some("Trajectory".into()) },
        protocol: ProtocolRef { key: "TRAJ-001".into(), version: "0.1.0".into() },
        benchmark: BenchmarkRef { key: "TRAJ-001".into(), version: "0.1.0".into() },
        grade: default_attribute_grade(&metric_scores),
        passed: default_passed(&metric_scores),
        confidence: None,
        confidence_band: None,
        level: None,
        improvement_areas: default_improvement_areas(&metric_scores),
        metric_scores,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::EntityRef;

    #[test]
    fn attribute_score_matches_verdict() {
        let s = score(good());
        let verdict = evaluate(&s, TrajectoryThresholds::default());
        let entity = EntityRef { id: "did:agent:x".into(), entity_type: "agent".into(), name: None, version: None };
        let a = attribute_score(entity, &s, TrajectoryThresholds::default());
        assert!((a.grade - verdict.grade).abs() < 1e-9);
        assert_eq!(a.passed, verdict.passed);
        assert_eq!(a.attribute.key, "trajectory");
        assert_eq!(a.metric_scores.len(), 4);
    }

    fn good() -> TrajectoryInput {
        TrajectoryInput {
            correct_tool_calls: 9, total_tool_calls: 10,   // 0.90
            optimal_steps: 8, actual_steps: 10,            // 0.80
            adhered_steps: 9, planned_steps: 10,           // 0.90
            valid_actions: 19, total_actions: 20,          // 0.95
        }
    }

    #[test]
    fn strong_trajectory_passes() {
        let v = evaluate_default(good());
        assert!(v.passed, "areas: {:?}", v.improvement_areas);
        assert!((v.grade - 1.0).abs() < 1e-9);
    }

    #[test]
    fn step_efficiency_caps_at_one() {
        let mut i = good();
        i.actual_steps = 4; // fewer than optimal 8 -> capped at 1.0
        assert!((score(i).step_efficiency - 1.0).abs() < 1e-9);
    }

    #[test]
    fn weak_trajectory_flags_worst_first() {
        let i = TrajectoryInput {
            correct_tool_calls: 3, total_tool_calls: 10,   // 0.30 (worst)
            optimal_steps: 3, actual_steps: 10,            // 0.30
            adhered_steps: 5, planned_steps: 10,           // 0.50
            valid_actions: 18, total_actions: 20,          // 0.90 (passes)
        };
        let v = evaluate_default(i);
        assert!(!v.passed);
        let metrics: Vec<&str> = v.improvement_areas.iter().map(|g| g.metric.as_str()).collect();
        assert!(metrics.contains(&"tool_call_accuracy"));
        assert!(!metrics.contains(&"grounding_accuracy"));
        for w in v.improvement_areas.windows(2) {
            assert!(w[0].severity >= w[1].severity);
        }
    }
}
