# External Tools

Tools Agent-Bench **references** to obtain measurements. Bench does not embed or
re-implement them — a tool runs externally and produces measured metric
*values*, which Bench ingests via `POST /v1/runs` (`SubmitRun`) and scores
against the protocol's thresholds into a generic `AttributeScore`. Formula
definitions for those metrics live in **Agent-Metrics**; Bench only references
them by id.

| Tool | Type | Produces | Maps into |
|---|---|---|---|
| [OpenCompass](./opencompass.md) | benchmark runner | per-dataset metric values (accuracy, pass@k, judge scores) over 70+ datasets | `SubmitRun.metrics[]` (value + `Threshold`), `BenchmarkRef` |
