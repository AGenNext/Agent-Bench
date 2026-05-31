//! Agent-Bench platform library.
//!
//! Agent-Bench **measures and reports**; it does not define metrics. Metric
//! formulas live in Agent-Metrics and are referenced by id; frameworks that
//! apply them (CLEAR → Agent-Eval, SLOs → Agent-SLA) own their application.
//!
//! * [`evaluation`] — the generic entity-attribute-protocol-benchmark metamodel:
//!   it turns supplied metric *values* + protocol thresholds into an
//!   `AttributeScore` (grade, pass/fail, improvement areas, level).
//! * [`attributes`] — per-attribute runners (memory, trajectory) that produce
//!   `AttributeScore`s.
//! * the server layer (`db`, `api`, `tenancy`, `ml`) — multi-tenant Axum API
//!   over embedded SurrealDB, behind the `server` feature.

pub mod attributes;
pub mod card;
pub mod domain;
pub mod evaluation;
pub mod judge;
pub mod reference;

#[cfg(feature = "server")]
pub mod api;
#[cfg(feature = "server")]
pub mod db;
#[cfg(feature = "server")]
pub mod error;
#[cfg(feature = "server")]
pub mod ml;
#[cfg(feature = "server")]
pub mod tenancy;
