//! Agent-Bench platform library.
//!
//! Two layers:
//! * [`metrics`] — the pure-Rust scoring core (CLEAR, rank fidelity, progress
//!   rate). Always compiled, fully unit-tested, no I/O.
//! * the server layer (`db`, `api`, `tenancy`, `ml`) — multi-tenant Axum API
//!   over embedded SurrealDB, behind the `server` feature.

pub mod attributes;
pub mod card;
pub mod domain;
pub mod judge;
pub mod metrics;
pub mod scoring;

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
