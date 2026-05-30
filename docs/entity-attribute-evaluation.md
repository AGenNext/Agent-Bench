# Entity Attribute Evaluation Model

Agent-Bench is a generic **Entity → Attribute → Evaluation** framework.

It does not assume that metrics, formulas, protocols, or benchmarks are hardcoded.
They are inputs supplied by benchmark authors, protocol authors, enterprise teams, or adapters.

## Core idea

```text
Entity + Attribute + Protocol + Benchmark + Conditions + Evidence = Evaluation Result
```

## Entity

An entity is anything that can be evaluated.

Examples:

- agent
- agent team
- workflow
- prompt
- tool
- skill
- memory implementation
- model
- runtime
- MCP server
- A2A endpoint
- API
- application
- platform
- organization

## Attribute

An attribute is the aspect being evaluated.

Examples:

- memory
- runtime
- tool use
- reasoning
- governance
- security
- trust
- cost
- latency
- reliability
- compliance
- observability

## Protocol

A protocol is the interpretation layer.

It defines the metric inputs, formula inputs, thresholds, grading logic, and improvement-area logic for one attribute.

Metric and formula definitions are **protocol inputs**, not globally hardcoded platform assumptions.

Example:

```yaml
key: AMB-MEMORY
version: 0.1.0
attribute: memory
metrics:
  - key: recall_accuracy
    value_type: float
    direction: higher_is_better
    formula:
      kind: expression
      expression: correct_recalls / total_recalls
    threshold:
      operator: ">="
      value: 0.70
  - key: p99_recall_latency_ms
    value_type: float
    direction: lower_is_better
    formula:
      kind: observed
    threshold:
      operator: "<="
      value: 2000
```

## Benchmark

A benchmark supplies tasks, fixtures, datasets, evidence, traces, or measurements.

Examples:

- CRUD-Bench
- SWE-Bench
- GAIA
- RAGAS
- BrowserBench
- human evaluation
- internal enterprise dataset
- custom compliance review

## Evaluation

An evaluation is a concrete run of a protocol against an entity attribute using benchmark evidence under declared conditions.

Required dimensions:

- entity
- attribute
- protocol key and version
- benchmark key and version
- conditions
- evidence links or embedded evidence
- metric results
- grade
- pass/fail
- improvement areas

## Result shape

```json
{
  "entity": "did:agnext:agent/customer-success",
  "attribute": "memory",
  "protocol": "AMB-MEMORY@0.1.0",
  "benchmark": "MemoryQA@2026-05",
  "conditions": {
    "dataset": "memoryqa-1000",
    "sample_size": 1000,
    "trials": 3,
    "hardware": "m2-max",
    "evaluator_version": "0.1.0"
  },
  "metrics": {
    "recall_accuracy": 0.91,
    "gap_handling": 0.74,
    "p99_recall_latency_ms": 1280
  },
  "grade": 0.93,
  "passed": true,
  "improvement_areas": []
}
```

## Card integration

Agent-Bench stores evaluations as first-class records and can materialize the latest result back into the evaluated entity card.

The card should contain claims such as:

```json
{
  "evaluations": {
    "memory": {
      "protocol": "AMB-MEMORY@0.1.0",
      "benchmark": "MemoryQA@2026-05",
      "grade": 0.93,
      "passed": true,
      "evaluated_at": "2026-05-29T00:00:00Z"
    }
  }
}
```

## Why this model

This keeps Agent-Bench open to any benchmarker:

- metrics are inputs
- formulas are inputs
- thresholds are inputs
- benchmark suites are inputs
- evidence is input
- Agent-Bench standardizes the result envelope, reproducibility manifest, leaderboard fields, and card writeback

That means the platform can evaluate any entity, not only agents, and any attribute, not only memory.
