//! Memory attribute evaluation.
//!
//! Agent-Bench evaluates an agent **per attribute**; this is the **Memory**
//! attribute, scored against the AMB-001 protocol (see
//! `benchmarks/memory/AMB-001-benchmark.yaml` and
//! `benchmarks/reference/`). It answers Agent-Bench's only two questions for
//! memory:
//!   1. How good is this agent's memory?  → [`MemoryVerdict::grade`] + per-metric
//!   2. What should it do next?            → [`MemoryVerdict::improvement_areas`]
//!
//! Agent-Bench does not define the protocol — it measures against the supplied
//! thresholds. AMB-001 ships as the default.

use serde::{Deserialize, Serialize};

/// Categories of recall query (AMB-001 query set).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryQueryCategory {
    IdentityRecall,
    TemporalRecall,
    Conflict,
    Gap,
    Decay,
    CrossSession,
}

/// One recall-query outcome.
#[derive(Debug, Clone)]
pub struct MemoryQueryResult {
    pub category: MemoryQueryCategory,
    /// Did the framework return the correct memory vs. the gold answer?
    pub correct: bool,
    /// For `Gap` queries: did it return a useful signal rather than empty?
    pub gap_handled: Option<bool>,
    /// For `Conflict` queries: 0 = none, 1 = partial, 2 = correct.
    pub conflict_score: Option<u8>,
    /// Recall latency for this query, in ms.
    pub recall_latency_ms: f64,
}

/// The supplied protocol: pass thresholds. `amb_001()` is the default.
#[derive(Debug, Clone, Copy)]
pub struct MemoryThresholds {
    pub recall_accuracy: f64,
    pub gap_handling: f64,
    pub conflict_handling_avg: f64,
    pub cold_start_latency_ms: f64,
    pub p50_recall_latency_ms: f64,
    pub p99_recall_latency_ms: f64,
}

impl MemoryThresholds {
    /// AMB-001 pass thresholds.
    pub fn amb_001() -> Self {
        Self {
            recall_accuracy: 0.70,
            gap_handling: 0.50,
            conflict_handling_avg: 1.0,
            cold_start_latency_ms: 5000.0,
            p50_recall_latency_ms: 500.0,
            p99_recall_latency_ms: 2000.0,
        }
    }
}

/// Aggregate memory scores (the six AMB-001 metrics + ops complexity).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct MemoryScores {
    pub recall_accuracy: f64,
    pub gap_handling: f64,
    pub conflict_handling_avg: f64,
    pub cold_start_latency_ms: f64,
    pub p50_recall_latency_ms: f64,
    pub p99_recall_latency_ms: f64,
    /// Count of external processes/services required at runtime (lower = simpler).
    pub external_deps_required: u32,
}

/// Percentile (linear interpolation) of a latency sample, in ms.
fn percentile(values: &[f64], p: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut v = values.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    if v.len() == 1 {
        return v[0];
    }
    let rank = p.clamp(0.0, 1.0) * (v.len() - 1) as f64;
    let lo = rank.floor() as usize;
    let hi = rank.ceil() as usize;
    let frac = rank - lo as f64;
    v[lo] + (v[hi] - v[lo]) * frac
}

/// Compute aggregate memory scores from per-query results.
pub fn score(
    results: &[MemoryQueryResult],
    cold_start_latency_ms: f64,
    external_deps_required: u32,
) -> MemoryScores {
    if results.is_empty() {
        return MemoryScores {
            cold_start_latency_ms,
            external_deps_required,
            ..Default::default()
        };
    }

    // Recall accuracy across all recall queries with a gold answer (all here).
    let recall_accuracy =
        results.iter().filter(|r| r.correct).count() as f64 / results.len() as f64;

    // Gap handling over gap queries only.
    let gaps: Vec<bool> = results.iter().filter_map(|r| r.gap_handled).collect();
    let gap_handling = if gaps.is_empty() {
        1.0 // no gap queries → not penalized
    } else {
        gaps.iter().filter(|&&g| g).count() as f64 / gaps.len() as f64
    };

    // Conflict handling average (0/1/2) over conflict queries only.
    let conflicts: Vec<u8> = results.iter().filter_map(|r| r.conflict_score).collect();
    let conflict_handling_avg = if conflicts.is_empty() {
        2.0 // no conflict queries → not penalized
    } else {
        conflicts.iter().map(|&c| c as f64).sum::<f64>() / conflicts.len() as f64
    };

    let latencies: Vec<f64> = results.iter().map(|r| r.recall_latency_ms).collect();

    MemoryScores {
        recall_accuracy,
        gap_handling,
        conflict_handling_avg,
        cold_start_latency_ms,
        p50_recall_latency_ms: percentile(&latencies, 0.50),
        p99_recall_latency_ms: percentile(&latencies, 0.99),
        external_deps_required,
    }
}

/// A flagged shortfall: which metric, its value, the threshold, and the deficit.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryGap {
    pub metric: String,
    pub value: f64,
    pub threshold: f64,
    /// Normalized severity in [0,1] (bigger = further from passing).
    pub severity: f64,
}

/// The verdict — Agent-Bench's two answers for the memory attribute.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryVerdict {
    pub scores: MemoryScores,
    /// Q1: how good — fraction of metrics meeting threshold, in [0,1].
    pub grade: f64,
    /// True if every metric meets its threshold.
    pub passed: bool,
    /// Q2: what next — shortfalls, worst-first.
    pub improvement_areas: Vec<MemoryGap>,
}

/// Evaluate scores against the supplied thresholds → the two-question verdict.
pub fn evaluate(scores: &MemoryScores, t: MemoryThresholds) -> MemoryVerdict {
    // (name, value, threshold, higher_is_better)
    let checks: [(&str, f64, f64, bool); 6] = [
        ("recall_accuracy", scores.recall_accuracy, t.recall_accuracy, true),
        ("gap_handling", scores.gap_handling, t.gap_handling, true),
        ("conflict_handling", scores.conflict_handling_avg, t.conflict_handling_avg, true),
        ("cold_start_latency_ms", scores.cold_start_latency_ms, t.cold_start_latency_ms, false),
        ("p50_recall_latency_ms", scores.p50_recall_latency_ms, t.p50_recall_latency_ms, false),
        ("p99_recall_latency_ms", scores.p99_recall_latency_ms, t.p99_recall_latency_ms, false),
    ];

    let mut passed_count = 0;
    let mut gaps = Vec::new();
    for (name, value, threshold, higher_better) in checks {
        let ok = if higher_better { value >= threshold } else { value <= threshold };
        if ok {
            passed_count += 1;
        } else {
            // Severity: normalized distance from the threshold.
            let severity = if higher_better {
                ((threshold - value) / threshold).clamp(0.0, 1.0)
            } else if threshold > 0.0 {
                ((value - threshold) / threshold).clamp(0.0, 1.0)
            } else {
                1.0
            };
            gaps.push(MemoryGap { metric: name.to_string(), value, threshold, severity });
        }
    }
    gaps.sort_by(|a, b| b.severity.partial_cmp(&a.severity).unwrap());

    MemoryVerdict {
        scores: scores.clone(),
        grade: passed_count as f64 / checks.len() as f64,
        passed: gaps.is_empty(),
        improvement_areas: gaps,
    }
}

/// Convenience: score + evaluate in one call against AMB-001 defaults.
pub fn evaluate_amb_001(
    results: &[MemoryQueryResult],
    cold_start_latency_ms: f64,
    external_deps_required: u32,
) -> MemoryVerdict {
    let scores = score(results, cold_start_latency_ms, external_deps_required);
    evaluate(&scores, MemoryThresholds::amb_001())
}

// ---------------------------------------------------------------------------
// Comparative evaluation — Agent-Memory vs. other frameworks (AMB-001).
// ---------------------------------------------------------------------------

/// One framework's memory scores in a comparison (e.g. agent-memory, mem0, …).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkMemory {
    pub framework: String,
    pub scores: MemoryScores,
    /// External deps surfaced separately for the ops-complexity column.
    #[serde(default)]
    pub external_deps_required: u32,
}

/// One row of the comparative leaderboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryComparisonRow {
    pub rank: u32,
    pub framework: String,
    pub grade: f64,
    pub passed: bool,
    pub scores: MemoryScores,
}

/// The per-metric leader across frameworks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricLeader {
    pub metric: String,
    pub framework: String,
    pub value: f64,
}

/// A comparative evaluation answering both questions for the focal framework.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryComparison {
    /// Q1: how good is each — ranked best-first.
    pub ranking: Vec<MemoryComparisonRow>,
    /// Best framework per metric.
    pub per_metric_leader: Vec<MetricLeader>,
    /// The framework we're presenting (e.g. "agent-memory").
    pub focal: String,
    pub focal_rank: u32,
    /// Q2: what the focal framework must do to reach the next level —
    /// the gap to the per-metric leader, worst-first.
    pub focal_next_level: Vec<MemoryGap>,
}

const COMPARED_METRICS: [(&str, bool); 7] = [
    ("recall_accuracy", true),
    ("gap_handling", true),
    ("conflict_handling", true),
    ("cold_start_latency_ms", false),
    ("p50_recall_latency_ms", false),
    ("p99_recall_latency_ms", false),
    ("external_deps_required", false),
];

fn metric_value(s: &MemoryScores, metric: &str) -> f64 {
    match metric {
        "recall_accuracy" => s.recall_accuracy,
        "gap_handling" => s.gap_handling,
        "conflict_handling" => s.conflict_handling_avg,
        "cold_start_latency_ms" => s.cold_start_latency_ms,
        "p50_recall_latency_ms" => s.p50_recall_latency_ms,
        "p99_recall_latency_ms" => s.p99_recall_latency_ms,
        "external_deps_required" => s.external_deps_required as f64,
        _ => 0.0,
    }
}

/// Build a comparative evaluation: rank frameworks, find per-metric leaders, and
/// compute the focal framework's gap-to-leader (its path to the next level).
pub fn compare(
    entries: &[FrameworkMemory],
    thresholds: MemoryThresholds,
    focal: &str,
) -> MemoryComparison {
    // Rank by grade (metrics passed), tie-break by recall then lower p50 latency.
    let mut rows: Vec<MemoryComparisonRow> = entries
        .iter()
        .map(|e| {
            let v = evaluate(&e.scores, thresholds);
            MemoryComparisonRow {
                rank: 0,
                framework: e.framework.clone(),
                grade: v.grade,
                passed: v.passed,
                scores: e.scores.clone(),
            }
        })
        .collect();
    rows.sort_by(|a, b| {
        b.grade
            .partial_cmp(&a.grade)
            .unwrap()
            .then(b.scores.recall_accuracy.partial_cmp(&a.scores.recall_accuracy).unwrap())
            .then(a.scores.p50_recall_latency_ms.partial_cmp(&b.scores.p50_recall_latency_ms).unwrap())
    });
    for (i, r) in rows.iter_mut().enumerate() {
        r.rank = (i + 1) as u32;
    }

    // Per-metric leader.
    let mut leaders = Vec::new();
    for (metric, higher_better) in COMPARED_METRICS {
        let best = entries.iter().max_by(|a, b| {
            let (va, vb) = (metric_value(&a.scores, metric), metric_value(&b.scores, metric));
            if higher_better {
                va.partial_cmp(&vb).unwrap()
            } else {
                vb.partial_cmp(&va).unwrap() // lower is better → invert
            }
        });
        if let Some(b) = best {
            leaders.push(MetricLeader {
                metric: metric.to_string(),
                framework: b.framework.clone(),
                value: metric_value(&b.scores, metric),
            });
        }
    }

    // Focal framework's gap to the leader per metric (its next-level path).
    let focal_scores = entries.iter().find(|e| e.framework == focal).map(|e| &e.scores);
    let mut next_level = Vec::new();
    if let Some(fs) = focal_scores {
        for (metric, higher_better) in COMPARED_METRICS {
            let leader = leaders.iter().find(|l| l.metric == metric);
            if let Some(l) = leader {
                if l.framework == focal {
                    continue; // already the leader on this metric
                }
                let v = metric_value(fs, metric);
                let target = l.value;
                let behind = if higher_better { v < target } else { v > target };
                if behind {
                    let severity = if higher_better && target > 0.0 {
                        ((target - v) / target).clamp(0.0, 1.0)
                    } else if !higher_better && v > 0.0 {
                        ((v - target) / v).clamp(0.0, 1.0)
                    } else {
                        0.0
                    };
                    next_level.push(MemoryGap {
                        metric: metric.to_string(),
                        value: v,
                        threshold: target, // target = current leader's value
                        severity,
                    });
                }
            }
        }
        next_level.sort_by(|a, b| b.severity.partial_cmp(&a.severity).unwrap());
    }

    let focal_rank = rows.iter().find(|r| r.framework == focal).map(|r| r.rank).unwrap_or(0);

    MemoryComparison {
        ranking: rows,
        per_metric_leader: leaders,
        focal: focal.to_string(),
        focal_rank,
        focal_next_level: next_level,
    }
}

#[cfg(test)]
mod tests {
    use super::MemoryQueryCategory::*;
    use super::*;

    fn q(cat: MemoryQueryCategory, correct: bool, lat: f64) -> MemoryQueryResult {
        MemoryQueryResult { category: cat, correct, gap_handled: None, conflict_score: None, recall_latency_ms: lat }
    }

    #[test]
    fn percentiles() {
        let v = [100.0, 200.0, 300.0, 400.0];
        assert!((percentile(&v, 0.5) - 250.0).abs() < 1e-9);
        assert!(percentile(&v, 0.99) > 390.0);
    }

    #[test]
    fn strong_memory_passes() {
        let mut results: Vec<MemoryQueryResult> =
            (0..9).map(|_| q(IdentityRecall, true, 100.0)).collect();
        results.push(q(IdentityRecall, false, 120.0)); // 90% recall
        let v = evaluate_amb_001(&results, 1000.0, 0);
        assert!(v.passed, "areas: {:?}", v.improvement_areas);
        assert!((v.grade - 1.0).abs() < 1e-9);
        assert!(v.improvement_areas.is_empty());
    }

    #[test]
    fn weak_memory_flags_worst_first() {
        // 40% recall (below 0.70) and slow p99.
        let mut results = vec![q(IdentityRecall, true, 100.0), q(IdentityRecall, false, 100.0)];
        results.push(q(TemporalRecall, false, 3000.0)); // drags p99 over 2000
        let v = evaluate_amb_001(&results, 1000.0, 0);
        assert!(!v.passed);
        assert!(v.grade < 1.0);
        // recall_accuracy and p99 should be flagged.
        let metrics: Vec<&str> = v.improvement_areas.iter().map(|g| g.metric.as_str()).collect();
        assert!(metrics.contains(&"recall_accuracy"));
        assert!(metrics.contains(&"p99_recall_latency_ms"));
        // Worst-first ordering by severity.
        for w in v.improvement_areas.windows(2) {
            assert!(w[0].severity >= w[1].severity);
        }
    }

    fn fm(name: &str, recall: f64, p50: f64, p99: f64, deps: u32) -> FrameworkMemory {
        FrameworkMemory {
            framework: name.into(),
            scores: MemoryScores {
                recall_accuracy: recall,
                gap_handling: 0.6,
                conflict_handling_avg: 1.2,
                cold_start_latency_ms: 800.0,
                p50_recall_latency_ms: p50,
                p99_recall_latency_ms: p99,
                external_deps_required: deps,
            },
            external_deps_required: deps,
        }
    }

    #[test]
    fn comparative_evaluation_agent_memory_vs_others() {
        // Agent-Memory: embedded (0 deps), strong recall, fast. Others external.
        let entries = vec![
            fm("agent-memory", 0.82, 120.0, 400.0, 0),
            fm("mem0", 0.78, 300.0, 900.0, 1),
            fm("zep", 0.74, 250.0, 1200.0, 2),
            fm("letta", 0.69, 280.0, 1500.0, 2),
        ];
        let cmp = compare(&entries, MemoryThresholds::amb_001(), "agent-memory");

        // Q1: ranking is best-first; agent-memory leads.
        assert_eq!(cmp.ranking[0].framework, "agent-memory");
        assert_eq!(cmp.focal_rank, 1);

        // Agent-memory leads recall, latency, and ops simplicity (0 deps).
        let leader = |m: &str| cmp.per_metric_leader.iter().find(|l| l.metric == m).unwrap();
        assert_eq!(leader("recall_accuracy").framework, "agent-memory");
        assert_eq!(leader("external_deps_required").framework, "agent-memory");
        assert_eq!(leader("p99_recall_latency_ms").framework, "agent-memory");

        // Since it already leads, its next-level list is empty on those metrics.
        assert!(cmp.focal_next_level.iter().all(|g| g.metric != "recall_accuracy"));
    }

    #[test]
    fn focal_behind_gets_next_level_path() {
        // Agent-memory is behind on recall; mem0 leads.
        let entries = vec![
            fm("agent-memory", 0.71, 120.0, 400.0, 0),
            fm("mem0", 0.90, 300.0, 900.0, 1),
        ];
        let cmp = compare(&entries, MemoryThresholds::amb_001(), "agent-memory");
        let recall_gap = cmp.focal_next_level.iter().find(|g| g.metric == "recall_accuracy");
        assert!(recall_gap.is_some(), "should flag recall gap to leader");
        let g = recall_gap.unwrap();
        assert!((g.threshold - 0.90).abs() < 1e-9, "target = leader's value");
        assert!(g.value < g.threshold);
    }

    #[test]
    fn gap_and_conflict_scored_only_on_their_queries() {
        let results = vec![
            MemoryQueryResult { category: Gap, correct: false, gap_handled: Some(true), conflict_score: None, recall_latency_ms: 50.0 },
            MemoryQueryResult { category: Conflict, correct: true, gap_handled: None, conflict_score: Some(2), recall_latency_ms: 60.0 },
        ];
        let s = score(&results, 500.0, 0);
        assert!((s.gap_handling - 1.0).abs() < 1e-9);
        assert!((s.conflict_handling_avg - 2.0).abs() < 1e-9);
    }
}
