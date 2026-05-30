//! Per-attribute agent evaluation.
//!
//! Agent-Bench evaluates an agent **one attribute at a time**, each against the
//! recognized **agent performance metrics** for that attribute, and answers its
//! only two questions per attribute: *how good is it* and *what should it do
//! next*. Today only the **Memory** attribute is implemented.
//!
//! Metrics are not arbitrary — they are the recognized **agent performance
//! metrics**, organized into families defined elsewhere (Agent-Bench measures
//! against them, it does not define them):
//!
//! | Family | Captures | Memory-attribute metrics that roll up |
//! |--------|----------|----------------------------------------|
//! | **Agent GPA**  | overall grade / quality average | the memory `grade` (fraction of metrics passed) |
//! | **Agent SLA**  | latency / availability          | cold-start, p50/p99 recall latency |
//! | **Agent EVAL** | task quality / correctness      | recall accuracy, gap handling, conflict handling |
//! | (ops)          | operational complexity          | external deps required |
//!
//! So a per-attribute verdict is a composition of family metrics: Agent-Bench
//! collects them, ranks, and answers the two questions. The families themselves
//! (GPA, SLA, EVAL, …) are grounded in the reference library and owned in their
//! respective repos.

pub mod memory;
