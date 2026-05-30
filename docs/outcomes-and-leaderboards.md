# Evaluation Outcomes and Leaderboards

Agent-Bench separates raw evaluation results from materialized outcomes.

A single evaluation can produce multiple outcomes:

```text
Evaluation Result
  -> Metric Scores
  -> Attribute Score
  -> Entity Score
  -> Benchmark Score
  -> Leaderboard Row
  -> Rank Snapshot
  -> Badge / Level
  -> Improvement Plan
  -> Card Patch
```

## Why outcomes are separate

The same metric evidence can be interpreted differently by different protocols, audiences, or leaderboard views.

Examples:

- engineering wants p50/p99 latency
- product wants pass/fail and grade
- enterprise buyer wants maturity level
- leaderboard wants rankable score
- governance wants policy compliance outcome
- agent card wants latest trusted claim

So Agent-Bench stores the evaluation result once and materializes purpose-specific outcomes.

---

## 1. Metric Score

A metric score is the smallest outcome.

Metric and formula definitions are protocol inputs. Agent-Bench stores the observed/evaluated value and the protocol context.

```json
{
  "metric_key": "recall_accuracy",
  "value": 0.91,
  "normalized_score": 0.91,
  "passed": true,
  "threshold": {
    "operator": ">=",
    "value": 0.70
  },
  "direction": "higher_is_better",
  "severity": 0.0
}
```

### Fields

- `metric_key`: protocol-local metric identifier
- `value`: raw metric value
- `normalized_score`: score converted to [0, 1] when possible
- `passed`: threshold outcome
- `threshold`: threshold used by the protocol
- `direction`: higher/lower/target/range
- `severity`: distance from threshold, used for improvement ordering

---

## 2. Attribute Score

An attribute score summarizes all metric scores for one entity attribute under one protocol.

```json
{
  "entity_id": "did:agnext:agent/customer-success",
  "attribute_key": "memory",
  "protocol": "AMB-MEMORY@0.1.0",
  "benchmark": "MemoryQA@2026-05",
  "grade": 0.93,
  "passed": true,
  "metric_count": 6,
  "metrics_passed": 6,
  "confidence": 0.88,
  "level": "production",
  "improvement_areas": []
}
```

### Attribute score calculation

Default rule:

```text
attribute_grade = weighted_average(metric.normalized_score)
```

If no weights are supplied:

```text
attribute_grade = passed_metrics / total_metrics
```

The exact rule is protocol supplied.

---

## 3. Entity Score

An entity score summarizes multiple attribute scores for one entity.

```json
{
  "entity_id": "did:agnext:agent/customer-success",
  "entity_type": "agent",
  "overall_grade": 0.86,
  "level": "operational",
  "attribute_scores": {
    "memory": 0.93,
    "runtime": 0.81,
    "tool_use": 0.79,
    "governance": 0.91
  },
  "blocking_attributes": ["tool_use"],
  "latest_evaluated_at": "2026-05-29T00:00:00Z"
}
```

### Entity score calculation

Default rule:

```text
entity_grade = weighted_average(attribute_scores)
```

Entity scoring profiles can be protocol supplied.

Examples:

- production readiness profile
- governance-first profile
- cost-optimized profile
- latency-critical profile
- enterprise compliance profile

---

## 4. Benchmark Score

A benchmark score is the result of an entity on a specific benchmark suite.

```json
{
  "benchmark": "SWE-Bench@2026-05",
  "entity_id": "did:agnext:agent/coding-agent",
  "score": 0.42,
  "tasks_total": 500,
  "tasks_passed": 210,
  "primary_metric": "resolved_rate",
  "conditions_hash": "sha256:...",
  "evidence_bundle": "artifact://runs/123/evidence.json"
}
```

Benchmark score is what leaderboards usually rank.

---

## 5. Leaderboard Row

A leaderboard row is a materialized, comparable view.

```json
{
  "leaderboard_id": "memory-prod-2026-05",
  "rank": 1,
  "entity_id": "did:agnext:memory/agent-memory",
  "entity_name": "Agent-Memory",
  "entity_type": "memory",
  "attribute_key": "memory",
  "protocol": "AMB-MEMORY@0.1.0",
  "benchmark": "MemoryQA@2026-05",
  "score": 0.93,
  "primary_metric": "attribute_grade",
  "secondary_metrics": {
    "recall_accuracy": 0.91,
    "p99_recall_latency_ms": 1280,
    "cost_per_success": 0.002
  },
  "passed": true,
  "level": "production",
  "conditions_hash": "sha256:...",
  "evaluated_at": "2026-05-29T00:00:00Z"
}
```

### Leaderboard dimensions

Leaderboards should be filterable by:

- entity type
- attribute
- protocol
- benchmark
- version
- dataset
- hardware
- region
- cost profile
- latency profile
- tenant visibility
- public/private visibility

---

## 6. Rank Snapshot

Leaderboard ranks must be snapshot-based, not live mutable facts.

```json
{
  "leaderboard_id": "memory-prod-2026-05",
  "snapshot_at": "2026-05-29T00:00:00Z",
  "ranking_metric": "score",
  "rows": [
    { "rank": 1, "entity_id": "did:agnext:memory/agent-memory", "score": 0.93 },
    { "rank": 2, "entity_id": "did:external:memory/zep", "score": 0.89 }
  ]
}
```

This gives reproducibility and prevents rank drift from changing historical reports.

---

## 7. Badges and Levels

Badges are derived outcomes.

Examples:

```text
passed
production_ready
enterprise_ready
fastest_on_hardware
lowest_cost
best_memory
best_governance
policy_safe
human_review_required
```

Levels are protocol supplied.

Example:

```text
not_evaluated
experimental
development
operational
production
enterprise
mission_critical
```

---

## 8. Improvement Plan

Improvement outcomes are ordered by severity.

```json
{
  "entity_id": "did:agnext:agent/customer-success",
  "attribute_key": "tool_use",
  "protocol": "AMB-TOOL-USE@0.1.0",
  "improvement_areas": [
    {
      "metric_key": "recovery_rate",
      "severity": 0.31,
      "current": 0.49,
      "target": 0.80,
      "recommendation": "Improve retry and fallback behavior for failed tool calls."
    },
    {
      "metric_key": "tool_selection_accuracy",
      "severity": 0.12,
      "current": 0.68,
      "target": 0.80,
      "recommendation": "Add tool descriptions and disambiguation examples."
    }
  ]
}
```

---

## 9. Card Patch Outcome

A card patch is the entity-facing summary.

```json
{
  "evaluations": {
    "memory": {
      "protocol": "AMB-MEMORY@0.1.0",
      "benchmark": "MemoryQA@2026-05",
      "grade": 0.93,
      "passed": true,
      "level": "production",
      "rank": 1,
      "leaderboard_id": "memory-prod-2026-05",
      "evaluated_at": "2026-05-29T00:00:00Z"
    }
  }
}
```

---

## 10. Outcome tables

Recommended SurrealDB records:

```text
evaluation_result
metric_score
attribute_score
entity_score
benchmark_score
leaderboard
leaderboard_row
rank_snapshot
badge_award
improvement_plan
card_patch
```

Recommended relations:

```text
entity -> has_attribute -> attribute
entity -> has_score -> entity_score
attribute_score -> derived_from -> evaluation_result
leaderboard_row -> ranks -> entity
leaderboard_row -> derived_from -> benchmark_score
rank_snapshot -> contains -> leaderboard_row
card_patch -> summarizes -> evaluation_result
```

---

## Principle

Raw results are evidence.

Outcomes are views over evidence.

Leaderboards are ranked, versioned, reproducible views over outcomes.
