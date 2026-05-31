//! Rank-fidelity metrics and the Mid-Range Difficulty Filter.
//!
//! Implements the methods distilled in
//! `benchmarks/reference/efficient-benchmarking-ai-agents.md`:
//! rankings survive distribution shift better than absolute scores, and the
//! mid-range (30–70% pass rate) task filter preserves ranking at lower cost.

/// Convert raw values to ranks (1 = smallest), assigning midranks to ties.
fn rank(values: &[f64]) -> Vec<f64> {
    let n = values.len();
    let mut idx: Vec<usize> = (0..n).collect();
    idx.sort_by(|&a, &b| values[a].partial_cmp(&values[b]).unwrap());

    let mut ranks = vec![0.0; n];
    let mut i = 0;
    while i < n {
        let mut j = i;
        while j + 1 < n && (values[idx[j + 1]] - values[idx[i]]).abs() < 1e-12 {
            j += 1;
        }
        // Average rank for ties spanning [i, j].
        let midrank = ((i + j) as f64) / 2.0 + 1.0;
        for k in i..=j {
            ranks[idx[k]] = midrank;
        }
        i = j + 1;
    }
    ranks
}

fn pearson(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len() as f64;
    if n == 0.0 {
        return 0.0;
    }
    let ma = a.iter().sum::<f64>() / n;
    let mb = b.iter().sum::<f64>() / n;
    let mut cov = 0.0;
    let mut va = 0.0;
    let mut vb = 0.0;
    for i in 0..a.len() {
        let da = a[i] - ma;
        let db = b[i] - mb;
        cov += da * db;
        va += da * da;
        vb += db * db;
    }
    if va == 0.0 || vb == 0.0 {
        return 0.0;
    }
    cov / (va.sqrt() * vb.sqrt())
}

/// Spearman's ρ — Pearson correlation on rank vectors.
pub fn spearman(predicted: &[f64], actual: &[f64]) -> f64 {
    assert_eq!(predicted.len(), actual.len());
    pearson(&rank(predicted), &rank(actual))
}

/// Kendall's τ (τ_b variant) — concordant vs. discordant pairs with tie correction.
pub fn kendall_tau(predicted: &[f64], actual: &[f64]) -> f64 {
    assert_eq!(predicted.len(), actual.len());
    let n = predicted.len();
    let (mut concordant, mut discordant) = (0i64, 0i64);
    let (mut tie_p, mut tie_a) = (0i64, 0i64);
    for i in 0..n {
        for j in (i + 1)..n {
            let dp = predicted[i] - predicted[j];
            let da = actual[i] - actual[j];
            if dp == 0.0 && da == 0.0 {
                tie_p += 1;
                tie_a += 1;
            } else if dp == 0.0 {
                tie_p += 1;
            } else if da == 0.0 {
                tie_a += 1;
            } else if (dp > 0.0) == (da > 0.0) {
                concordant += 1;
            } else {
                discordant += 1;
            }
        }
    }
    let n0 = (n * (n - 1) / 2) as i64;
    let denom = (((n0 - tie_p) * (n0 - tie_a)) as f64).sqrt();
    if denom == 0.0 {
        return 0.0;
    }
    (concordant - discordant) as f64 / denom
}

/// Probability that a randomly chosen agent pair is ranked correctly: (τ+1)/2.
pub fn pairwise_correct_prob(tau: f64) -> f64 {
    (tau + 1.0) / 2.0
}

/// A task and its historical pass rate, for the mid-range filter.
#[derive(Debug, Clone)]
pub struct TaskDifficulty {
    pub task_id: String,
    pub pass_rate: f64,
}

/// Mid-Range Difficulty Filter: keep tasks with pass rate in `[lo, hi]`.
/// The paper's default band is `[0.30, 0.70]`.
pub fn mid_range_filter(tasks: &[TaskDifficulty], lo: f64, hi: f64) -> Vec<String> {
    tasks
        .iter()
        .filter(|t| t.pass_rate >= lo && t.pass_rate <= hi)
        .map(|t| t.task_id.clone())
        .collect()
}

/// The paper's default 30–70% band.
pub fn mid_range_default(tasks: &[TaskDifficulty]) -> Vec<String> {
    mid_range_filter(tasks, 0.30, 0.70)
}

/// Fraction of tasks eliminated by a selection.
pub fn reduction_ratio(total: usize, selected: usize) -> f64 {
    if total == 0 {
        return 0.0;
    }
    1.0 - (selected as f64 / total as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spearman_perfect_and_inverse() {
        let a = [1.0, 2.0, 3.0, 4.0];
        let b = [10.0, 20.0, 30.0, 40.0];
        assert!((spearman(&a, &b) - 1.0).abs() < 1e-9);
        let c = [40.0, 30.0, 20.0, 10.0];
        assert!((spearman(&a, &c) + 1.0).abs() < 1e-9);
    }

    #[test]
    fn kendall_matches_known() {
        let a = [1.0, 2.0, 3.0, 4.0];
        let b = [1.0, 2.0, 3.0, 4.0];
        assert!((kendall_tau(&a, &b) - 1.0).abs() < 1e-9);
        assert!((pairwise_correct_prob(1.0) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn mid_range_keeps_band() {
        let tasks = vec![
            TaskDifficulty { task_id: "a".into(), pass_rate: 0.10 },
            TaskDifficulty { task_id: "b".into(), pass_rate: 0.50 },
            TaskDifficulty { task_id: "c".into(), pass_rate: 0.65 },
            TaskDifficulty { task_id: "d".into(), pass_rate: 0.95 },
        ];
        let kept = mid_range_default(&tasks);
        assert_eq!(kept, vec!["b".to_string(), "c".to_string()]);
        assert!((reduction_ratio(4, kept.len()) - 0.5).abs() < 1e-9);
    }
}
