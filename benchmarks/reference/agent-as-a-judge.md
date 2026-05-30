# Agent-as-a-Judge & DevAI — Glossary, Methods & Formulas

Reference notes distilled from:

> Zhuge, Zhao, Ashley, Wang, Khizbullin, Xiong, Liu, Chang, Krishnamoorthi,
> Tian, Shi, Chandra, Schmidhuber. *Agent-as-a-Judge: Evaluate Agents with
> Agents.* arXiv:2410.10934v2 (Meta AI / KAUST), 2024.

**One-line summary.** Final-outcome evaluation ignores the step-by-step nature
of agents, while human evaluation is costly. **Agent-as-a-Judge** uses an
agentic system to evaluate agentic systems, giving intermediate, process-level
feedback. On the new **DevAI** benchmark it aligns with human consensus (~90%),
rivals human evaluation, and saves ~97% of evaluation time and cost vs. humans.

---

## 1. Glossary

| Term | Definition |
|---|---|
| **Agent-as-a-Judge** | A framework where an agentic system evaluates another agent, providing intermediate feedback across the whole task-solving process. An agentic extension of LLM-as-a-Judge. |
| **LLM-as-a-Judge** | Using an LLM to grade outputs; lacks access to the agent's intermediate trajectory. |
| **Human-as-a-Judge** | Human expert evaluation of agent outputs; thorough but labor-intensive (the gold-standard baseline here). |
| **DevAI** | The introduced benchmark: 55 realistic automated AI-development tasks with rich manual annotations — 365 hierarchical requirements and 125 preferences. |
| **Requirement** | An atomic, checkable milestone a task must satisfy. Requirements form a Directed Acyclic Graph (DAG) with dependency edges. |
| **Preference** | A softer, optional requirement representing desirable-but-not-mandatory qualities. |
| **Requirement DAG** | Directed acyclic graph of requirements; dependencies mean a child requirement is meaningful only if its prerequisites are met. Yields non-sparse feedback vs. binary success. |
| **Independent (I) evaluation** | Scoring each requirement on its own, ignoring dependencies. |
| **Dependent (D) evaluation** | Scoring a requirement as met only if its prerequisite requirements are also met (stricter). |
| **Consensus evaluation** | The final human label, reached after evaluators debate to agreement; treated as the reference "ground truth." |
| **Judge shift** | The absolute disagreement of an automated judge from Human-as-a-Judge consensus. |
| **Judge alignment** | How closely a judge agrees with human consensus (Agent-as-a-Judge ≈ 90%). |
| **Black-box setting** | Judge sees only inputs/outputs, no intermediate trajectory. |
| **Gray-box setting** | Judge uses manually collected trajectory data. |
| **White-box testing** | Evaluation with full access to workspace, trajectories, and source code. |
| **Self-Termination** | Whether the agent stops on its own (vs. running until forced to halt). |
| **Task Solve Rate** | Fraction of whole tasks where *all* requirements are met. |
| **Agent-as-a-Judge modules** | Eight interacting components: **graph**, **locate**, **read**, **search**, **retrieve**, **ask**, **memory**, **planning**. |

### Agent-as-a-Judge modules
- **graph** — builds a project graph of files, modules, dependencies; can split code into snippets.
- **locate** — finds the folder/file a requirement refers to.
- **read** — reads/understands multimodal data across 33 formats (code, images, video, docs).
- **search** — contextual code retrieval of relevant snippets and hidden dependencies.
- **retrieve** — pulls relevant information from trajectories/history.
- **ask** — queries whether a requirement is satisfied.
- **memory** — stores intermediate judgments/context.
- **planning** — sequences the evaluation steps.

---

## 2. Methods

### 2.1 The evaluation problem
Agentic tasks are multi-step, so a single end-of-task success bit discards most
signal. DevAI instead checks each requirement in a dependency DAG, exposing
*where* an agent falters. Three judges are compared: Human-as-a-Judge,
LLM-as-a-Judge, and Agent-as-a-Judge.

### 2.2 DevAI benchmark
55 AI-development tasks, 365 hierarchical requirements (DAG with dependency
edges), 125 preferences. Agents are scored on how many requirements they
satisfy; preferences are optional softer targets.

### 2.3 Human-as-a-Judge baseline
Three expert evaluators review baseline agents (MetaGPT, GPT-Pilot, OpenHands)
in two rounds: (1) independent judgments with minimal instructions (~58 human
hours), then (2) debate to **consensus** (~28.5 additional hours). The consensus
is the reference label. Best baselines satisfy ~29% of requirements (~44%
ignoring prerequisites), but all requirements on only one task — confirming
DevAI is challenging but appropriate.

### 2.4 Agent-as-a-Judge framework
An agentic judge (the eight modules above) imitates the human review process:
build the project graph, locate the artifact for each requirement, read/verify
it (cross-referencing multimodal data and execution traces), and decide
satisfaction. Operates in black-box or gray-box settings.

### 2.5 Key results
- Agent-as-a-Judge aligns with human **consensus at ~90%**, far above
  LLM-as-a-Judge, and is as reliable as the human baseline.
- It saves **~97.72% of time** and **~97.64% of cost** vs. human evaluation.
- Aligns more closely with consensus than individual human evaluators do — i.e.
  it tracks the debated "truth" better than any single annotator.

---

## 3. Formulas / metric definitions

**Requirements Met — Independent (I):**
```
RequirementsMet(I) = (Requirements satisfied) / (Total requirements)
                     # each requirement judged on its own
```

**Requirements Met — Dependent (D):**
```
RequirementsMet(D) = (Requirements satisfied AND all prerequisites satisfied)
                     / (Total requirements)
```

**Task Solve Rate:**
```
Task Solve Rate = (Tasks with ALL requirements met) / (Total tasks)
```

**Self-Termination rate:**
```
Self-Termination = (Runs the agent ended by itself) / (Total runs)
```

**Judge alignment / judge shift (vs. human consensus):**
```
Alignment  = (Judge decisions matching consensus) / (Total decisions)
Judge Shift = | JudgeScore − ConsensusScore |        # absolute disagreement
```

**Evaluation efficiency (vs. Human-as-a-Judge):**
```
Time saving = 1 − (Judge time / Human time)   ≈ 97.72%
Cost saving = 1 − (Judge cost / Human cost)   ≈ 97.64%
```
