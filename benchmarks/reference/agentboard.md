# AgentBoard — Glossary, Methods & Formulas

Reference notes distilled from:

> Chang Ma, Junlei Zhang, Zhihao Zhu, Cheng Yang, Junxian He, Yujiu Yang,
> Yaohui Jin, Zhenzhong Lan, Lingpeng Kong. *AgentBoard: An Analytical
> Evaluation Board of Multi-turn LLM Agents.* NeurIPS 2024 Datasets &
> Benchmarks Track.
> https://proceedings.neurips.cc/paper_files/paper/2024/hash/877b40688e330a0e2a3fc24084208dfa-Abstract-Datasets_and_Benchmarks_Track.html

**One-line summary.** Final success-rate hides what agents actually do in hard,
partially-observable, multi-turn tasks (most models score ~0%). **AgentBoard**
introduces a **fine-grained progress-rate** metric (incremental advancement
toward the goal), a unified benchmark of **9 tasks / 1013 environments** across
embodied, game, web, and tool agents, and an analytical toolkit for
multi-faceted, interpretable evaluation.

---

## 1. Glossary

| Term | Definition |
|---|---|
| **LLM agent** | An LLM that perceives an environment and acts step-by-step through multi-round interaction to achieve a goal. |
| **Partially-observable environment** | An environment where the agent sees only local observations, not full state — contrasts with fully-observable QA benchmarks (MMLU, GSM8K). |
| **Multi-round interaction** | Repeated act→observe loops needed to reach a goal, vs. single prompt–response. |
| **POMDP** | Partially Observable Markov Decision Process — the formal model of each task: tuple ⟨g, S, A, O, T⟩. |
| **Goal (g)** | The target the agent must achieve; may be decomposed into subgoals. |
| **State space (S)** | The set of possible environment states `sₜ`. |
| **Action space (A)** | The set of valid actions `aₜ` the agent may take. |
| **Observation space (O)** | What the agent perceives, including environment feedback. |
| **Transition function (T)** | `T: S × A → S`, how the environment evolves given an action. |
| **Feedback function (f)** | `oₜ = f(sₜ, aₜ)` — derives the observation/feedback each round (lists valid actions, executes action, reports state change or error). |
| **Trajectory (τ)** | The sequence `[s₀, a₀, s₁, a₁, …, sₜ]` produced by the agent's policy and environment transitions. |
| **Memory (mₜ)** | The agent's accumulated history of observations/actions used to predict the next action. |
| **Policy (π)** | The agent's action-selection rule given goal and memory. |
| **Reflex agent** | The baseline agent design that iteratively interacts with the environment using memory. |
| **Success rate** | Proportion of tasks fully completed — the standard but coarse metric; uninformative when most agents score ~0%. |
| **Progress rate (rₜ)** | AgentBoard's fine-grained metric: the highest matching score toward the goal achieved so far, in `[0,1]`. |
| **Matching score (f(·,g))** | Function quantifying similarity between the current state and the goal state, in `[0,1]` (continuous) or `{0,1}` (subgoal). |
| **Subgoal (gᵢ)** | A manually-labeled intermediate milestone; the goal decomposes into an ordered sequence `g = [g₁,…,g_K]`. |
| **Subgoal progress rate (r_subgoal)** | Progress measured by how many labeled subgoals are completed, via regex-based matching `f(·,gᵢ) → {0,1}`. |
| **Grounding accuracy** | Percentage of agent actions that are *valid* — i.e., correctly mapping high-level plans to executable, correctly-formatted actions. |
| **Sub-skill analysis** | Performance breakdown across abilities: memory, planning, spatial navigation, grounding, world modeling, self-reflection. |
| **Hard/Easy breakdown** | Performance split by example difficulty, exposing differences hidden in aggregate success rate. |
| **Long-range interaction** | Analysis of how progress evolves over many steps (progress rate w.r.t. step). |

### The four task categories / nine environments
- **Embodied AI:** AlfWorld, ScienceWorld, BabyAI
- **Game:** Jericho, PDDL
- **Web:** WebShop, WebArena
- **Tool:** Tool-Query, Tool-Operation

---

## 2. Methods

### 2.1 Design principles
AgentBoard is built on five principles: **task diversity & uniformity**,
**multi-round interaction**, **partially-observable environments**,
**fine-grained progress rate**, and **analytical (multi-faceted) evaluation**.

### 2.2 Formal task model (POMDP)
Each task is a POMDP ⟨g, S, A, O, T⟩. The agent with policy π predicts action
`aₜ` from goal `g` and memory `mₜ = {o_j, a_j, …, oₜ}`. Environment feedback is
`oₜ = f(sₜ, aₜ)`, which (1) lists valid actions on request, and (2) executes a
valid action and describes the resulting state change (or error).

### 2.3 Fine-grained progress rate
Because hard tasks yield near-zero success rates (treating all failures as
equal), AgentBoard assigns each round a **progress rate `rₜ ∈ [0,1]`** =
the highest matching score `f(s,g)` achieved up to step `t`. Two instantiations:
- **Continuous** matching score `f(·,g) → [0,1]` where state similarity is measurable.
- **Subgoal-based** `r_subgoal`: decompose `g` into an ordered subgoal sequence
  `[g₁,…,g_K]`, each with a labeled completion state; a regex matcher
  `f(·,gᵢ) → {0,1}` checks completion. Authors manually label and verify
  subgoals, enforcing a unique subgoal sequence (while allowing diverse
  trajectories). Games are capped at ≤15 subgoals.

### 2.4 Analytical toolkit
A Wandb-based interactive panel provides: (1) fine-grained progress tracking,
(2) grounding accuracy, (3) hard/easy performance breakdown, (4) long-range
interaction (progress vs. step), (5) sub-skill scores, (6) trajectory
visualization.

### 2.5 Key findings
- Progress rate separates models that look identical under success rate
  (near-zero success but different progress).
- Proprietary models (GPT-4, Claude2) lead; many open-weight models have notably
  **lower grounding accuracy** (e.g. Text-Davinci-003 ~58.9% avg) yet can still
  do well on planning/other sub-skills — showing why multi-dimensional analysis
  matters.

---

## 3. Formulas / metric definitions

**Task as POMDP:**
```
Task = ⟨ g, S, A, O, T ⟩,   T : S × A → S
Feedback:  oₜ = f(sₜ, aₜ)
Memory:    mₜ = { o_j, a_j, o_{j+1}, …, oₜ },  0 ≤ j < t
Trajectory: τ = [s₀, a₀, s₁, a₁, …, sₜ]
```

**Progress rate (continuous matching score):**
```
rₜ = max_{0 ≤ i ≤ t}  f(sᵢ, g) ,     f(·, g) → [0,1]
   r₀ = 0  (no progress),   rₜ = 1  ⟺ task complete
```
The progress rate is the *highest* matching score achieved so far (monotone
non-decreasing), so transient regressions don't lower it.

**Subgoal-based progress rate:**
```
g = [g₁, …, g_K]                      # ordered subgoals
f(·, gᵢ) → {0,1}                       # regex match: subgoal i complete?
r_subgoal = (number of completed subgoals) / K
          = max over visited states of  (#subgoals satisfied) / K
```

**Success rate:**
```
Success Rate = (Tasks fully completed) / (Total tasks)
```

**Grounding accuracy:**
```
Grounding Accuracy = (Valid actions issued) / (Total actions issued) × 100%
```
