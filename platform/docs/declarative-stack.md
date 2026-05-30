# The Declarative Stack — One Language for Humans, Agents & Infra

Everything is declarative, and each layer is a graph expressed in a form that
**humans read, machines apply, and agents can reason over**. The point is a
*shared semantic contract*: like **schema.org JSON-LD**, one linked vocabulary
that every actor speaks.

```
Terraform   ── the infra/resource graph + shared ontology   (cloud, DNS, IAM, clusters)
   │            schema.org-style: a linked vocabulary humans + agents + infra all speak
Helm        ── the application graph on the cluster          (services + dependencies + wiring)
EvaluationPolicy (CRD)
            ── the desired evaluation state                  (which agents scored on which bench/hardware)
GitOps (Argo CD/Flux)
            ── reconciles git → cluster                       (the benchmark is the gate)
```

## Terraform as the shared schema (schema.org / JSON-LD analogy)

Terraform's declarative resource graph is the **lingua franca** of the platform:

- **Human-readable** — HCL describes intent: "this cluster, this DB, these roles."
- **Machine-applicable** — providers turn it into real cloud resources, with a
  state graph of dependencies (like JSON-LD's typed nodes + edges).
- **Agent-readable** — because it's a typed, linked description (resources have
  schemas, references form a graph), an **agent can introspect and reason** about
  the infrastructure the same way it reasons over schema.org data: "what stores
  exist, what enforces policy, where does telemetry go?"

This is the AgentField thesis taken to the infra layer: *identity-aware,
declarative, governed* — the infrastructure describes itself in a vocabulary
agents understand, so automation (and the agents under test) operate against a
self-describing system rather than opaque endpoints.

## Layer responsibilities

| Layer | Form | Speaks | Owns |
|---|---|---|---|
| **Terraform** | HCL / resource graph (schema.org-like ontology) | humans · agents · cloud APIs | cloud infra, clusters, IAM, DNS, the shared schema |
| **Helm** | chart (values + templates) | humans · k8s | app topology: API, SurrealDB, ClickHouse, auth plane, sandbox |
| **EvaluationPolicy CRD** | YAML | humans · controller | desired eval state (`reconciliation.md`) |
| **GitOps** | git + Argo CD/Flux | humans · cluster | sync + fail-closed gates (`reconciliation.md`) |

## Why this matters for Agent-Bench

- **Self-describing benchmarks**: a run's environment (hardware, DSL, store,
  enforcement) is declared in the same graph, so result packages can cite the
  exact infra coordinates → stronger reproducibility manifests.
- **Agents reason about their substrate**: an agent-under-test (or an operator
  agent) can query the declarative graph to understand capabilities and limits,
  instead of hard-coding assumptions.
- **One source of truth**: human review, machine apply, agent reasoning, and
  GitOps gating all operate over the *same* declarative artifacts — no drift
  between "what we said" and "what runs."

## Status

Conceptual layering; the scoring core + API run today. Terraform modules and the
Helm chart are the IaC layer to add (see `deploy/`).
