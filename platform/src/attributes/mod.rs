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

//! ## Attribute roadmap
//!
//! Memory is the first attribute. Others are **already defined** elsewhere
//! (runtime, tools, rules, skills — e.g. the Agent-Kernel Action/Controller
//! modules and the registry's tool/skill definitions), so each plugs in here the
//! same way: a `score()` over the defined spec + an `evaluate()` against the
//! supplied protocol thresholds, answering the same two questions.
//!
//! | Attribute | Status | Measured against (defined elsewhere) |
//! |-----------|--------|--------------------------------------|
//! | **memory**  | implemented | AMB-001 protocol |
//! | runtime   | planned | runtime behavior spec |
//! | tools     | planned | tool definitions (registry) — tool-call accuracy, selection |
//! | rules     | planned | rule/policy definitions — conformance (Controller-style) |
//! | skills    | planned | skill definitions (registry graph) |
//!
//! Agent-Bench measures against these defined specs; it does not author them.
//!
//! Attributes and their thresholds are versioned: each evaluation records the
//! protocol version it measured against (e.g. `AMB-001`), so results stay
//! comparable as metrics are refined over time.

pub mod memory;
