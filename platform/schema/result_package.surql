-- =============================================================================
-- Agent-Bench — Result packages & reproducibility (the OUTPUT artifact)
-- =============================================================================
-- Bench's core output (README/DESIGN): a scored result, packaged as a verifiable,
-- reproducible artifact. Governing principle (benchmark-contract.md):
--   "If performance cannot be reproduced, it cannot be benchmarked."
-- A package binds a run's score to the exact conditions that produced it, so the
-- claim is replayable. Structure follows contracts/benchmark-contract.md.
-- Extends orchestrator.surql + metamodel.surql.
-- =============================================================================

-- -----------------------------------------------------------------------------
-- Reproducibility manifest: the exact, content-addressed inputs that fix a run.
-- Anyone with this manifest can re-place the same workload and expect the same
-- result. This IS reproducibility — image@digest + inputs + protocol@ver + seed.
-- -----------------------------------------------------------------------------
DEFINE TABLE repro_manifest SCHEMAFULL;
DEFINE FIELD image_digest ON repro_manifest TYPE string            -- OCI digest (immutable)
    ASSERT string::starts_with($value, 'sha256:');
DEFINE FIELD protocol     ON repro_manifest TYPE record<protocol>; -- protocol@version
DEFINE FIELD inputs_hash  ON repro_manifest TYPE string;           -- hash of workload.inputs
DEFINE FIELD dataset      ON repro_manifest TYPE option<string>;   -- dataset id/version, if any
DEFINE FIELD seed         ON repro_manifest TYPE option<int>;      -- RNG seed for determinism
DEFINE FIELD trials       ON repro_manifest TYPE int DEFAULT 1;
DEFINE FIELD hardware     ON repro_manifest FLEXIBLE TYPE option<object>;  -- {provider, region, dsl, ...}
DEFINE FIELD evaluator    ON repro_manifest TYPE option<string>;   -- evaluator/harness version

-- -----------------------------------------------------------------------------
-- Result package: the publishable artifact. Binds subject + run + score to the
-- manifest that makes it reproducible. Content-addressed by `digest` so it is
-- immutable and citable on a leaderboard.
-- -----------------------------------------------------------------------------
DEFINE TABLE result_package SCHEMAFULL;
DEFINE FIELD subject   ON result_package TYPE record<entity>;          -- what was measured
DEFINE FIELD run       ON result_package TYPE record<run>;             -- the orchestrated execution
DEFINE FIELD score     ON result_package TYPE record<attribute_score>;-- the verdict (metamodel)
DEFINE FIELD manifest  ON result_package TYPE record<repro_manifest>; -- how to reproduce it
DEFINE FIELD digest    ON result_package TYPE string                  -- content address of the package
    ASSERT string::starts_with($value, 'sha256:');
DEFINE FIELD category  ON result_package TYPE option<string>;         -- benchmark-contract.md category
DEFINE FIELD visibility ON result_package TYPE string DEFAULT 'private' -- publish gate
    ASSERT $value IN ['private', 'public'];
DEFINE FIELD attestation ON result_package FLEXIBLE TYPE option<object>; -- signature/provenance (optional)
DEFINE FIELD published_at ON result_package TYPE option<datetime>;
DEFINE FIELD created_at  ON result_package TYPE datetime VALUE time::now();
DEFINE INDEX result_package_digest ON result_package FIELDS digest UNIQUE;
DEFINE INDEX result_package_subject ON result_package FIELDS subject;

-- -----------------------------------------------------------------------------
-- Graph relations (crawl/construct): subject -> its packages -> reproduced-by.
-- -----------------------------------------------------------------------------
DEFINE TABLE packaged_as  TYPE RELATION FROM run            TO result_package;
DEFINE TABLE reproduced_by TYPE RELATION FROM result_package TO repro_manifest;
DEFINE TABLE published    TYPE RELATION FROM result_package TO entity;  -- package -> subject, when public

-- -----------------------------------------------------------------------------
-- Leaderboard provenance: a ranked row must trace to the public, reproducible
-- package that earned it — no orphan scores on a board. We add the provenance
-- field to the metamodel's leaderboard_row here (in the output layer) rather
-- than editing the shared metamodel, and relate the two for crawl.
-- -----------------------------------------------------------------------------
DEFINE FIELD from_package ON leaderboard_row TYPE option<record<result_package>>;
DEFINE TABLE ranks TYPE RELATION FROM result_package TO leaderboard_row;
