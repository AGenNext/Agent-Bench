-- Migration 0003 — generic AttributeScore storage.
-- Bench stores the protocol-produced AttributeScore (grade, pass/fail,
-- metric_scores, improvement_areas, level), not CLEAR-specific aggregates.
-- CLEAR/perf metrics are defined in Agent-Metrics and applied by Agent-Eval,
-- so the legacy `scores.*` fields are removed here.

REMOVE FIELD IF EXISTS scores ON TABLE run;

-- The run now carries the attribute/protocol it was evaluated under and the
-- generic score object. FLEXIBLE: the AttributeScore shape is owned by
-- src/evaluation.rs, not pinned column-by-column here.
DEFINE FIELD attribute ON run TYPE option<string>;
DEFINE FIELD protocol  ON run TYPE option<string>;
DEFINE FIELD score     ON run TYPE option<object> FLEXIBLE;
