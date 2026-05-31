//! Readiness scoring over a generic `AttributeScore`.
//!
//! With the `surrealml` feature, this calls a SurrealML model registered via
//! `DEFINE MODEL` using the `ml::` SurrealQL functions, letting the database
//! score agents natively. Without it, readiness is the attribute grade the
//! generic metamodel already produced.

use crate::db::Store;
use crate::error::AppResult;
use crate::evaluation::{AttributeScore, ImprovementArea};

/// Predict a 0–1 "production-readiness" score for an attribute result.
///
/// A trained SurrealML regressor can replace the grade once enough historical
/// evaluations exist; the feature vector is the metamodel's grade + confidence.
pub async fn readiness_score(
    store: &Store,
    tenant: &str,
    score: &AttributeScore,
) -> AppResult<f64> {
    #[cfg(feature = "surrealml")]
    {
        // Requires a model registered as `readiness` in this namespace, e.g.:
        //   DEFINE MODEL ml::readiness<...>;
        let predicted: Option<f64> = store
            .raw_query(
                tenant,
                "RETURN ml::readiness({ grade: $grade, confidence: $conf })",
                &[
                    ("grade", score.grade),
                    ("conf", score.confidence.unwrap_or(0.5)),
                ],
            )
            .await
            .ok()
            .flatten();
        if let Some(p) = predicted {
            return Ok(p.clamp(0.0, 1.0));
        }
    }

    let _ = (store, tenant);
    Ok(score.grade.clamp(0.0, 1.0))
}

/// Improvement areas for an attribute result (worst-first), as produced by the
/// generic metamodel.
pub fn areas(score: &AttributeScore) -> Vec<ImprovementArea> {
    score.improvement_areas.clone()
}
