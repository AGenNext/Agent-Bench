-- Migration 0001 — initial schema for the agent-evaluation platform.
-- Applied inside a per-tenant SurrealDB namespace/database. Each enterprise
-- tenant gets its own namespace, so these definitions are tenant-scoped.

-- ---------------------------------------------------------------------------
-- Agents submitted by the tenant for evaluation.
-- ---------------------------------------------------------------------------
DEFINE TABLE agent SCHEMAFULL;
DEFINE FIELD name        ON agent TYPE string;
DEFINE FIELD scaffold    ON agent TYPE string        ASSERT $value != NONE;  -- harness (ReAct, Plan-Execute, ...)
DEFINE FIELD model       ON agent TYPE string;                               -- underlying LLM
DEFINE FIELD version     ON agent TYPE string        DEFAULT "0.1.0";
DEFINE FIELD created_at  ON agent TYPE datetime       DEFAULT time::now();
DEFINE INDEX agent_name  ON agent FIELDS name, version UNIQUE;

-- ---------------------------------------------------------------------------
-- Benchmark suites available on the platform (shared catalogue, copied per ns).
-- ---------------------------------------------------------------------------
DEFINE TABLE benchmark SCHEMAFULL;
DEFINE FIELD benchmark_id ON benchmark TYPE string;
DEFINE FIELD name         ON benchmark TYPE string;
DEFINE FIELD domain       ON benchmark TYPE string;
DEFINE FIELD task_count   ON benchmark TYPE int         DEFAULT 0;
DEFINE FIELD created_at    ON benchmark TYPE datetime    DEFAULT time::now();
DEFINE INDEX benchmark_uid ON benchmark FIELDS benchmark_id UNIQUE;

-- ---------------------------------------------------------------------------
-- A run = one agent evaluated against one benchmark.
-- ---------------------------------------------------------------------------
DEFINE TABLE run SCHEMAFULL;
DEFINE FIELD agent       ON run TYPE record<agent>;
DEFINE FIELD benchmark   ON run TYPE record<benchmark>;
DEFINE FIELD status      ON run TYPE string  DEFAULT "pending"
    ASSERT $value IN ["pending", "running", "scored", "failed"];
DEFINE FIELD trials      ON run TYPE int     DEFAULT 1;
DEFINE FIELD created_at  ON run TYPE datetime DEFAULT time::now();

-- Aggregate CLEAR + ranking scores for the run (populated by the eval engine).
DEFINE FIELD scores                   ON run TYPE option<object>;
DEFINE FIELD scores.efficacy          ON run TYPE option<float>;
DEFINE FIELD scores.cost_usd          ON run TYPE option<float>;
DEFINE FIELD scores.cna               ON run TYPE option<float>;
DEFINE FIELD scores.cps               ON run TYPE option<float>;
DEFINE FIELD scores.scr               ON run TYPE option<float>;
DEFINE FIELD scores.pas               ON run TYPE option<float>;
DEFINE FIELD scores.pass_at_k         ON run TYPE option<float>;
DEFINE FIELD scores.progress_rate     ON run TYPE option<float>;
DEFINE FIELD scores.clear_composite   ON run TYPE option<float>;

-- ---------------------------------------------------------------------------
-- Per-task outcomes for a run — the performance matrix cells. Used by the
-- mid-range difficulty filter and progress-rate computation.
-- ---------------------------------------------------------------------------
DEFINE TABLE task_result SCHEMAFULL;
DEFINE FIELD run            ON task_result TYPE record<run>;
DEFINE FIELD task_id        ON task_result TYPE string;
DEFINE FIELD success        ON task_result TYPE float;   -- fraction of trials solved [0,1]
DEFINE FIELD progress_rate  ON task_result TYPE float    DEFAULT 0.0;
DEFINE FIELD cost_usd       ON task_result TYPE float    DEFAULT 0.0;
DEFINE FIELD latency_ms     ON task_result TYPE float    DEFAULT 0.0;
DEFINE FIELD within_sla     ON task_result TYPE bool     DEFAULT true;
DEFINE FIELD policy_violation ON task_result TYPE bool   DEFAULT false;
DEFINE INDEX task_result_uid ON task_result FIELDS run, task_id UNIQUE;
