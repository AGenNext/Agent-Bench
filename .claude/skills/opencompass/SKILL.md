---
name: opencompass
description: Run an OpenCompass evaluation and convert its summarizer output into Agent-Bench run submissions. Use when the user wants to evaluate a model/agent on OpenCompass datasets (e.g. gsm8k, mmlu, humaneval) and feed the results into Agent-Bench, or to map an existing OpenCompass results CSV/JSON into the SubmitRun shape. Agent-Bench scores the measured values against a protocol; it does not re-run inference or define metrics.
---

# OpenCompass → Agent-Bench

OpenCompass is an external **benchmark runner** (Python). It runs a model over
datasets and produces metric values. Agent-Bench ingests those values and scores
them against a protocol's thresholds. Keep the boundary: OpenCompass measures;
Agent-Metrics defines the formulas; Agent-Bench scores + reports.

## 1. Run an evaluation (external, Python)

```bash
# in an OpenCompass checkout / environment
python run.py --models <model_cfg> --datasets <dataset_cfg> --work-dir ./outputs
```

The summarizer writes a table under `outputs/.../summary/` as CSV
(`dataset, version, metric, mode, <model>`).

## 2. Convert the summary into `SubmitRun`

For each `(dataset, metric, score)` row, emit one `SubmittedMetric`. The
protocol — NOT OpenCompass — supplies `direction`, `threshold`, and `weight`.

- `key`        ← OpenCompass metric name (`accuracy`, `pass@1`, `score`), which
  should match an `agent-metrics:<key>` id.
- `value`      ← the measured score (normalize percentages to `[0,1]`).
- `normalized_score` ← `value` when already in `[0,1]`.
- `direction`  ← from the protocol (usually `higher_is_better`).
- `threshold`  ← from the protocol, e.g. `{ "gte": 0.70 }`.

Resulting body (`POST /v1/runs`, header `X-Tenant: <tenant>`):

```json
{
  "agent_id": "<agent record id>",
  "benchmark_id": "opencompass:<dataset>",
  "attribute": "<attribute key, e.g. reasoning>",
  "protocol": "<PROTOCOL@version>",
  "hardware": "",
  "trials": 1,
  "metrics": [
    { "key": "accuracy", "direction": "higher_is_better",
      "value": 0.81, "normalized_score": 0.81, "threshold": { "gte": 0.70 } }
  ]
}
```

## 3. Submit

```bash
curl -s -X POST "$AGENTBENCH_URL/v1/runs" \
  -H 'content-type: application/json' -H "X-Tenant: $TENANT" \
  -d @run.json
```

Agent-Bench returns a `Run` whose `score` is the generic `AttributeScore`
(grade, passed, improvement_areas, level). The leaderboard
(`GET /v1/leaderboard/opencompass:<dataset>`) ranks agents on that grade.

## Rules

- Do not re-implement OpenCompass metrics in Agent-Bench — reference
  `agent-metrics:<key>` ids; submit values only.
- Percentages → divide by 100 before submitting.
- One attribute per submission; group dataset metrics by the attribute the
  protocol covers.
