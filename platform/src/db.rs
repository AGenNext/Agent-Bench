//! SurrealDB storage with namespace-per-tenant isolation.
//!
//! Each enterprise tenant maps to its own SurrealDB **namespace** — SurrealDB's
//! native multi-tenancy primitive. The store uses the `any` engine, so the same
//! binary runs against an **embedded** engine (`memory`, `surrealkv://path`) or
//! a **remote** server (`ws://host:8000`) selected purely by connection string.
//! Migrations are versioned `.surql` files applied once per namespace and
//! tracked in a `_migration` table.

use std::collections::HashSet;
use std::sync::Arc;

use surrealdb::engine::any::Any;
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use tokio::sync::Mutex;

use crate::domain::{Agent, Benchmark, LeaderboardEntry, Run, SubmitRun};
use crate::error::{AppError, AppResult};
use crate::metrics::clear::ClearWeights;
use crate::scoring::{improvement_areas, score_run};

/// Ordered migrations embedded into the binary.
const MIGRATIONS: &[(&str, &str)] = &[("0001_init", include_str!("../migrations/0001_init.surql"))];

const DB_NAME: &str = "main";

/// Lowercase, replace non-alphanumerics with `_` — safe SurrealDB record keys.
fn slugify(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

/// Handle to the engine. A single `Mutex` serializes the
/// (switch-namespace → query) critical section so per-tenant access is safe on
/// the shared connection. (A per-tenant connection pool is the scale-up path.)
#[derive(Clone)]
pub struct Store {
    client: Arc<Surreal<Any>>,
    lock: Arc<Mutex<()>>,
    migrated: Arc<Mutex<HashSet<String>>>,
}

impl Store {
    /// Open an in-memory engine (ephemeral; ideal for tests and dev).
    pub async fn memory() -> AppResult<Self> {
        Self::connect("memory", None).await
    }

    /// Open a persistent SurrealKV engine at `path`.
    pub async fn surrealkv(path: &str) -> AppResult<Self> {
        Self::connect(&format!("surrealkv://{path}"), None).await
    }

    /// Connect to any SurrealDB endpoint (embedded or remote).
    ///
    /// * `memory`                  — embedded, ephemeral
    /// * `surrealkv://./data`      — embedded, persistent
    /// * `ws://surrealdb:8000`     — remote server (k8s), requires `creds`
    ///
    /// For remote endpoints, `creds` must be `Some((user, pass))` to sign in as
    /// root; embedded engines ignore credentials.
    pub async fn connect(endpoint: &str, creds: Option<(&str, &str)>) -> AppResult<Self> {
        let client = surrealdb::engine::any::connect(endpoint).await?;

        let remote = endpoint.starts_with("ws") || endpoint.starts_with("http");
        if remote {
            if let Some((username, password)) = creds {
                client.signin(Root { username, password }).await?;
            }
        }
        Ok(Self {
            client: Arc::new(client),
            lock: Arc::new(Mutex::new(())),
            migrated: Arc::new(Mutex::new(HashSet::new())),
        })
    }

    /// Switch the connection to a tenant namespace, applying migrations on first
    /// touch. Caller must already hold `self.lock`.
    async fn enter_tenant(&self, tenant: &str) -> AppResult<()> {
        self.client.use_ns(tenant).use_db(DB_NAME).await?;

        let mut migrated = self.migrated.lock().await;
        if migrated.contains(tenant) {
            return Ok(());
        }
        for (name, sql) in MIGRATIONS {
            // Idempotent: skip if this migration is already recorded.
            let already: Option<serde_json::Value> = self
                .client
                .query("SELECT * FROM _migration WHERE name = $name")
                .bind(("name", name.to_string()))
                .await?
                .take(0)?;
            if already.is_some() {
                continue;
            }
            self.client.query(*sql).await?;
            self.client
                .query("CREATE _migration SET name = $name, applied_at = time::now()")
                .bind(("name", name.to_string()))
                .await?;
        }
        migrated.insert(tenant.to_string());
        Ok(())
    }

    /// Run an arbitrary read query returning a single scalar. Used by SurrealML
    /// inference; gated to keep the surface minimal when ML is off.
    #[cfg(feature = "surrealml")]
    pub async fn raw_query(
        &self,
        tenant: &str,
        sql: &str,
        binds: &[(&'static str, f64)],
    ) -> AppResult<Option<f64>> {
        let _g = self.lock.lock().await;
        self.enter_tenant(tenant).await?;
        let mut q = self.client.query(sql);
        for (k, v) in binds {
            q = q.bind((*k, *v));
        }
        let out: Option<f64> = q.await?.take(0)?;
        Ok(out)
    }

    // ---- agents -----------------------------------------------------------

    pub async fn create_agent(&self, tenant: &str, mut agent: Agent) -> AppResult<Agent> {
        // Deterministic record key from name+version so clients can reference it.
        let slug = slugify(&format!("{}-{}", agent.name, agent.version));

        let _g = self.lock.lock().await;
        self.enter_tenant(tenant).await?;
        self.client
            .query(
                "CREATE type::thing('agent', $slug) SET name = $name, \
                 scaffold = $scaffold, model = $model, version = $version",
            )
            .bind(("slug", slug.clone()))
            .bind(("name", agent.name.clone()))
            .bind(("scaffold", agent.scaffold.clone()))
            .bind(("model", agent.model.clone()))
            .bind(("version", agent.version.clone()))
            .await?
            .check()?;
        agent.id = Some(slug);
        Ok(agent)
    }

    pub async fn list_agents(&self, tenant: &str) -> AppResult<Vec<Agent>> {
        let _g = self.lock.lock().await;
        self.enter_tenant(tenant).await?;
        let rows: Vec<Agent> = self
            .client
            .query(
                "SELECT record::id(id) AS id, name, scaffold, model, version FROM agent",
            )
            .await?
            .take(0)?;
        Ok(rows)
    }

    // ---- benchmarks -------------------------------------------------------

    pub async fn upsert_benchmark(&self, tenant: &str, mut b: Benchmark) -> AppResult<Benchmark> {
        let _g = self.lock.lock().await;
        self.enter_tenant(tenant).await?;
        // Use the public benchmark_id as the record key (stable, idempotent).
        self.client
            .query(
                "UPSERT type::thing('benchmark', $bid) SET benchmark_id = $bid, \
                 name = $name, domain = $domain, task_count = $tc",
            )
            .bind(("bid", b.benchmark_id.clone()))
            .bind(("name", b.name.clone()))
            .bind(("domain", b.domain.clone()))
            .bind(("tc", b.task_count))
            .await?
            .check()?;
        b.id = Some(b.benchmark_id.clone());
        Ok(b)
    }

    pub async fn list_benchmarks(&self, tenant: &str) -> AppResult<Vec<Benchmark>> {
        let _g = self.lock.lock().await;
        self.enter_tenant(tenant).await?;
        let rows: Vec<Benchmark> = self
            .client
            .query(
                "SELECT record::id(id) AS id, benchmark_id, name, domain, task_count \
                 FROM benchmark",
            )
            .await?
            .take(0)?;
        Ok(rows)
    }

    // ---- runs -------------------------------------------------------------

    /// Submit a run: score it with the metrics engine and persist the result.
    pub async fn submit_run(&self, tenant: &str, req: SubmitRun) -> AppResult<Run> {
        let scores = score_run(&req.results, ClearWeights::default());

        let _g = self.lock.lock().await;
        self.enter_tenant(tenant).await?;

        let scores_json = serde_json::to_value(&scores)
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        let created: Vec<serde_json::Value> = self
            .client
            .query(
                "CREATE run SET agent = type::thing('agent', $agent), \
                 benchmark = type::thing('benchmark', $bench), \
                 status = 'scored', trials = $trials, scores = $scores \
                 RETURN record::id(id) AS id",
            )
            .bind(("agent", req.agent_id.clone()))
            .bind(("bench", req.benchmark_id.clone()))
            .bind(("trials", req.trials))
            .bind(("scores", scores_json))
            .await?
            .take(0)?;

        let id = created
            .into_iter()
            .next()
            .and_then(|v| v.get("id").and_then(|i| i.as_str()).map(str::to_string))
            .unwrap_or_default();

        Ok(Run {
            id,
            agent_id: req.agent_id,
            benchmark_id: req.benchmark_id,
            status: "scored".into(),
            trials: req.trials,
            scores,
        })
    }

    /// Leaderboard for a benchmark: agents ranked by composite CLEAR score,
    /// with per-agent improvement areas. Uses SurrealQL aggregation to pick each
    /// agent's best run, then ranks in Rust.
    pub async fn leaderboard(&self, tenant: &str, benchmark_id: &str) -> AppResult<Vec<LeaderboardEntry>> {
        let _g = self.lock.lock().await;
        self.enter_tenant(tenant).await?;

        // Join runs to their agent, newest best run per agent.
        let rows: Vec<serde_json::Value> = self
            .client
            .query(
                "SELECT agent.name AS agent_name, agent.scaffold AS scaffold, \
                 record::id(agent) AS agent_id, scores \
                 FROM run \
                 WHERE benchmark.benchmark_id = $bid AND status = 'scored'",
            )
            .bind(("bid", benchmark_id.to_string()))
            .await?
            .take(0)?;

        let mut entries: Vec<LeaderboardEntry> = rows
            .into_iter()
            .map(|v| {
                let scores = v.get("scores").cloned().unwrap_or_default();
                let run_scores: crate::domain::RunScores =
                    serde_json::from_value(scores).unwrap_or_default();
                LeaderboardEntry {
                    rank: 0,
                    agent_id: v
                        .get("agent_id")
                        .and_then(|i| i.as_str())
                        .unwrap_or_default()
                        .to_string(),
                    agent_name: v
                        .get("agent_name")
                        .and_then(|s| s.as_str())
                        .unwrap_or("")
                        .to_string(),
                    scaffold: v
                        .get("scaffold")
                        .and_then(|s| s.as_str())
                        .unwrap_or("")
                        .to_string(),
                    efficacy: run_scores.clear.efficacy,
                    cna: run_scores.clear.cna,
                    clear_composite: run_scores.clear_composite,
                    improvement_areas: improvement_areas(&run_scores, 0.7),
                }
            })
            .collect();

        // Rank by composite score (desc); ties broken by efficacy.
        entries.sort_by(|a, b| {
            b.clear_composite
                .partial_cmp(&a.clear_composite)
                .unwrap()
                .then(b.efficacy.partial_cmp(&a.efficacy).unwrap())
        });
        for (i, e) in entries.iter_mut().enumerate() {
            e.rank = (i + 1) as u32;
        }
        Ok(entries)
    }
}
