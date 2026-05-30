-- Generic entity-attribute evaluation metamodel — DDL.
-- One-to-one with platform/src/evaluation.rs. Nothing here is specific to a
-- protocol, attribute, or benchmark: metrics, formulas, thresholds, grading,
-- levels and confidence are protocol *inputs*, not hardcoded.

-- --- Entity: anything evaluable (agent, tool, model, workflow, memory, …) ---
DEFINE TABLE entity SCHEMAFULL;
DEFINE FIELD did         ON entity TYPE string;            -- stable id / DID
DEFINE FIELD entity_type ON entity TYPE string;            -- protocol-neutral type
DEFINE FIELD name        ON entity TYPE option<string>;
DEFINE FIELD version     ON entity TYPE option<string>;
DEFINE INDEX entity_did  ON entity FIELDS did UNIQUE;

-- --- Attribute: the aspect being evaluated (memory, runtime, governance, …) -
DEFINE TABLE attribute SCHEMAFULL;
DEFINE FIELD key  ON attribute TYPE string;
DEFINE FIELD name ON attribute TYPE option<string>;
DEFINE INDEX attribute_key ON attribute FIELDS key UNIQUE;

-- --- Metric: protocol-supplied. formula is an input expression/ref, not code -
DEFINE TABLE metric SCHEMAFULL;
DEFINE FIELD key       ON metric TYPE string;
DEFINE FIELD direction ON metric TYPE string
    ASSERT $value IN ["higher_is_better", "lower_is_better", "target", "range"];
DEFINE FIELD formula   ON metric TYPE option<string>;
-- threshold: { kind: gte|lte|eq|range, value?, min?, max? } (Threshold enum)
DEFINE FIELD threshold ON metric TYPE option<object>;
DEFINE FIELD weight    ON metric TYPE option<float>;

-- --- Protocol: how an attribute is evaluated (metrics + thresholds + rules) --
DEFINE TABLE protocol SCHEMAFULL;
DEFINE FIELD key       ON protocol TYPE string;
DEFINE FIELD version   ON protocol TYPE string;
DEFINE FIELD attribute ON protocol TYPE option<string>;     -- attribute key
DEFINE FIELD metrics   ON protocol TYPE option<array>;      -- MetricSpec[]
DEFINE INDEX protocol_label ON protocol FIELDS key, version UNIQUE;

-- --- Benchmark: supplies tasks/fixtures/datasets/measurements ---------------
DEFINE TABLE benchmark SCHEMAFULL;
DEFINE FIELD key     ON benchmark TYPE string;
DEFINE FIELD version ON benchmark TYPE string;
DEFINE INDEX benchmark_label ON benchmark FIELDS key, version UNIQUE;

-- --- Evaluation: the run = entity + attribute + protocol + benchmark + cond --
DEFINE TABLE evaluation SCHEMAFULL;
DEFINE FIELD entity     ON evaluation TYPE record<entity>;
DEFINE FIELD attribute  ON evaluation TYPE record<attribute>;
DEFINE FIELD protocol   ON evaluation TYPE record<protocol>;
DEFINE FIELD benchmark  ON evaluation TYPE record<benchmark>;
-- conditions: EvaluationConditions (dataset, sample_size, trials, hardware,
-- runtime, evaluator_version, notes, conditions_hash)
DEFINE FIELD conditions ON evaluation TYPE option<object>;
DEFINE FIELD created_at ON evaluation TYPE datetime DEFAULT time::now();

-- --- Outcome: AttributeScore (the result of one evaluation) -----------------
DEFINE TABLE attribute_score SCHEMAFULL;
DEFINE FIELD evaluation        ON attribute_score TYPE option<record<evaluation>>;
DEFINE FIELD entity            ON attribute_score TYPE record<entity>;
DEFINE FIELD attribute         ON attribute_score TYPE record<attribute>;
DEFINE FIELD protocol          ON attribute_score TYPE record<protocol>;
DEFINE FIELD benchmark         ON attribute_score TYPE record<benchmark>;
DEFINE FIELD metric_scores     ON attribute_score TYPE array;          -- MetricScore[]
DEFINE FIELD grade             ON attribute_score TYPE float ASSERT $value >= 0 AND $value <= 1;
DEFINE FIELD passed            ON attribute_score TYPE bool;
DEFINE FIELD confidence        ON attribute_score TYPE option<float>;
DEFINE FIELD confidence_band   ON attribute_score TYPE option<string>;
DEFINE FIELD level             ON attribute_score TYPE option<string>;
DEFINE FIELD improvement_areas ON attribute_score TYPE array DEFAULT [];

-- --- EntityScore: composite across attributes -------------------------------
DEFINE TABLE entity_score SCHEMAFULL;
DEFINE FIELD entity              ON entity_score TYPE record<entity>;
DEFINE FIELD overall_grade       ON entity_score TYPE float ASSERT $value >= 0 AND $value <= 1;
DEFINE FIELD confidence          ON entity_score TYPE option<float>;
DEFINE FIELD confidence_band     ON entity_score TYPE option<string>;
DEFINE FIELD attribute_scores    ON entity_score TYPE object DEFAULT {};   -- key -> grade
DEFINE FIELD blocking_attributes ON entity_score TYPE array  DEFAULT [];
DEFINE FIELD level               ON entity_score TYPE option<string>;

-- --- Leaderboard row --------------------------------------------------------
DEFINE TABLE leaderboard_row SCHEMAFULL;
DEFINE FIELD leaderboard_id   ON leaderboard_row TYPE string;
DEFINE FIELD rank             ON leaderboard_row TYPE option<int>;
DEFINE FIELD entity           ON leaderboard_row TYPE record<entity>;
DEFINE FIELD attribute_key    ON leaderboard_row TYPE option<string>;
DEFINE FIELD protocol         ON leaderboard_row TYPE record<protocol>;
DEFINE FIELD benchmark        ON leaderboard_row TYPE record<benchmark>;
DEFINE FIELD score            ON leaderboard_row TYPE float;
DEFINE FIELD confidence       ON leaderboard_row TYPE option<float>;
DEFINE FIELD confidence_band  ON leaderboard_row TYPE option<string>;
DEFINE FIELD primary_metric   ON leaderboard_row TYPE string;
DEFINE FIELD secondary_metrics ON leaderboard_row TYPE object DEFAULT {};
DEFINE FIELD passed           ON leaderboard_row TYPE bool;
DEFINE FIELD level            ON leaderboard_row TYPE option<string>;
DEFINE FIELD conditions_hash  ON leaderboard_row TYPE option<string>;

-- --- Level scheme: maturity bands (grade -> level). Inputs, not hardcoded. ---
-- Ships with the Salesforce Agentic Maturity Model (4 levels); add others.
DEFINE TABLE level_scheme SCHEMAFULL;
DEFINE FIELD scheme    ON level_scheme TYPE string;
DEFINE FIELD level     ON level_scheme TYPE int;
DEFINE FIELD name      ON level_scheme TYPE string;
DEFINE FIELD min_grade ON level_scheme TYPE float;
DEFINE INDEX level_scheme_uid ON level_scheme FIELDS scheme, level UNIQUE;

UPSERT level_scheme:`salesforce-agentic:1` SET scheme='salesforce-agentic', level=1, name='Fixed-Function',             min_grade=0.0;
UPSERT level_scheme:`salesforce-agentic:2` SET scheme='salesforce-agentic', level=2, name='Knowledge & Reasoning',     min_grade=0.50;
UPSERT level_scheme:`salesforce-agentic:3` SET scheme='salesforce-agentic', level=3, name='Multistep / Multi-turn',    min_grade=0.70;
UPSERT level_scheme:`salesforce-agentic:4` SET scheme='salesforce-agentic', level=4, name='Multi-Agent / Autonomous',  min_grade=0.90;

-- Workspace-Bench 5-stage Workspace Learning (L0-L4). Bands illustrative.
UPSERT level_scheme:`workspace-learning:0` SET scheme='workspace-learning', level=0, name='Data-Insensitive Execution',     min_grade=0.0;
UPSERT level_scheme:`workspace-learning:1` SET scheme='workspace-learning', level=1, name='User-Specified File Execution', min_grade=0.40;
UPSERT level_scheme:`workspace-learning:2` SET scheme='workspace-learning', level=2, name='File-to-File Dependency Reasoning', min_grade=0.55;
UPSERT level_scheme:`workspace-learning:3` SET scheme='workspace-learning', level=3, name='Task-to-File Dependency Discovery', min_grade=0.70;
UPSERT level_scheme:`workspace-learning:4` SET scheme='workspace-learning', level=4, name='Workspace-Native Self-Evolution', min_grade=0.90;

-- --- Graph relations (crawl/construct) --------------------------------------
DEFINE TABLE has_attribute  TYPE RELATION FROM entity          TO attribute;
DEFINE TABLE uses_protocol  TYPE RELATION FROM evaluation      TO protocol;
DEFINE TABLE uses_benchmark TYPE RELATION FROM evaluation      TO benchmark;
DEFINE TABLE produces       TYPE RELATION FROM evaluation      TO attribute_score;
DEFINE TABLE ranked_in      TYPE RELATION FROM attribute_score TO leaderboard_row;
