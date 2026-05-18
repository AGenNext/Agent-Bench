# Benchmark Contract

## Governing Principle

```text
If performance cannot be reproduced,
it cannot be benchmarked.
```

## Benchmark Contract

Each benchmark must define:

```yaml
benchmark_id: string
name: string
version: string
domain: string
objective: string
input_artifacts: []
expected_outputs: []
constraints: []
fixtures: []
execution_steps: []
evaluation_refs: []
pass_thresholds: {}
repeatability_requirements: []
notes: []
```

## Benchmark Categories

- artifact_generation
- enterprise_search
- agent_coordination
- model_routing
- trust_and_provenance
- cost_and_latency
- policy_compliance
- usability
- release_readiness

## Final Rule

A benchmark is a repeatable task definition, not a score.
