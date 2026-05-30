# AWS Bedrock AgentCore Evaluations — Glossary, Methods & Formulas

Reference notes distilled from:

> *Evaluate agent performance with Amazon Bedrock AgentCore Evaluations.*
> AWS Bedrock AgentCore Developer Guide.
> https://docs.aws.amazon.com/bedrock-agentcore/latest/devguide/evaluations.html

**One-line summary.** AgentCore Evaluations automatically scores how well an
agent performs tasks, handles edge cases, and stays consistent. It provides
**13 built-in LLM-judge evaluators** plus custom evaluators, supports
**ground-truth / trajectory** scoring, and runs both **online** (sampling live
production traces) and **on-demand** (CI/CD and interactive testing).

---

## 1. Glossary

| Term | Definition |
|---|---|
| **AgentCore Evaluations** | AWS managed service that assesses agent quality, safety, task completion, and tool usage from traces. |
| **Evaluator** | A scoring function applied to an agent's trace/output. Built-in or custom. |
| **Built-in evaluator** | Pre-configured LLM-as-a-judge evaluator with a crafted prompt template, selected judge model, and standardized scoring criteria (13 available). |
| **Custom evaluator** | A user-defined evaluator (LLM-based or code-based) for use-case-specific dimensions. |
| **LLM-as-a-judge** | The mechanism behind built-in evaluators: an LLM grades agent behavior. |
| **Trace** | The recorded execution of an agent run (inputs, steps, tool calls, output) scored by evaluators. |
| **Ground Truth** | Reference expectations to measure against: reference answers, behavioral assertions, expected tool-execution sequences. |
| **Behavioral assertion** | A session-level expectation about what the agent should do/achieve. |
| **Expected tool trajectory** | The reference sequence of tool calls the agent should make. |
| **Trajectory evaluation** | Scoring the agent's path (tool calls, steps), not just the final answer. |
| **Goal success rate / goal attainment** | Whether the agent achieves the end-to-end task objective. |
| **Tool selection accuracy** | Whether the agent invoked the correct tool(s) for the request. |
| **Context relevance** | Whether retrieved/used context is relevant to the request. |
| **Online evaluation** | Continuous monitoring that samples and scores live production traces. |
| **On-demand evaluation** | Programmatic evaluation for regression testing in CI/CD and interactive development. |
| **Dataset evaluation / batch evaluation** | Evaluating an agent over a dataset of sessions, optionally with ground-truth metadata. |

### The built-in evaluator dimensions (13)
Common quality dimensions covered include: **correctness**, **helpfulness**,
**tool selection accuracy**, **safety**, **goal success rate**, and **context
relevance** (plus additional response-quality, task-completion, and tool-usage
evaluators).

---

## 2. Methods

### 2.1 What it measures
Automated assessment of how well an agent/tool performs specific tasks, handles
edge cases, and maintains consistency across inputs and contexts — spanning
response quality, safety, task completion, and tool usage.

### 2.2 Evaluator types
- **Built-in evaluators** — 13 pre-configured LLM-judge evaluators with fixed
  prompt templates, judge models, and scoring criteria. Best for common needs.
- **Custom evaluators** — define bespoke metrics; can be **code-based**
  (deterministic) or LLM-based, for dimensions specific to your agent.

### 2.3 Ground truth & trajectory scoring
Provide Ground Truth via session metadata to enable reference-based scoring:
- **reference answers** for response validation,
- **behavioral assertions** for session-level goals,
- **expected tool trajectories** for tool-sequence validation.
Evaluators can then compute goal attainment, tool-invocation accuracy, and any
custom metric.

### 2.4 Evaluation modes
- **Online evaluation** — continuously samples and scores live production traces
  to monitor quality and catch regressions in deployment.
- **On-demand evaluation** — run programmatically for CI/CD regression gates and
  interactive development.

### 2.5 Recommended practice
Pair deterministic custom evaluators (fast, reproducible CI/CD gates) with
built-in LLM judges (nuanced quality), validate against ground-truth
trajectories, and run online evaluation in production for drift detection.

---

## 3. Formulas / metric definitions

AgentCore reports rates/ratios over an evaluation run; judge dimensions return
scores (often pass/fail or graded) aggregated to a rate.

**Goal success rate (end-to-end task completion):**
```
Goal Success Rate = (Sessions achieving the goal) / (Total sessions)
```

**Tool selection accuracy:**
```
Tool Selection Accuracy = (Correct tool invocations) / (Total tool invocations)
```

**Tool trajectory match (vs. expected trajectory):**
```
Trajectory Match = (Tool calls matching expected sequence) / (Expected tool calls)
```

**Built-in judge dimension rate** (correctness, helpfulness, safety, context
relevance, …):
```
DimensionScore = mean over traces of judge(trace)        # graded
   or
DimensionRate  = (Traces passing the dimension) / (Total traces)   # pass/fail
```

**Online sampling:**
```
Sampled traces scored continuously; quality metric = running mean of
evaluator outputs over the sampling window.
```
