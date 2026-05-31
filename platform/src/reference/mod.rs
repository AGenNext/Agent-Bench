//! Reference metric implementations.
//!
//! These mirror the canonical metric definitions hosted in **Agent-Metrics**
//! and the way **Agent-Eval** composes CLEAR. They are kept for parity checks
//! and offline analysis — Bench's live evaluation path does **not** use them.
//! Bench consumes measured metric *values* + protocol thresholds and produces a
//! generic `AttributeScore` (see `crate::evaluation` and `crate::db`).

pub mod clear;
pub mod perf;
pub mod progress;
pub mod ranking;
pub mod scoring;
