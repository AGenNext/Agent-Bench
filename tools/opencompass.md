# OpenCompass (external benchmark runner)

- Upstream: https://github.com/open-compass/opencompass
- Role in Agent-Bench: **measurement source**. OpenCompass runs a model/agent
  over its datasets and emits metric values; Agent-Bench ingests those values
  and scores them against a supplied protocol's thresholds. Bench does **not**
  define the metrics or re-run inference.

## Boundary

| Concern | Owner |
|---|---|
| Run inference over datasets, compute raw metric values, summarize | **OpenCompass** |
| Canonical metric formula/definition | **Agent-Metrics** (referenced by id) |
| Score values vs. protocol thresholds → grade / pass / improvement areas | **Agent-Bench** (`evaluation.rs`) |
| Cross-attribute roll-up / leveling | **Agent-GPA** |

OpenCompass is Python; it runs as an external process. Agent-Bench (Rust, no
Python) consumes its JSON summary — it is referenced, not embedded.

## Mapping: OpenCompass result → Agent-Bench `SubmitRun`

OpenCompass produces a summary table of `dataset → metric → score`. Each cell
becomes one `SubmittedMetric` (the protocol supplies `direction`, `threshold`,
`weight`):

```json
{
  "agent_id": "qwen2-7b-instruct-1",
  "benchmark_id": "opencompass:gsm8k",
  "attribute": "reasoning",
  "protocol": "OC-REASONING@0.1.0",
  "metrics": [
    { "key": "accuracy", "direction": "higher_is_better",
      "value": 0.81, "normalized_score": 0.81,
      "threshold": { "gte": 0.70 } }
  ]
}
```

- `value` is OpenCompass's measured score (already computed against an
  Agent-Metrics definition — e.g. `agent-metrics:accuracy`).
- `threshold` / `direction` / `weight` come from the protocol, not OpenCompass.
- Bench returns an `AttributeScore` (grade, passed, improvement areas, level)
  and ranks it on the leaderboard.

## Operating it

See the `opencompass` Claude skill (`.claude/skills/opencompass/SKILL.md`) for
running an evaluation and converting its summarizer output into the `SubmitRun`
shape above.
