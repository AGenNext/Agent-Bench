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
DEFINE FIELD subject   ON workload TYPE record<entity>;    -- acted on / measured (metamodel entity)
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
DEFINE FIELD observation ON run TYPE option<object>;                    -- measured metric values (no formula here)
DEFINE FIELD score      ON run TYPE option<record<attribute_score>>;    -- generic verdict (metamodel)
DEFINE FIELD started_at  ON run TYPE option<datetime>;
DEFINE FIELD ended_at    ON run TYPE option<datetime>;
DEFINE FIELD created_at   ON run TYPE datetime VALUE time::now();
DEFINE INDEX run_phase ON run FIELDS phase;

-- -----------------------------------------------------------------------------
-- Workload kind: the open set, as data not code. "Anything" — seeded with the
-- known kinds; new kinds are rows, never a schema change.
-- -----------------------------------------------------------------------------
DEFINE TABLE workload_kind SCHEMAFULL;
DEFINE FIELD key  ON workload_kind TYPE string;
DEFINE FIELD does ON workload_kind TYPE string;   -- what the container does
DEFINE INDEX workload_kind_key ON workload_kind FIELDS key UNIQUE;

UPSERT workload_kind:research         SET key='research',         does='run a research harness, produce a cited report';
UPSERT workload_kind:bench            SET key='bench',            does='replay a benchmark suite for a subject';
UPSERT workload_kind:`create-agent`   SET key='create-agent',    does='synthesize/assemble an agent into the registry';
UPSERT workload_kind:`improve-quality` SET key='improve-quality', does='run an improvement loop on a subject';
UPSERT workload_kind:`cluster-conformance` SET key='cluster-conformance', does='prove a registered cluster honors placement/budget on real capacity (Talos-style integration)';

-- -----------------------------------------------------------------------------
-- Cluster conformance check: a registered cluster is only usable once it proves
-- it honors what placement promises. Talos-style integration scenarios modeled
-- as DATA (apply desired state -> reconcile -> assert observed). New scenarios
-- are rows, not code; the cluster-conformance workload runs them on real
-- capacity and writes phase back to `run`.
-- -----------------------------------------------------------------------------
DEFINE TABLE conformance_check SCHEMAFULL;
DEFINE FIELD key      ON conformance_check TYPE string;
DEFINE FIELD area     ON conformance_check TYPE string   -- storage | security | resource | provisioning | core
    ASSERT $value IN ['storage','security','resource','provisioning','core'];
DEFINE FIELD asserts  ON conformance_check TYPE string;  -- the observed state that must hold
DEFINE FIELD required ON conformance_check TYPE bool DEFAULT true;  -- gates cluster.healthy
DEFINE INDEX conformance_check_key ON conformance_check FIELDS key UNIQUE;

UPSERT conformance_check:`pvc-bind`        SET key='pvc-bind',        area='storage',      asserts='a PersistentVolumeClaim binds on the cluster CSI driver';
UPSERT conformance_check:`apparmor`        SET key='apparmor',        area='security',     asserts='an AppArmor-confined pod is admitted and confined';
UPSERT conformance_check:`user-namespace`  SET key='user-namespace',  area='security',     asserts='a pod runs in an isolated user namespace';
UPSERT conformance_check:`oom-handling`    SET key='oom-handling',    area='resource',     asserts='a pod over its memory budget is OOM-killed, not the node';
UPSERT conformance_check:`manifest-apply`  SET key='manifest-apply',  area='core',         asserts='applied manifests reconcile to Ready';
UPSERT conformance_check:`version-skew`    SET key='version-skew',    area='core',         asserts='control-plane/kubelet version skew is within the supported band';
UPSERT conformance_check:`bare-metal-provision` SET key='bare-metal-provision', area='provisioning', asserts='a node provisions from bare metal and joins Ready', required=false;

-- -----------------------------------------------------------------------------
-- Graph relations (crawl/construct) — mirrors metamodel.surql.
-- -----------------------------------------------------------------------------
DEFINE TABLE has_kind     TYPE RELATION FROM workload TO workload_kind;
DEFINE TABLE placed_on    TYPE RELATION FROM run      TO cluster;
DEFINE TABLE orchestrates TYPE RELATION FROM workload TO run;
DEFINE TABLE constrained_by TYPE RELATION FROM workload TO placement;
-- a cluster is proven by the conformance checks it passes (gates cluster.healthy)
DEFINE TABLE proven_by   TYPE RELATION FROM cluster  TO conformance_check;
