-- =============================================================================
-- Seed: the multi-cloud cluster registry. Proves "adding a cloud is a row, not
-- code" — the same workload places onto any of these via the one Runtime
-- contract (OCI image + Kubernetes Job). Providers mirror SurrealDB's own
-- self-hosted matrix (Docker, AKS/EKS/GKE, KIND) deployed via the Helm chart.
-- A cluster is only trusted for placement once it is `proven_by` the required
-- conformance checks (build/deployment/self-hosted/overview).
-- =============================================================================

UPSERT cluster:`eks-us-east-1`  SET name='eks-us-east-1',  provider='eks',    region='us-east-1',      endpoint='https://eks.us-east-1.amazonaws.com', healthy=false, capacity={ cpu: '64', mem: '256Gi' };
UPSERT cluster:`gke-us-central` SET name='gke-us-central', provider='gke',    region='us-central1',    endpoint='https://gke.us-central1.googleapis.com', healthy=false, capacity={ cpu: '64', mem: '256Gi' };
UPSERT cluster:`aks-westeu`     SET name='aks-westeu',     provider='aks',    region='westeurope',     endpoint='https://aks.westeurope.azmk8s.io',    healthy=false, capacity={ cpu: '48', mem: '192Gi' };
UPSERT cluster:`kind-local`     SET name='kind-local',     provider='kind',   region='local',          endpoint='https://127.0.0.1:6443',              healthy=false, capacity={ cpu: '4',  mem: '8Gi'   };

-- Each cluster must pass the required conformance suite before healthy=true.
-- (The cluster-conformance workload runs the suite and flips this; here we wire
-- the proving edges so the graph is complete and crawlable.)
RELATE cluster:`eks-us-east-1`->proven_by->conformance_check:`pvc-bind`;
RELATE cluster:`eks-us-east-1`->proven_by->conformance_check:`oom-handling`;
RELATE cluster:`eks-us-east-1`->proven_by->conformance_check:`manifest-apply`;
RELATE cluster:`gke-us-central`->proven_by->conformance_check:`pvc-bind`;
RELATE cluster:`gke-us-central`->proven_by->conformance_check:`oom-handling`;
RELATE cluster:`gke-us-central`->proven_by->conformance_check:`manifest-apply`;
RELATE cluster:`aks-westeu`->proven_by->conformance_check:`pvc-bind`;
RELATE cluster:`aks-westeu`->proven_by->conformance_check:`oom-handling`;
RELATE cluster:`aks-westeu`->proven_by->conformance_check:`manifest-apply`;
RELATE cluster:`kind-local`->proven_by->conformance_check:`manifest-apply`;
