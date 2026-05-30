use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Any object that can be evaluated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityRef {
    /// Stable identifier, preferably a DID or registry ID.
    pub id: String,
    /// Protocol-neutral entity type, e.g. agent, tool, model, workflow, memory.
    pub entity_type: String,
    /// Optional human-readable name for reports and leaderboard rows.
    pub name: Option<String>,
    /// Optional entity version evaluated in this run.
    pub version: Option<String>,
}

/// The aspect of an entity being evaluated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttributeRef {
    pub key: String,
    pub name: Option<String>,
}

/// Protocol identity. Metric definitions, formulas, thresholds, grading rules,
/// levels, and confidence rules are supplied by the protocol, not hardcoded by
/// Agent-Bench.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolRef {
    pub key: String,
    pub version: String,
}

impl ProtocolRef {
    pub fn label(&self) -> String {
        format!("{}@{}", self.key, self.version)
    }
}

/// Benchmark identity. A benchmark supplies tasks, fixtures, datasets, traces,
/// evidence, or measurements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BenchmarkRef {
    pub key: String,
    pub version: String,
}

impl BenchmarkRef {
    pub fn label(&self) -> String {
        format!("{}@{}", self.key, self.version)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricDirection {
    HigherIsBetter,
    LowerIsBetter,
    Target,
    Range,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Threshold {
    Gte(f64),
    Lte(f64),
    Eq(f64),
    Range { min: f64, max: f64 },
}

impl Threshold {
    pub fn passed(&self, value: f64) -> bool {
        match self {
            Threshold::Gte(target) => value >= *target,
            Threshold::Lte(target) => value <= *target,
            Threshold::Eq(target) => (value - *target).abs() <= f64::EPSILON,
            Threshold::Range { min, max } => value >= *min && value <= *max,
        }
    }

    /// Distance from the acceptable threshold. Zero means pass.
    pub fn severity(&self, value: f64) -> f64 {
        match self {
            Threshold::Gte(target) => ((*target - value) / target.abs().max(1.0)).max(0.0),
            Threshold::Lte(target) => ((value - *target) / target.abs().max(1.0)).max(0.0),
            Threshold::Eq(target) => ((value - *target).abs() / target.abs().max(1.0)).max(0.0),
            Threshold::Range { min, max } => {
                if value < *min {
                    ((*min - value) / min.abs().max(1.0)).max(0.0)
                } else if value > *max {
                    ((value - *max) / max.abs().max(1.0)).max(0.0)
                } else {
                    0.0
                }
            }
        }
    }
}

/// Protocol-supplied metric definition. The formula is intentionally an input
/// reference/expression, not a Rust enum that claims all metrics are built in.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricSpec {
    pub key: String,
    pub direction: MetricDirection,
    pub formula: Option<String>,
    pub threshold: Option<Threshold>,
    pub weight: Option<f64>,
}

/// Evaluated metric outcome.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricScore {
    pub metric_key: String,
    pub value: f64,
    pub normalized_score: Option<f64>,
    pub passed: Option<bool>,
    pub severity: Option<f64>,
    pub weight: f64,
}

impl MetricScore {
    pub fn from_spec(spec: &MetricSpec, value: f64, normalized_score: Option<f64>) -> Self {
        let passed = spec.threshold.as_ref().map(|threshold| threshold.passed(value));
        let severity = spec.threshold.as_ref().map(|threshold| threshold.severity(value));
        Self {
            metric_key: spec.key.clone(),
            value,
            normalized_score,
            passed,
            severity,
            weight: spec.weight.unwrap_or(1.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct EvaluationConditions {
    pub dataset: Option<String>,
    pub sample_size: Option<u64>,
    pub trials: Option<u32>,
    pub hardware: Option<String>,
    pub runtime: Option<String>,
    pub evaluator_version: Option<String>,
    pub notes: Option<String>,
    /// Stable hash of the full conditions manifest when available.
    pub conditions_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ConfidenceBreakdown {
    pub sample: Option<f64>,
    pub reproducibility: Option<f64>,
    pub evidence: Option<f64>,
    pub evaluator: Option<f64>,
    pub coverage: Option<f64>,
    pub stability: Option<f64>,
    pub freshness: Option<f64>,
    pub independence: Option<f64>,
}

impl ConfidenceBreakdown {
    pub fn overall(&self) -> Option<f64> {
        let values = [
            self.sample,
            self.reproducibility,
            self.evidence,
            self.evaluator,
            self.coverage,
            self.stability,
            self.freshness,
            self.independence,
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        if values.is_empty() {
            None
        } else {
            Some(values.iter().sum::<f64>() / values.len() as f64)
        }
    }

    pub fn band(confidence: f64) -> &'static str {
        match confidence {
            c if c >= 0.90 => "very_high",
            c if c >= 0.70 => "high",
            c if c >= 0.40 => "medium",
            _ => "low",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImprovementArea {
    pub metric_key: String,
    pub severity: f64,
    pub current: f64,
    pub target: Option<f64>,
    pub recommendation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttributeScore {
    pub entity: EntityRef,
    pub attribute: AttributeRef,
    pub protocol: ProtocolRef,
    pub benchmark: BenchmarkRef,
    pub metric_scores: Vec<MetricScore>,
    pub grade: f64,
    pub passed: bool,
    pub confidence: Option<f64>,
    pub confidence_band: Option<String>,
    pub level: Option<String>,
    pub improvement_areas: Vec<ImprovementArea>,
}

/// Default generic grade: weighted average of normalized scores when supplied;
/// otherwise fraction of metrics that passed. Protocols may override this.
pub fn default_attribute_grade(metric_scores: &[MetricScore]) -> f64 {
    let weighted = metric_scores
        .iter()
        .filter_map(|score| score.normalized_score.map(|s| (s, score.weight)))
        .collect::<Vec<_>>();

    if !weighted.is_empty() {
        let total_weight = weighted.iter().map(|(_, weight)| *weight).sum::<f64>();
        if total_weight > 0.0 {
            return weighted
                .iter()
                .map(|(score, weight)| score.clamp(0.0, 1.0) * *weight)
                .sum::<f64>()
                / total_weight;
        }
    }

    let judged = metric_scores
        .iter()
        .filter_map(|score| score.passed)
        .collect::<Vec<_>>();

    if judged.is_empty() {
        0.0
    } else {
        judged.iter().filter(|passed| **passed).count() as f64 / judged.len() as f64
    }
}

pub fn default_passed(metric_scores: &[MetricScore]) -> bool {
    metric_scores.iter().all(|score| score.passed.unwrap_or(true))
}

pub fn default_improvement_areas(metric_scores: &[MetricScore]) -> Vec<ImprovementArea> {
    let mut areas = metric_scores
        .iter()
        .filter_map(|score| {
            let severity = score.severity?;
            if severity <= 0.0 {
                return None;
            }
            Some(ImprovementArea {
                metric_key: score.metric_key.clone(),
                severity,
                current: score.value,
                target: None,
                recommendation: None,
            })
        })
        .collect::<Vec<_>>();

    areas.sort_by(|a, b| b.severity.total_cmp(&a.severity));
    areas
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityScore {
    pub entity: EntityRef,
    pub overall_grade: f64,
    pub confidence: Option<f64>,
    pub confidence_band: Option<String>,
    pub attribute_scores: BTreeMap<String, f64>,
    pub blocking_attributes: Vec<String>,
    pub level: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeaderboardRow {
    pub leaderboard_id: String,
    pub rank: Option<u64>,
    pub entity: EntityRef,
    pub attribute_key: Option<String>,
    pub protocol: ProtocolRef,
    pub benchmark: BenchmarkRef,
    pub score: f64,
    pub confidence: Option<f64>,
    pub confidence_band: Option<String>,
    pub primary_metric: String,
    pub secondary_metrics: BTreeMap<String, f64>,
    pub passed: bool,
    pub level: Option<String>,
    pub conditions_hash: Option<String>,
}

/// A maturity-level band: an entity at `grade >= min_grade` reaches this level.
/// Level schemes are inputs (like protocols), not hardcoded; the Salesforce
/// Agentic Maturity Model ships as one scheme via [`salesforce_agentic_levels`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LevelBand {
    pub level: u8,
    pub name: String,
    pub min_grade: f64,
}

/// Salesforce Agentic Maturity Model — 4 levels.
pub fn salesforce_agentic_levels() -> Vec<LevelBand> {
    vec![
        LevelBand { level: 1, name: "Fixed-Function".into(), min_grade: 0.0 },
        LevelBand { level: 2, name: "Knowledge & Reasoning".into(), min_grade: 0.50 },
        LevelBand { level: 3, name: "Multistep / Multi-turn".into(), min_grade: 0.70 },
        LevelBand { level: 4, name: "Multi-Agent / Autonomous".into(), min_grade: 0.90 },
    ]
}

/// Assign the highest level band a grade reaches. Returns `(level, name)`.
pub fn assign_level(grade: f64, bands: &[LevelBand]) -> Option<(u8, String)> {
    bands
        .iter()
        .filter(|b| grade >= b.min_grade)
        .max_by(|a, b| a.level.cmp(&b.level))
        .map(|b| (b.level, b.name.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn salesforce_levels_assign_by_grade() {
        let s = salesforce_agentic_levels();
        assert_eq!(assign_level(0.95, &s), Some((4, "Multi-Agent / Autonomous".into())));
        assert_eq!(assign_level(0.72, &s), Some((3, "Multistep / Multi-turn".into())));
        assert_eq!(assign_level(0.55, &s), Some((2, "Knowledge & Reasoning".into())));
        assert_eq!(assign_level(0.10, &s), Some((1, "Fixed-Function".into())));
    }

    #[test]
    fn threshold_pass_and_severity_work() {
        let threshold = Threshold::Gte(0.8);
        assert!(threshold.passed(0.81));
        assert!(!threshold.passed(0.6));
        assert!(threshold.severity(0.6) > 0.0);
        assert_eq!(threshold.severity(0.9), 0.0);
    }

    #[test]
    fn default_grade_uses_weighted_normalized_scores() {
        let scores = vec![
            MetricScore { metric_key: "a".into(), value: 1.0, normalized_score: Some(1.0), passed: Some(true), severity: Some(0.0), weight: 2.0 },
            MetricScore { metric_key: "b".into(), value: 0.0, normalized_score: Some(0.0), passed: Some(false), severity: Some(1.0), weight: 1.0 },
        ];
        assert!((default_attribute_grade(&scores) - 0.6666666667).abs() < 1e-9);
    }

    #[test]
    fn default_grade_falls_back_to_passed_fraction() {
        let scores = vec![
            MetricScore { metric_key: "a".into(), value: 1.0, normalized_score: None, passed: Some(true), severity: Some(0.0), weight: 1.0 },
            MetricScore { metric_key: "b".into(), value: 0.0, normalized_score: None, passed: Some(false), severity: Some(1.0), weight: 1.0 },
        ];
        assert_eq!(default_attribute_grade(&scores), 0.5);
    }

    #[test]
    fn confidence_overall_averages_available_dimensions() {
        let confidence = ConfidenceBreakdown {
            sample: Some(1.0),
            reproducibility: Some(0.8),
            ..Default::default()
        };
        assert_eq!(confidence.overall(), Some(0.9));
        assert_eq!(ConfidenceBreakdown::band(0.9), "very_high");
    }
}
