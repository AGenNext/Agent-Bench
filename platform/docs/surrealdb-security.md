# SurrealDB-Native Security & Trust

SurrealDB enforces **[Security & Authorisation](https://surrealdb.com/features#security-and-authorisation)**
and **[Trust & Compliance](https://surrealdb.com/features#trust-and-compliance)**
at the data layer. We adopt these so the **database is a Policy Enforcement
Point** — it verifies the caller's identity and applies per-record permissions
itself, instead of trusting the application's `X-Tenant` header.

This is the strongest form of our principle *enforcement is owned by the
runtime, never the application*: even a buggy or compromised API cannot read
across tenants, because the DB rejects it.

## Identity comes from Agent-Auth

The JWT is issued by **Agent-Auth (casdoor)**. SurrealDB verifies it natively via
a JWT access method — no shared secret in the app, and the DB trusts only
Agent-Auth's signing key.

```surql
-- Verify Agent-Auth / casdoor tokens. The app no longer connects as root for
-- user requests; it forwards the end-user's JWT and SurrealDB authenticates it.
DEFINE ACCESS agent_auth ON DATABASE TYPE JWT
    ALGORITHM RS256 KEY "-----BEGIN PUBLIC KEY-----
    <casdoor JWKS / public key>
    -----END PUBLIC KEY-----";
```

Token claims are then available as `$token` (raw claims) and `$auth` (the
resolved record), e.g. `$token.tenant`, `$token.roles`, `$token.sub`.

## Defense in depth — three layers

| Layer | Mechanism | What it stops |
|---|---|---|
| Namespace | one SurrealDB **namespace per tenant** | cross-enterprise data access |
| Record permissions | `PERMISSIONS FOR … WHERE …` on every table | a user touching records they don't own / can't view |
| External authz | OpenFGA (Agent-Auth) via AuthZEN | rich relationship checks (team/role graphs) |

Layers 1–2 are SurrealDB-native; layer 3 is Agent-Auth. The same Agent-Auth
identity drives all three.

### Record-level permissions (target schema)

```surql
DEFINE TABLE agent SCHEMAFULL
  PERMISSIONS
    FOR select WHERE owner = $auth.id OR $token.roles CONTAINS "viewer"
    FOR create, update, delete WHERE owner = $auth.id
                                   OR $token.roles CONTAINS "admin";

DEFINE TABLE run SCHEMAFULL
  PERMISSIONS
    FOR select WHERE agent.owner = $auth.id OR $token.roles CONTAINS "viewer"
    FOR create WHERE $token.roles CONTAINS "submitter";
```

> System users (root/NS/DB) **bypass** record permissions — which is why the
> current root-connected tests still pass. Permissions apply to JWT/record auth,
> i.e. real user requests in production.

## RBAC for operators

```surql
DEFINE USER ops ON NAMESPACE PASSWORD "…" ROLES VIEWER;   -- read-only operator
DEFINE USER ci  ON DATABASE  PASSWORD "…" ROLES EDITOR;   -- migrations/CI
```
Roles: `OWNER` > `EDITOR` > `VIEWER`, scopable at ROOT / NAMESPACE / DATABASE.

## Trust & Compliance features we use

| Need | SurrealDB feature |
|---|---|
| **Audit trail** | **Change feeds** — `DEFINE TABLE run CHANGEFEED 30d;` then `SHOW CHANGES FOR TABLE run SINCE …` gives an immutable, replayable record of every mutation. Feeds Agent-Compliance evidence. |
| **Least-capability** | Server **capabilities** flags (`--deny-all` then allow-list scripting/funcs/net) — lock down what SurrealQL can do, important since we run near untrusted agents. |
| **Encryption in transit** | TLS on the server endpoint (`--web-crt/--web-key`); the platform connects over `wss://`. |
| **Encryption at rest** | provided by the storage layer (encrypted PVs via the CNCF storage stack). |
| **Token lifetime** | `DURATION FOR TOKEN`/`FOR SESSION` on access methods — short-lived tokens, continuous re-verification (zero-trust). |

## Everything lives in the database — and the database protects it

The database is not just the run store; it is the **protected system of record
for every asset**: code, generated artifacts, business/eval logic, **tool
definitions**, and **skill definitions** all live as records. SurrealDB's
multi-model engine fits them naturally:

| Asset | SurrealDB model |
|---|---|
| Code / artifacts | documents (+ blob refs by digest) |
| Logic / rules / policies | documents / functions |
| Tool definitions | documents with typed schemas |
| Skill definitions | documents + **graph** edges (skill → tools, skill → skill) |
| Memories / embeddings | **vector** fields for semantic recall |
| Relationships | **graph** edges (agent → skill → tool) |

Because access to every record is governed by the same `PERMISSIONS` + JWT layer,
**the database protects all of it uniformly** — a tool or skill definition is
guarded exactly like a run record. No separate ACL system per asset type; one
enforcement plane for the whole substrate.

### Database vs. registry — complementary

- **Database (SurrealDB)** = the *live, protected* substrate: queryable,
  permissioned, mutable-with-audit (change feeds). It **protects** (who may read
  /write each asset).
- **Registry (OCI/ORAS)** = the *immutable, signed* ledger: content-addressed
  artifacts + attestations. It **attests** (this exact bytes, signed, with
  provenance).

The DB holds and governs the asset and references the registry digest; the
registry proves immutability and signature. Together: the database protects
access, the registry proves integrity.

## Migration path

1. **Now (this PR):** schema + root connection; security definitions provided as
   a template (`migrations/0002_security.surql.template`), not auto-applied.
2. **Next:** the API forwards the end-user JWT to SurrealDB (`authenticate(jwt)`)
   instead of connecting as root for user requests; apply the JWT access method
   with Agent-Auth's real signing key; enable record permissions + change feeds.
3. **Result:** tenant isolation and per-record authz are enforced by SurrealDB,
   audited by change feeds, with identity owned by Agent-Auth — the app holds no
   authorization logic of its own.
