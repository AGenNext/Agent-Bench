-- Metric definitions in DSL (SurrealQL functions) + result shapes in DDL.
-- The formula IS the definition — one-to-one with the Rust reference in
-- src/metrics/*. Verified to execute by tests/schema_ddl.rs.

-- --- CLEAR (src/metrics/clear.rs) ------------------------------------------
DEFINE FUNCTION fn::cna($efficacy: float, $cost: float) {
    RETURN IF $cost > 0 { ($efficacy * 100) / $cost } ELSE { 0.0 };
};
DEFINE FUNCTION fn::cps($total_cost: float, $successes: float) {
    RETURN IF $successes > 0 { $total_cost / $successes } ELSE { math::inf };
};
DEFINE FUNCTION fn::scr($within_sla: int, $total: int) {
    RETURN IF $total > 0 { <float>$within_sla / <float>$total } ELSE { 0.0 };
};
DEFINE FUNCTION fn::pas($violations: int, $critical: int) {
    RETURN IF $critical > 0 { 1.0 - (<float>$violations / <float>$critical) } ELSE { 1.0 };
};
DEFINE FUNCTION fn::clear_composite(
    $w_cost: float, $w_lat: float, $w_eff: float, $w_ass: float, $w_rel: float,
    $cost_norm: float, $lat_norm: float, $eff: float, $ass: float, $rel: float
) {
    RETURN $w_cost * $cost_norm + $w_lat * $lat_norm + $w_eff * $eff
         + $w_ass * $ass + $w_rel * $rel;
};

-- --- Perf (src/metrics/perf.rs) --------------------------------------------
DEFINE FUNCTION fn::speedup($baseline_ms: float, $kernel_ms: float) {
    RETURN IF $kernel_ms > 0 AND $baseline_ms > 0 { $baseline_ms / $kernel_ms } ELSE { 0.0 };
};
-- Geometric mean = product^(1/n); equals exp(mean(ln(x))).
DEFINE FUNCTION fn::geomean($xs: array<float>) {
    RETURN IF array::len($xs) == 0 { 1.0 }
    ELSE { math::pow(math::product($xs), 1.0 / array::len($xs)) };
};
DEFINE FUNCTION fn::correctness($correct: int, $total: int) {
    RETURN IF $total > 0 { <float>$correct / <float>$total } ELSE { 0.0 };
};

-- --- Progress (src/metrics/progress.rs) ------------------------------------
DEFINE FUNCTION fn::progress_continuous($matching_scores: array<float>) {
    RETURN IF array::len($matching_scores) == 0 { 0.0 } ELSE { math::max($matching_scores) };
};
DEFINE FUNCTION fn::progress_subgoal($completed: int, $k: int) {
    RETURN IF $k > 0 { <float>$completed / <float>$k } ELSE { 0.0 };
};
DEFINE FUNCTION fn::success_rate($passed: int, $total: int) {
    RETURN IF $total > 0 { <float>$passed / <float>$total } ELSE { 0.0 };
};
DEFINE FUNCTION fn::grounding_accuracy($valid: int, $total: int) {
    RETURN IF $total > 0 { <float>$valid / <float>$total } ELSE { 0.0 };
};

-- --- Ranking (src/metrics/ranking.rs) --------------------------------------
DEFINE FUNCTION fn::reduction_ratio($total: int, $selected: int) {
    RETURN IF $total > 0 { 1.0 - (<float>$selected / <float>$total) } ELSE { 0.0 };
};
-- (τ + 1) / 2 — probability a random pair is ranked correctly.
DEFINE FUNCTION fn::pairwise_correct_prob($tau: float) {
    RETURN ($tau + 1.0) / 2.0;
};

-- --- Workspace-Bench (arXiv:2605.03596): rubric pass rate, F1, TCR@p --------
-- rubric_pass_rate = passed/total → reuse fn::correctness.
-- F1 = 2PR/(P+R) — node/edge dependency-graph recognition.
DEFINE FUNCTION fn::f1($precision: float, $recall: float) {
    RETURN IF ($precision + $recall) > 0.0 {
        (2.0 * $precision * $recall) / ($precision + $recall)
    } ELSE { 0.0 };
};
-- TCR@p = fraction of tasks meeting >= p of their rubrics.
DEFINE FUNCTION fn::tcr($tasks_met: int, $tasks_total: int) {
    RETURN IF $tasks_total > 0 { <float>$tasks_met / <float>$tasks_total } ELSE { 0.0 };
};

-- --- Result shapes (DDL) ---------------------------------------------------
DEFINE TABLE clear_scores SCHEMAFULL;
DEFINE FIELD efficacy ON clear_scores TYPE float ASSERT $value >= 0 AND $value <= 1;
DEFINE FIELD cost_usd ON clear_scores TYPE float ASSERT $value >= 0;
DEFINE FIELD cna      ON clear_scores TYPE float ASSERT $value >= 0;
DEFINE FIELD cps      ON clear_scores TYPE float;
DEFINE FIELD scr      ON clear_scores TYPE float ASSERT $value >= 0 AND $value <= 1;
DEFINE FIELD pas      ON clear_scores TYPE float ASSERT $value >= 0 AND $value <= 1;

DEFINE TABLE perf_scores SCHEMAFULL;
DEFINE FIELD correctness     ON perf_scores TYPE float ASSERT $value >= 0 AND $value <= 1;
DEFINE FIELD speedup_geomean ON perf_scores TYPE float ASSERT $value >= 0;
DEFINE FIELD fast_1          ON perf_scores TYPE float ASSERT $value >= 0 AND $value <= 1;
