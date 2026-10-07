-- Migration 0004 — collapse onto the generic metamodel + orchestrator + result packages.
--
-- The live store becomes ONE coherent world. Before this, the store carried the
-- CLEAR-era shape (agent/benchmark/run + per-task matrix) with a generic score
-- object bolted on (0003). This migration installs the full declared model that
-- schema/{metamodel,orchestrator,result_package}.surql define and validate, and
-- removes the last CLEAR-only structure (`task_result`).
--
-- Non-breaking by construction: every NEW field added to the pre-existing `run`
-- and `benchmark` tables is `option<…>` or carries a DEFAULT, so the current
-- service layer (which writes agent/benchmark/status/score) keeps working while
-- the store now also holds the generic + orchestrator + result-package model.
-- The run's `score` (object, 0003) and `created_at` (DEFAULT, 0001) are left as
-- they are — the generic AttributeScore is stored there.
--
-- Applied once per tenant namespace, like every migration.

-- ===========================================================================
-- 0. Remove the last CLEAR-only remnant: the per-task performance matrix.
-- ===========================================================================
REMOVE TABLE IF EXISTS task_result;

-- ===========================================================================
-- 1. Generic entity-attribute metamodel (schema/metamodel.surql).
-- ===========================================================================
DEFINE TABLE entity SCHEMAFULL;
DEFINE FIELD did         ON entity TYPE string;
DEFINE FIELD entity_type ON entity TYPE string;
DEFINE FIELD name        ON entity TYPE option<string>;
DEFINE FIELD version     ON entity TYPE option<string>;
DEFINE INDEX entity_did  ON entity FIELDS did UNIQUE;

DEFINE TABLE attribute SCHEMAFULL;
DEFINE FIELD key  ON attribute TYPE string;
DEFINE FIELD name ON attribute TYPE option<string>;
DEFINE INDEX attribute_key ON attribute FIELDS key UNIQUE;

DEFINE TABLE metric SCHEMAFULL;
DEFINE FIELD key       ON metric TYPE string;
DEFINE FIELD direction ON metric TYPE string
    ASSERT $value IN ["higher_is_better", "lower_is_better", "target", "range"];
DEFINE FIELD formula   ON metric TYPE option<string>;
DEFINE FIELD threshold ON metric FLEXIBLE TYPE option<object>;
DEFINE FIELD weight    ON metric TYPE option<float>;

DEFINE TABLE protocol SCHEMAFULL;
DEFINE FIELD key       ON protocol TYPE string;
DEFINE FIELD version   ON protocol TYPE string;
DEFINE FIELD attribute ON protocol TYPE option<string>;
DEFINE FIELD metrics   ON protocol FLEXIBLE TYPE option<array>;
DEFINE INDEX protocol_label ON protocol FIELDS key, version UNIQUE;

-- `benchmark` pre-exists (0001: benchmark_id/name/domain/task_count). Add the
-- metamodel's key/version as OPTIONAL so legacy benchmark rows + upsert_benchmark
-- (which set benchmark_id, not key/version) keep validating. No unique index on
-- (key,version) here — it would clash with existing rows that have neither.
DEFINE FIELD key     ON benchmark TYPE option<string>;
DEFINE FIELD version ON benchmark TYPE option<string>;

DEFINE TABLE evaluation SCHEMAFULL;
DEFINE FIELD entity     ON evaluation TYPE record<entity>;
DEFINE FIELD attribute  ON evaluation TYPE record<attribute>;
DEFINE FIELD protocol   ON evaluation TYPE record<protocol>;
DEFINE FIELD benchmark  ON evaluation TYPE record<benchmark>;
DEFINE FIELD conditions ON evaluation FLEXIBLE TYPE option<object>;
DEFINE FIELD created_at ON evaluation TYPE datetime DEFAULT time::now();

DEFINE TABLE attribute_score SCHEMAFULL;
DEFINE FIELD evaluation        ON attribute_score TYPE option<record<evaluation>>;
DEFINE FIELD entity            ON attribute_score TYPE record<entity>;
DEFINE FIELD attribute         ON attribute_score TYPE record<attribute>;
DEFINE FIELD protocol          ON attribute_score TYPE record<protocol>;
DEFINE FIELD benchmark         ON attribute_score TYPE record<benchmark>;
DEFINE FIELD metric_scores     ON attribute_score FLEXIBLE TYPE array;
DEFINE FIELD grade             ON attribute_score TYPE float ASSERT $value >= 0 AND $value <= 1;
DEFINE FIELD passed            ON attribute_score TYPE bool;
DEFINE FIELD confidence        ON attribute_score TYPE option<float>;
DEFINE FIELD confidence_band   ON attribute_score TYPE option<string>;
DEFINE FIELD level             ON attribute_score TYPE option<string>;
DEFINE FIELD improvement_areas ON attribute_score TYPE array DEFAULT [];

DEFINE TABLE entity_score SCHEMAFULL;
DEFINE FIELD entity              ON entity_score TYPE record<entity>;
DEFINE FIELD overall_grade       ON entity_score TYPE float ASSERT $value >= 0 AND $value <= 1;
DEFINE FIELD confidence          ON entity_score TYPE option<float>;
DEFINE FIELD confidence_band     ON entity_score TYPE option<string>;
DEFINE FIELD attribute_scores    ON entity_score FLEXIBLE TYPE object DEFAULT {};
DEFINE FIELD blocking_attributes ON entity_score TYPE array  DEFAULT [];
DEFINE FIELD level               ON entity_score TYPE option<string>;

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
DEFINE FIELD secondary_metrics ON leaderboard_row FLEXIBLE TYPE object DEFAULT {};
DEFINE FIELD passed           ON leaderboard_row TYPE bool;
DEFINE FIELD level            ON leaderboard_row TYPE option<string>;
DEFINE FIELD conditions_hash  ON leaderboard_row TYPE option<string>;

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
UPSERT level_scheme:`workspace-learning:0` SET scheme='workspace-learning', level=0, name='Data-Insensitive Execution',     min_grade=0.0;
UPSERT level_scheme:`workspace-learning:1` SET scheme='workspace-learning', level=1, name='User-Specified File Execution', min_grade=0.40;
UPSERT level_scheme:`workspace-learning:2` SET scheme='workspace-learning', level=2, name='File-to-File Dependency Reasoning', min_grade=0.55;
UPSERT level_scheme:`workspace-learning:3` SET scheme='workspace-learning', level=3, name='Task-to-File Dependency Discovery', min_grade=0.70;
UPSERT level_scheme:`workspace-learning:4` SET scheme='workspace-learning', level=4, name='Workspace-Native Self-Evolution', min_grade=0.90;

DEFINE TABLE has_attribute  TYPE RELATION FROM entity          TO attribute;
DEFINE TABLE uses_protocol  TYPE RELATION FROM evaluation      TO protocol;
DEFINE TABLE uses_benchmark TYPE RELATION FROM evaluation      TO benchmark;
DEFINE TABLE produces       TYPE RELATION FROM evaluation      TO attribute_score;
DEFINE TABLE ranked_in      TYPE RELATION FROM attribute_score TO leaderboard_row;

-- ===========================================================================
-- 2. Orchestrator / control plane (schema/orchestrator.surql).
--    New `run` fields are OPTIONAL (or DEFAULTed) so legacy run writes still
--    validate. `run.score`/`run.created_at` are intentionally NOT redefined.
-- ===========================================================================
DEFINE TABLE cluster SCHEMAFULL;
DEFINE FIELD name     ON cluster TYPE string;
DEFINE FIELD provider ON cluster TYPE string;
DEFINE FIELD region   ON cluster TYPE string;
DEFINE FIELD endpoint ON cluster TYPE string;
DEFINE FIELD healthy  ON cluster TYPE bool DEFAULT true;
DEFINE FIELD capacity ON cluster FLEXIBLE TYPE option<object>;
DEFINE INDEX cluster_name ON cluster FIELDS name UNIQUE;

DEFINE TABLE placement SCHEMAFULL;
DEFINE FIELD providers ON placement TYPE array DEFAULT [];
DEFINE FIELD regions   ON placement TYPE array DEFAULT [];
DEFINE FIELD failover  ON placement TYPE string DEFAULT 'reschedule';
DEFINE FIELD max_retries ON placement TYPE int DEFAULT 3;

DEFINE TABLE workload SCHEMAFULL;
DEFINE FIELD kind      ON workload TYPE string;
DEFINE FIELD subject   ON workload TYPE record<entity>;
DEFINE FIELD protocol  ON workload TYPE record<protocol>;
DEFINE FIELD image     ON workload TYPE string
    ASSERT $value != NONE AND string::len($value) > 0;
DEFINE FIELD inputs    ON workload FLEXIBLE TYPE object DEFAULT {};
DEFINE FIELD placement ON workload TYPE record<placement>;
DEFINE FIELD budget    ON workload FLEXIBLE TYPE option<object>;
DEFINE FIELD created_at ON workload TYPE datetime VALUE time::now();
DEFINE FIELD embedding ON workload TYPE option<array<float>>;
DEFINE INDEX workload_vec ON workload FIELDS embedding MTREE DIMENSION 1536 DIST COSINE;

-- New fields on the pre-existing `run` table — all optional/defaulted.
DEFINE FIELD workload   ON run TYPE option<record<workload>>;
DEFINE FIELD cluster    ON run TYPE option<record<cluster>>;
DEFINE FIELD phase      ON run TYPE string DEFAULT 'pending'
    ASSERT $value IN ['pending','placed','running','succeeded','failed','rescheduled'];
DEFINE FIELD attempt    ON run TYPE int DEFAULT 0;
DEFINE FIELD handle     ON run TYPE option<string>;
DEFINE FIELD observation ON run FLEXIBLE TYPE option<object>;
DEFINE FIELD started_at ON run TYPE option<datetime>;
DEFINE FIELD ended_at   ON run TYPE option<datetime>;
DEFINE FIELD trace_id   ON run TYPE option<string>;
DEFINE FIELD span_id    ON run TYPE option<string>;
DEFINE INDEX run_phase ON run FIELDS phase;
DEFINE INDEX run_trace ON run FIELDS trace_id;

DEFINE TABLE span SCHEMAFULL;
DEFINE FIELD run     ON span TYPE record<run>;
DEFINE FIELD name    ON span TYPE string;
DEFINE FIELD phase   ON span TYPE option<string>;
DEFINE FIELD attrs   ON span FLEXIBLE TYPE option<object>;
DEFINE FIELD started_at ON span TYPE datetime;
DEFINE FIELD ended_at   ON span TYPE option<datetime>;
DEFINE INDEX span_run ON span FIELDS run;

DEFINE TABLE workload_kind SCHEMAFULL;
DEFINE FIELD key  ON workload_kind TYPE string;
DEFINE FIELD does ON workload_kind TYPE string;
DEFINE INDEX workload_kind_key ON workload_kind FIELDS key UNIQUE;
UPSERT workload_kind:research         SET key='research',         does='run a research harness, produce a cited report';
UPSERT workload_kind:bench            SET key='bench',            does='replay a benchmark suite for a subject';
UPSERT workload_kind:`create-agent`   SET key='create-agent',    does='synthesize/assemble an agent into the registry';
UPSERT workload_kind:`improve-quality` SET key='improve-quality', does='run an improvement loop on a subject';
UPSERT workload_kind:`cluster-conformance` SET key='cluster-conformance', does='prove a registered cluster honors placement/budget on real capacity (Talos-style integration)';

DEFINE TABLE conformance_check SCHEMAFULL;
DEFINE FIELD key      ON conformance_check TYPE string;
DEFINE FIELD area     ON conformance_check TYPE string
    ASSERT $value IN ['storage','security','resource','provisioning','core'];
DEFINE FIELD asserts  ON conformance_check TYPE string;
DEFINE FIELD required ON conformance_check TYPE bool DEFAULT true;
DEFINE INDEX conformance_check_key ON conformance_check FIELDS key UNIQUE;
UPSERT conformance_check:`pvc-bind`        SET key='pvc-bind',        area='storage',      asserts='a PersistentVolumeClaim binds on the cluster CSI driver';
UPSERT conformance_check:`apparmor`        SET key='apparmor',        area='security',     asserts='an AppArmor-confined pod is admitted and confined';
UPSERT conformance_check:`user-namespace`  SET key='user-namespace',  area='security',     asserts='a pod runs in an isolated user namespace';
UPSERT conformance_check:`oom-handling`    SET key='oom-handling',    area='resource',     asserts='a pod over its memory budget is OOM-killed, not the node';
UPSERT conformance_check:`manifest-apply`  SET key='manifest-apply',  area='core',         asserts='applied manifests reconcile to Ready';
UPSERT conformance_check:`version-skew`    SET key='version-skew',    area='core',         asserts='control-plane/kubelet version skew is within the supported band';
UPSERT conformance_check:`bare-metal-provision` SET key='bare-metal-provision', area='provisioning', asserts='a node provisions from bare metal and joins Ready', required=false;

DEFINE TABLE has_kind     TYPE RELATION FROM workload TO workload_kind;
DEFINE TABLE placed_on    TYPE RELATION FROM run      TO cluster;
DEFINE TABLE orchestrates TYPE RELATION FROM workload TO run;
DEFINE TABLE constrained_by TYPE RELATION FROM workload TO placement;
DEFINE TABLE proven_by    TYPE RELATION FROM cluster  TO conformance_check;
DEFINE TABLE traced_by    TYPE RELATION FROM run      TO span;

-- ===========================================================================
-- 3. Result packages & reproducibility (schema/result_package.surql).
-- ===========================================================================
DEFINE TABLE repro_manifest SCHEMAFULL;
DEFINE FIELD image_digest ON repro_manifest TYPE string
    ASSERT string::starts_with($value, 'sha256:');
DEFINE FIELD protocol     ON repro_manifest TYPE record<protocol>;
DEFINE FIELD inputs_hash  ON repro_manifest TYPE string;
DEFINE FIELD dataset      ON repro_manifest TYPE option<string>;
DEFINE FIELD seed         ON repro_manifest TYPE option<int>;
DEFINE FIELD trials       ON repro_manifest TYPE int DEFAULT 1;
DEFINE FIELD hardware     ON repro_manifest FLEXIBLE TYPE option<object>;
DEFINE FIELD evaluator    ON repro_manifest TYPE option<string>;

DEFINE TABLE result_package SCHEMAFULL;
DEFINE FIELD subject   ON result_package TYPE record<entity>;
DEFINE FIELD run       ON result_package TYPE record<run>;
DEFINE FIELD score     ON result_package TYPE record<attribute_score>;
DEFINE FIELD manifest  ON result_package TYPE record<repro_manifest>;
DEFINE FIELD digest    ON result_package TYPE string
    ASSERT string::starts_with($value, 'sha256:');
DEFINE FIELD category  ON result_package TYPE option<string>;
DEFINE FIELD visibility ON result_package TYPE string DEFAULT 'private'
    ASSERT $value IN ['private', 'public'];
DEFINE FIELD attestation ON result_package FLEXIBLE TYPE option<object>;
DEFINE FIELD published_at ON result_package TYPE option<datetime>;
DEFINE FIELD created_at  ON result_package TYPE datetime VALUE time::now();
DEFINE INDEX result_package_digest ON result_package FIELDS digest UNIQUE;
DEFINE INDEX result_package_subject ON result_package FIELDS subject;

DEFINE TABLE packaged_as   TYPE RELATION FROM run            TO result_package;
DEFINE TABLE reproduced_by TYPE RELATION FROM result_package TO repro_manifest;
DEFINE TABLE published     TYPE RELATION FROM result_package TO entity;

DEFINE FIELD from_package ON leaderboard_row TYPE option<record<result_package>>;
DEFINE TABLE ranks TYPE RELATION FROM result_package TO leaderboard_row;
