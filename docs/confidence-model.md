# Confidence Model

Agent-Bench separates **score** from **confidence**.

```text
Score      = how good the entity appears to be for an attribute.
Confidence = how much we trust that score.
```

This matters because a high score from weak evidence should not be treated the same as a high score from strong, reproducible evidence.

## Example

```text
Agent A
  attribute_score: 0.94
  confidence:      0.42

Agent B
  attribute_score: 0.88
  confidence:      0.91
```

Agent A looks better, but Agent B is more trustworthy.

Leaderboards, badges, and card claims should be able to display both.

---

## Confidence as an outcome

Confidence is a derived outcome from evaluation evidence and conditions.

```text
Evaluation Result
  -> Metric Scores
  -> Attribute Score
  -> Confidence Score
  -> Leaderboard Row
  -> Card Patch
```

Confidence can exist at multiple levels:

- metric confidence
- attribute confidence
- entity confidence
- benchmark confidence
- leaderboard confidence
- card-claim confidence

---

## Confidence dimensions

Recommended dimensions:

```text
sample_confidence
reproducibility_confidence
evidence_confidence
evaluator_confidence
coverage_confidence
stability_confidence
freshness_confidence
independence_confidence
```

### 1. Sample confidence

How strong is the sample size?

Inputs:

- sample size
- number of trials
- number of tasks
- class/category balance
- benchmark difficulty spread

Example:

```text
sample_confidence = min(1.0, log10(sample_size + 1) / log10(target_sample_size + 1))
```

The formula is protocol supplied.

### 2. Reproducibility confidence

Can the result be reproduced?

Inputs:

- pinned benchmark version
- pinned dataset version
- pinned scoring version
- pinned runtime version
- deterministic seed
- reproducibility manifest present
- evidence bundle present

### 3. Evidence confidence

How strong is the evidence?

Inputs:

- trace availability
- artifact availability
- log completeness
- evaluator notes
- human review evidence
- tool-call evidence
- output artifact hash

### 4. Evaluator confidence

How trusted is the evaluator?

Inputs:

- official evaluator
- third-party evaluator
- self-reported run
- signed attestation
- evaluator version
- evaluator reputation

### 5. Coverage confidence

How much of the attribute did the benchmark cover?

Inputs:

- number of covered metric groups
- number of missing metric groups
- task coverage by scenario
- edge-case coverage
- domain coverage

### 6. Stability confidence

How stable is the score across runs?

Inputs:

- variance
- standard deviation
- pass-rate stability
- rank stability
- regression history

### 7. Freshness confidence

How fresh is the evaluation?

Inputs:

- evaluated_at
- entity version age
- protocol version age
- benchmark version age
- dataset version age

### 8. Independence confidence

How independent is the result?

Inputs:

- self-run
- vendor-run
- platform-run
- customer-run
- third-party-run
- public reproducible run

---

## Default confidence envelope

```json
{
  "confidence": 0.86,
  "confidence_band": "high",
  "confidence_dimensions": {
    "sample": 0.92,
    "reproducibility": 0.95,
    "evidence": 0.87,
    "evaluator": 0.80,
    "coverage": 0.78,
    "stability": 0.83,
    "freshness": 0.90,
    "independence": 0.75
  },
  "confidence_notes": [
    "Dataset and scoring versions are pinned.",
    "Trace bundle is available.",
    "Only 3 trials were executed; more trials would improve stability confidence."
  ]
}
```

---

## Confidence bands

Default bands:

```text
0.00 - 0.39 = low
0.40 - 0.69 = medium
0.70 - 0.89 = high
0.90 - 1.00 = very_high
```

Bands can be protocol supplied.

---

## Leaderboard use

Leaderboards should show both rank score and confidence.

```json
{
  "rank": 1,
  "entity_id": "did:agnext:memory/agent-memory",
  "attribute_key": "memory",
  "score": 0.93,
  "confidence": 0.86,
  "confidence_band": "high",
  "rank_policy": "score_then_confidence"
}
```

Possible leaderboard policies:

```text
score_only
score_then_confidence
confidence_adjusted_score
minimum_confidence_required
separate_verified_leaderboard
```

### Confidence-adjusted score

Some leaderboards may rank by:

```text
confidence_adjusted_score = score * confidence
```

This prevents weak, non-reproducible results from outranking strong verified results.

The ranking formula must be declared in the leaderboard definition.

---

## Badge use

Badges should require minimum confidence.

Example:

```yaml
badge: production_ready
requires:
  score: ">= 0.85"
  confidence: ">= 0.80"
  passed: true
```

A result can pass but still be blocked from a badge if confidence is too low.

---

## Card claim use

Entity cards should never expose a score without confidence context.

```json
{
  "evaluations": {
    "memory": {
      "protocol": "AMB-MEMORY@0.1.0",
      "benchmark": "MemoryQA@2026-05",
      "grade": 0.93,
      "confidence": 0.86,
      "confidence_band": "high",
      "passed": true,
      "evaluated_at": "2026-05-29T00:00:00Z"
    }
  }
}
```

---

## Principle

A score without confidence is only a claim.

A score with evidence, reproducibility, coverage, stability, and evaluator context becomes a trustworthy evaluation outcome.
