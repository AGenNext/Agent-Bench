-- Metric REFERENCES — not definitions. Agent-Bench measures and reports; it
-- does not define metrics. The canonical formulas/grammar live in Agent-Metrics
-- (github.com/AGenNext/Agent-Metrics) and are referenced here by id. Frameworks
-- that apply them (CLEAR in Agent-Eval, SLOs in Agent-SLA) reference the same
-- ids. Nothing in this file declares a formula.

-- A reference to a metric definition hosted by Agent-Metrics. `ref` is the
-- canonical id (agent-metrics:<key>@<version>); Bench never inlines the formula.
DEFINE TABLE metric_ref SCHEMAFULL;
DEFINE FIELD key       ON metric_ref TYPE string;
DEFINE FIELD ref       ON metric_ref TYPE string;     -- agent-metrics:<key>@<version>
DEFINE FIELD direction ON metric_ref TYPE string
    ASSERT $value IN ["higher_is_better", "lower_is_better", "target", "range"];
DEFINE FIELD attribute ON metric_ref TYPE option<string>;
DEFINE FIELD source    ON metric_ref TYPE option<string>;   -- applying framework
DEFINE INDEX metric_ref_key ON metric_ref FIELDS key UNIQUE;

-- CLEAR dimensions (applied by Agent-Eval) ----------------------------------
UPSERT metric_ref:cna             SET key='cna',             ref='agent-metrics:cna@1.0.0',             direction='higher_is_better', attribute='cost',     source='agent-eval:clear';
UPSERT metric_ref:cps             SET key='cps',             ref='agent-metrics:cps@1.0.0',             direction='lower_is_better',  attribute='cost',     source='agent-eval:clear';
UPSERT metric_ref:scr             SET key='scr',             ref='agent-metrics:scr@1.0.0',             direction='higher_is_better', attribute='latency',  source='agent-eval:clear';
UPSERT metric_ref:pas             SET key='pas',             ref='agent-metrics:pas@1.0.0',             direction='higher_is_better', attribute='assurance',source='agent-eval:clear';
UPSERT metric_ref:clear_composite SET key='clear_composite', ref='agent-metrics:clear_composite@1.0.0', direction='higher_is_better', attribute='composite',source='agent-eval:clear';

-- Performance ---------------------------------------------------------------
UPSERT metric_ref:speedup     SET key='speedup',     ref='agent-metrics:speedup@1.0.0',     direction='higher_is_better', attribute='performance';
UPSERT metric_ref:geomean     SET key='geomean',     ref='agent-metrics:geomean@1.0.0',     direction='higher_is_better', attribute='performance';
UPSERT metric_ref:correctness SET key='correctness', ref='agent-metrics:correctness@1.0.0', direction='higher_is_better', attribute='performance';

-- Progress ------------------------------------------------------------------
UPSERT metric_ref:progress_continuous SET key='progress_continuous', ref='agent-metrics:progress_continuous@1.0.0', direction='higher_is_better', attribute='progress';
UPSERT metric_ref:progress_subgoal    SET key='progress_subgoal',    ref='agent-metrics:progress_subgoal@1.0.0',    direction='higher_is_better', attribute='progress';
UPSERT metric_ref:success_rate        SET key='success_rate',        ref='agent-metrics:success_rate@1.0.0',        direction='higher_is_better', attribute='progress';
UPSERT metric_ref:grounding_accuracy  SET key='grounding_accuracy',  ref='agent-metrics:grounding_accuracy@1.0.0',  direction='higher_is_better', attribute='trajectory';
UPSERT metric_ref:step_efficiency     SET key='step_efficiency',     ref='agent-metrics:step_efficiency@1.0.0',     direction='higher_is_better', attribute='trajectory';

-- Ranking fidelity ----------------------------------------------------------
UPSERT metric_ref:reduction_ratio       SET key='reduction_ratio',       ref='agent-metrics:reduction_ratio@1.0.0',       direction='higher_is_better', attribute='ranking';
UPSERT metric_ref:pairwise_correct_prob SET key='pairwise_correct_prob', ref='agent-metrics:pairwise_correct_prob@1.0.0', direction='higher_is_better', attribute='ranking';

-- Workspace-Bench (arXiv:2605.03596) ----------------------------------------
UPSERT metric_ref:f1  SET key='f1',  ref='agent-metrics:f1@1.0.0',  direction='higher_is_better', attribute='workspace', source='workspace-bench';
UPSERT metric_ref:tcr SET key='tcr', ref='agent-metrics:tcr@1.0.0', direction='higher_is_better', attribute='workspace', source='workspace-bench';
