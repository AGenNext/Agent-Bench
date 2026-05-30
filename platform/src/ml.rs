//! Improvement-area prediction.
//!
//! With the `surrealml` feature, this calls a SurrealML model registered via
//! `DEFINE MODEL` using the `ml::` SurrealQL functions, letting the database
//! score agents natively. Without it, we fall back to the deterministic
//! heuristic in [`crate::scoring::improvement_areas`].

use crate::db::Store;
use crate::domain::RunScores;
use crate::error::AppResult;
use crate::scoring::improvement_areas;

/// Predict a 0–1 "production-readiness" score for a run's aggregate scores.
///
/// The CLEAR reference reports CLEAR composite correlating with expert
/// deployment readiness (ρ = 0.83). A trained SurrealML regressor can replace
/// the heuristic once enough historical runs exist.
pub async fn readiness_score(store: &Store, tenant: &str, scores: &RunScores) -> AppResult<f64> {
    #[cfg(feature = "surrealml")]
    {
        // Requires a model registered as `readiness` in this namespace, e.g.:
        //   DEFINE MODEL ml::readiness<...>;
        // Inference via the ml:: function over the CLEAR feature vector.
        let predicted: Option<f64> = store
            .raw_query(
                tenant,
                "RETURN ml::readiness({ \
                    efficacy: $e, reliability: $r, assurance: $a, \
                    sla: $s, cost_norm: $c })",
                &[
                    ("e", scores.clear.efficacy),
                    ("r", scores.pass_at_k),
                    ("a", scores.clear.pas),
                    ("s", scores.clear.scr),
                    ("c", 0.5),
                ],
            )
            .await
            .ok()
            .flatten();
        if let Some(p) = predicted {
            return Ok(p.clamp(0.0, 1.0));
        }
    }

    // Heuristic fallback: composite already blends the dimensions.
    let _ = (store, tenant);
    Ok(scores.clear_composite.clamp(0.0, 1.0))
}

/// Improvement areas for a run (worst CLEAR dimensions below threshold).
pub fn areas(scores: &RunScores) -> Vec<String> {
    improvement_areas(scores, 0.7)
}
