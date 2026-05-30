-- Trajectory attribute — DDL. One-to-one with src/attributes/trajectory.rs.
-- Score the agent's execution path: tool calls, steps, plan adherence, grounding.

DEFINE TABLE trajectory_scores SCHEMAFULL;
DEFINE FIELD tool_call_accuracy  ON trajectory_scores TYPE float ASSERT $value >= 0 AND $value <= 1;
DEFINE FIELD step_efficiency     ON trajectory_scores TYPE float ASSERT $value >= 0 AND $value <= 1;
DEFINE FIELD plan_adherence      ON trajectory_scores TYPE float ASSERT $value >= 0 AND $value <= 1;
DEFINE FIELD grounding_accuracy  ON trajectory_scores TYPE float ASSERT $value >= 0 AND $value <= 1;

DEFINE TABLE trajectory_thresholds SCHEMAFULL;
DEFINE FIELD protocol            ON trajectory_thresholds TYPE string;
DEFINE FIELD tool_call_accuracy  ON trajectory_thresholds TYPE float;
DEFINE FIELD step_efficiency     ON trajectory_thresholds TYPE float;
DEFINE FIELD plan_adherence      ON trajectory_thresholds TYPE float;
DEFINE FIELD grounding_accuracy  ON trajectory_thresholds TYPE float;

-- Default thresholds, as data (mirrors TrajectoryThresholds::default()).
UPSERT trajectory_thresholds:`TRAJ-001@0.1.0` SET
    protocol           = "TRAJ-001@0.1.0",
    tool_call_accuracy = 0.80,
    step_efficiency    = 0.60,
    plan_adherence     = 0.70,
    grounding_accuracy = 0.80;

-- Metric formulas (step_efficiency = min(optimal/actual, 1); the ratios) are
-- NOT defined here — they are referenced from Agent-Metrics via metric_ref
-- (agent-metrics:step_efficiency@1.0.0, …). This file holds only the result
-- shape and the supplied thresholds.
