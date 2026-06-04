# Deploying the platform (k3s / Kubernetes)

Fixes the `surrealdb` CrashLoopBackOff and wires the API to a shared SurrealDB.

## Why the original deploy crashlooped

`surrealdb/surrealdb:latest` with `replicas: 3` fails for two reasons:

1. **No start command** — the image entrypoint needs an explicit
   `start --bind ... --user ... --pass ... <storage>`; without it the container
   exits and Kubernetes restarts it forever.
2. **3 replicas on a single-node store** — RocksDB/SurrealKV/memory are
   **single-node**. Three replicas over one RWO volume cannot share state and
   crashloop. HA needs a **distributed storage tier** (SurrealDB Cloud Dedicated
   today; self-hosted **SurrealDS** when it ships) — not a self-managed cluster.

## Apply

```bash
# 1. Single durable SurrealDB node (correct for getting green):
kubectl apply -f surrealdb.yaml

# 2. Build & push the API image, then deploy it:
docker build -f ../Dockerfile -t ghcr.io/agennext/agentbench-platform:latest ../..
docker push ghcr.io/agennext/agentbench-platform:latest
kubectl apply -f platform.yaml

# 3. Verify:
kubectl -n surrealdb  rollout status statefulset/surrealdb
kubectl -n agentbench rollout status deployment/agentbench-platform
kubectl -n agentbench port-forward svc/agentbench-platform 8080:80
curl localhost:8080/health
```

## Architecture note

- The API is **stateless** → scale `replicas` freely; all state lives in
  SurrealDB. It connects via `AGENTBENCH_DB_URL=ws://surrealdb.surrealdb:8000`.
- The same binary also runs **fully embedded** (no server) when `AGENTBENCH_DB_URL`
  is unset — used for tests and single-node dev.
- **Scaling SurrealDB:** keep it at 1 replica with the PVC (RocksDB, the
  recommended on-disk server engine). For HA, point the platform at **SurrealDB
  Cloud Dedicated**, or run query nodes over **SurrealDS** when it ships — see
  the HA note in `surrealdb.yaml`. SurrealDB splits compute from storage, so this
  is a deployment change, not an application change.
