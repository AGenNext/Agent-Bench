# MLflow Agent Evaluation — Glossary, Methods & Formulas

Reference notes distilled from:

> *AI Agent Evaluations: A Developer's Practical Guide* — MLflow
> https://mlflow.org/articles/ai-agent-evaluations-a-developers-practical-guide/
> Supporting docs: https://mlflow.org/docs/latest/genai/eval-monitor/
> Source code: https://github.com/mlflow/mlflow (`mlflow.genai.evaluate`, scorers, judges)

**One-line summary.** Evaluate agents on *outcomes and trajectories*, not just
final answers. Combine fast deterministic checks (CI/CD gates) with LLM judges
for nuanced quality, score every intermediate step via traces, and keep
LLM judges trustworthy by calibrating them against a small human-labeled set.

---

## 1. Glossary

| Term | Definition |
|---|---|
| **Scorer** | An MLflow evaluation function that assesses an agent output/trace and returns a value (pass/fail, numeric, or a `Feedback` object). Both code-based scorers and LLM judges are types of scorers. |
| **Code-based (deterministic) scorer** | Rule-based check — exact match, regex, JSON schema, code compilation/test pass. Fast, reproducible; ideal for CI/CD gates. |
| **LLM judge** | A scorer that uses an LLM to grade nuanced dimensions (helpfulness, faithfulness, safety) where deterministic rules fall short. |
| **Predefined (built-in) scorer** | MLflow's research-validated judges: `Correctness`, `RelevanceToQuery`, `Safety`, `Guidelines`, `ExpectationsGuidelines`, `RetrievalGroundedness`, `RetrievalRelevance`, `RetrievalSufficiency`. |
| **Feedback** | The structured result an LLM judge returns (value + rationale), enabling explainable scoring. |
| **Ground truth / expectations** | Reference answers/labels. Required by `Correctness` (and retrieval-relevance style judges); other judges (Safety, Guidelines, Groundedness) run reference-free. |
| **Trace** | The full recorded execution of an agent: inputs, intermediate reasoning, tool calls, retriever spans, and final output. The substrate for step-level scoring and debugging. |
| **Span** | A single step within a trace. Retrieval scorers require spans of type `RETRIEVER`. |
| **Evaluation dataset** | The set of inputs (and optional expectations) the agent is run against during `mlflow.genai.evaluate()`. |
| **Transcript logging** | Recording every intermediate step so it is independently reviewable — non-negotiable for debugging multi-step failures. |
| **Trajectory metrics** | Metrics over the agent's path, not just its answer: Tool Call Accuracy, Plan Adherence, Step Efficiency. |
| **Tool Call Accuracy** | Fraction of tool invocations that are correct (right tool, right arguments). |
| **Plan Adherence** | How closely the agent followed its intended execution plan. |
| **Step Efficiency** | Ratio of optimal steps to actual steps taken. |
| **Task Success Rate** | Primary outcome metric: fraction of tasks the agent completes successfully. |
| **Groundedness** | Whether the output is supported by retrieved/provided context (no hallucination). |
| **Relevance** | Whether the output (or retrieved context) actually addresses the query. |
| **Safety / PII flags** | Checks for harmful output and leakage of sensitive data. |
| **Multi-turn evaluation** | Scoring behavior across a whole conversation: context carry-over, clarifying questions, error recovery, goal completion. |
| **Judge calibration** | Periodically checking LLM-judge agreement against human labels and recalibrating when it drifts. |
| **Calibration set** | A small human-labeled set (≈50–200 examples) used to measure judge–human agreement. |
| **Production monitoring** | Continuously scoring live traces to track quality and debug failures with full execution context. |

---

## 2. Methods

### 2.1 The two-tier scoring stack
- **Deterministic metrics** (exact match, regex, schema validation, code
  compilation/tests) — fast, reproducible, used as CI/CD gates.
- **LLM judges** — handle nuanced quality (helpfulness, faithfulness, safety).
Most teams use **both**: deterministic checks catch hard failures cheaply;
judges grade subjective quality.

### 2.2 Score the trajectory, not just the answer
Tracing that scores **intermediate decision steps** is required for debugging
multi-step workflows. Beyond final-answer correctness, evaluate:
- **Tool Call Accuracy** — were the right tools called with the right args?
- **Plan Adherence** — did execution follow the intended plan?
- **Step Efficiency** — how close to the optimal number of steps?
- **Safety / PII flags** — did any step leak data or violate constraints?

### 2.3 Recommended metric layering
1. Start with **Task Success Rate** as the primary outcome metric.
2. Layer in **Tool Call Accuracy**, **Step Efficiency**, **Latency**, and
   **Token Cost** for a full diagnostic picture.
3. Add domain judges (**Correctness**, **Groundedness**, **Relevance**,
   **Safety**, **Guidelines**) per use case.

### 2.4 Retrieval / RAG agents
Use retrieval scorers that require `RETRIEVER` spans in the trace:
- **RetrievalGroundedness** — output supported by retrieved context.
- **RetrievalRelevance** — retrieved chunks relevant to the query.
- **RetrievalSufficiency** — retrieved context sufficient to answer (vs. ground truth).

### 2.5 Multi-turn evaluation
Score the full conversation: context carried across turns, quality of
clarifying questions, recovery from earlier mistakes, and whether the user's
goal is ultimately reached.

### 2.6 Keeping LLM judges trustworthy (calibration loop)
- Maintain a human-labeled **calibration set of ~50–200 examples**.
- Re-run **judge–human agreement** checks every time you change the judge model
  or rubric.
- Treat **~75% agreement** as the threshold below which the judge needs
  recalibration.

### 2.7 From development to production
The same scorers used offline run against **production traces** for continuous
quality monitoring and full-context failure debugging — one evaluation
definition spanning dev → CI/CD → production.

---

## 3. Formulas / metric definitions

MLflow's article is a practical guide; its quantitative metrics are rates and
ratios computed over a run rather than closed-form equations.

**Task Success Rate:**
```
Task Success Rate = (Successful tasks) / (Total tasks)
```

**Tool Call Accuracy:**
```
Tool Call Accuracy = (Correct tool calls) / (Total tool calls)
```

**Step Efficiency:**
```
Step Efficiency = (Optimal steps) / (Actual steps taken)
```

**SLA / latency and cost** (diagnostic layer):
```
Latency      = end-to-end task completion time
Token Cost   = input tokens + output tokens, priced per model
```

**Judge–human agreement (calibration):**
```
Agreement = (Judge labels matching human labels) / (Calibration-set size)
            recalibrate when Agreement < ~0.75
            calibration-set size ≈ 50–200 examples
```

**Scorer output forms:**
```
Code-based scorer → pass/fail or numeric
LLM judge         → Feedback(value, rationale)   # value may be pass/fail or graded
```

**Aggregate quality** for a dimension over a dataset is the mean (or pass-rate)
of its per-trace scorer outputs, e.g.:
```
Correctness rate = (Traces judged correct) / (Total traces)
```
