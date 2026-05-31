-- =============================================================================
-- Agent-Bench — Orchestrator / Control Plane (declarative)
-- =============================================================================
-- Extends metamodel.surql. A replica of the real world: workloads do real work
-- (research | bench | create-agent | improve-quality | any) on interchangeable
-- multi-cloud capacity, while the control plane keeps ownership + availability.
-- Truth lives here; clouds are placement only. Bench measures and reports.
-- =============================================================================

-- -----------------------------------------------------------------------------
-- Cluster: a registered, cloud-agnostic execution backend. Adding a cloud is
-- adding a row, not code. The control plane never stores truth in the cluster.
-- -----------------------------------------------------------------------------
DEFINE TABLE cluster SCHEMAFULL;
DEFINE FIELD name     ON cluster TYPE string;
DEFINE FIELD provider ON cluster TYPE string;     -- eks | gke | aks | k3s | docker | …
DEFINE FIELD region   ON cluster TYPE string;
DEFINE FIELD endpoint ON cluster TYPE string;     -- kube-api / runtime endpoint
DEFINE FIELD healthy  ON cluster TYPE bool DEFAULT true;
DEFINE FIELD capacity ON cluster TYPE option<object>;  -- {cpu, mem} advertised
DEFINE INDEX cluster_name ON cluster FIELDS name UNIQUE;

-- -----------------------------------------------------------------------------
-- Placement: declarative constraints + HA policy for where a workload may run.
-- -----------------------------------------------------------------------------
DEFINE TABLE placement SCHEMAFULL;
DEFINE FIELD providers ON placement TYPE array DEFAULT [];   -- allowed providers, [] = any
DEFINE FIELD regions   ON placement TYPE array DEFAULT [];   -- allowed regions, [] = any
DEFINE FIELD failover  ON placement TYPE string DEFAULT 'reschedule'; -- none | reschedule | replicate
DEFINE FIELD max_retries ON placement TYPE int DEFAULT 3;

-- -----------------------------------------------------------------------------
-- Workload: the generic unit the orchestrator places. kind is open — "anything".
-- A workload binds a subject + supplied protocol to a cloud-agnostic OCI image.
-- Reproducible from image@digest + inputs + protocol@version + placement.
-- -----------------------------------------------------------------------------
DEFINE TABLE workload SCHEMAFULL;
DEFINE FIELD kind      ON workload TYPE string;            -- research | bench | create-agent | improve-quality | …
DEFINE FIELD subject   ON workload TYPE record<subject>;   -- acted on / measured
DEFINE FIELD protocol  ON workload TYPE record<protocol>;  -- supplied metric set + thresholds (input)
DEFINE FIELD image     ON workload TYPE string             -- OCI ref @ digest (cloud-agnostic)
    ASSERT $value != NONE AND string::len($value) > 0;
DEFINE FIELD inputs    ON workload TYPE object DEFAULT {};  -- kind-specific payload
DEFINE FIELD placement ON workload TYPE record<placement>;
DEFINE FIELD budget    ON workload TYPE option<object>;     -- {cpu, mem, wall, cost}
DEFINE FIELD created_at ON workload TYPE datetime VALUE time::now();

-- -----------------------------------------------------------------------------
-- Run: one orchestrated execution of a workload on a chosen cluster. Lifecycle
-- is reconciled here (desired vs observed); the cluster only reports phase.
-- -----------------------------------------------------------------------------
DEFINE TABLE run SCHEMAFULL;
DEFINE FIELD workload ON run TYPE record<workload>;
DEFINE FIELD cluster  ON run TYPE option<record<cluster>>;  -- chosen at placement
DEFINE FIELD phase    ON run TYPE string DEFAULT 'pending'  -- pending | placed | running | succeeded | failed | rescheduled
    ASSERT $value IN ['pending','placed','running','succeeded','failed','rescheduled'];
DEFINE FIELD attempt   ON run TYPE int DEFAULT 0;
DEFINE FIELD handle     ON run TYPE option<string>;         -- backend job handle
DEFINE FIELD observation ON run TYPE option<record<observation>>;  -- measured values (metamodel)
DEFINE FIELD score      ON run TYPE option<record<score>>;         -- generic verdict (metamodel)
DEFINE FIELD started_at  ON run TYPE option<datetime>;
DEFINE FIELD ended_at    ON run TYPE option<datetime>;
DEFINE FIELD created_at   ON run TYPE datetime VALUE time::now();
DEFINE INDEX run_phase ON run FIELDS phase;
