-- Memory attribute — definitions in DDL, not prose.
-- One-to-one with the Rust reference types (src/attributes/memory.rs, src/card.rs).
-- Types and ASSERTs pin the meaning precisely; ranges are enforced, not described.

-- ---------------------------------------------------------------------------
-- Memory scores (AMB-001 metrics). Mirrors `MemoryScores`.
-- ---------------------------------------------------------------------------
DEFINE TABLE memory_scores SCHEMAFULL;
DEFINE FIELD recall_accuracy        ON memory_scores TYPE float ASSERT $value >= 0 AND $value <= 1;
DEFINE FIELD gap_handling           ON memory_scores TYPE float ASSERT $value >= 0 AND $value <= 1;
DEFINE FIELD conflict_handling_avg  ON memory_scores TYPE float ASSERT $value >= 0 AND $value <= 2;
DEFINE FIELD cold_start_latency_ms  ON memory_scores TYPE float ASSERT $value >= 0;
DEFINE FIELD p50_recall_latency_ms  ON memory_scores TYPE float ASSERT $value >= 0;
DEFINE FIELD p99_recall_latency_ms  ON memory_scores TYPE float ASSERT $value >= 0;
DEFINE FIELD external_deps_required ON memory_scores TYPE int   ASSERT $value >= 0;

-- ---------------------------------------------------------------------------
-- Thresholds (a protocol = metric set + thresholds). Mirrors `MemoryThresholds`.
-- ---------------------------------------------------------------------------
DEFINE TABLE memory_thresholds SCHEMAFULL;
DEFINE FIELD protocol               ON memory_thresholds TYPE string;   -- e.g. "AMB-001@0.1.0"
DEFINE FIELD recall_accuracy        ON memory_thresholds TYPE float;
DEFINE FIELD gap_handling           ON memory_thresholds TYPE float;
DEFINE FIELD conflict_handling_avg  ON memory_thresholds TYPE float;
DEFINE FIELD cold_start_latency_ms  ON memory_thresholds TYPE float;
DEFINE FIELD p50_recall_latency_ms  ON memory_thresholds TYPE float;
DEFINE FIELD p99_recall_latency_ms  ON memory_thresholds TYPE float;

-- AMB-001 default thresholds, as data (not prose). Mirrors `MemoryThresholds::amb_001()`.
UPSERT memory_thresholds:`AMB-001@0.1.0` SET
    protocol              = "AMB-001@0.1.0",
    recall_accuracy       = 0.70,
    gap_handling          = 0.50,
    conflict_handling_avg = 1.0,
    cold_start_latency_ms = 5000.0,
    p50_recall_latency_ms = 500.0,
    p99_recall_latency_ms = 2000.0;

-- ---------------------------------------------------------------------------
-- Card evaluation block written to a subject's card. Mirrors `CardEval` /
-- `TestConditions`. "tested under these conditions -> this result".
-- ---------------------------------------------------------------------------
DEFINE TABLE card_eval SCHEMAFULL;
DEFINE FIELD subject_did                 ON card_eval TYPE string;        -- DID
DEFINE FIELD attribute                   ON card_eval TYPE string;        -- "memory"
DEFINE FIELD grade                       ON card_eval TYPE float ASSERT $value >= 0 AND $value <= 1;
DEFINE FIELD passed                      ON card_eval TYPE bool;
DEFINE FIELD metrics                     ON card_eval TYPE object;        -- attribute-specific scores
DEFINE FIELD improvement_areas           ON card_eval TYPE array<string>;
DEFINE FIELD evaluated_at                ON card_eval TYPE datetime;

-- Documented test conditions / provenance (nested). Mirrors `TestConditions`.
DEFINE FIELD conditions                  ON card_eval TYPE object;
DEFINE FIELD conditions.protocol         ON card_eval TYPE string;
DEFINE FIELD conditions.dataset          ON card_eval TYPE string;
DEFINE FIELD conditions.sample_size      ON card_eval TYPE int   ASSERT $value >= 0;
DEFINE FIELD conditions.trials           ON card_eval TYPE int   ASSERT $value >= 1;
DEFINE FIELD conditions.hardware         ON card_eval TYPE option<string>;
DEFINE FIELD conditions.dsl              ON card_eval TYPE option<string>;
DEFINE FIELD conditions.evaluator_version ON card_eval TYPE string;
DEFINE FIELD conditions.notes            ON card_eval TYPE option<string>;

-- One current evaluation per (subject, attribute); repeated runs update in place.
DEFINE INDEX card_eval_subject_attr ON card_eval FIELDS subject_did, attribute UNIQUE;
