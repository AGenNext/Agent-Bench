# Contributing

Standard GitOps practices. Read [`platform/docs/DESIGN.md`](platform/docs/DESIGN.md)
first — it's the canonical map of the repo.

## What to open

| Situation | Open |
|---|---|
| **Bug** — something is wrong/broken | an **Issue** (label `bug`) |
| **Feature request** — something missing | an **Issue** (label `enhancement`) |
| **Question / proposal to discuss** | an **Issue** (label `discussion`) |
| **Small, obvious change** (typo, doc fix, small bug, small improvement) | **just a Pull Request** — no issue needed |
| **Larger / behavioral change** | open an Issue first to align, then a PR linking it |

Rule of thumb: **small thing → straight to a PR; anything that needs discussion
or is non-trivial → issue first.**

**In case of doubt, confirm with the user.** An agent may decide how far to
expand or reduce its working context (max = all nodes/files/logs in scope;
min = the current topic) — but when the scope, intent, or correct change is
ambiguous, it asks rather than guesses. Don't write something that might be
wrong; confirm first.

## Issues

**Bug:**
```
what happened:   <observed>
expected:        <expected>
repro:           <steps / command>
component:       <file path or DESIGN.md map row>
```

**Feature request:**
```
problem:         <what you can't do today>
proposal:        <what you want>
component:       <where it fits in DESIGN.md>
```

## Pull requests

1. **Branch** off the default branch; keep one concern per PR (small + focused).
2. **Open as a draft** until ready for review.
3. **Tests must pass:** `cargo test` and `cargo test --features server` stay
   green; behavior changes add tests.
4. **Keep `DESIGN.md` in sync** — update the component row if input/protocol/
   output changed (see the `PROPOSAL` block in DESIGN.md).
5. **Protocol/meaning changes are versioned**, not edited in place.
6. **Link the issue** it resolves (`Closes #123`) when there is one.
7. **Conventional commits** for messages (`feat:`, `fix:`, `docs:`, `chore:`,
   `test:`, `refactor:`).

## Frontend — standardized component library only

Agent-Bench is backend (Rust API) today. **If** any UI is added (e.g. a
leaderboard view):

- Use **only the shared/standardized component library** — no bespoke or one-off
  components. Frontend design is standardized across the org.
- The design system / component library is **owned in its respective repo** and
  published to the **artifact registry**; depend on the **versioned registry
  artifact** — don't fork, vendor, or re-implement it.
- Frontend code is TypeScript and limited to presentation/validation — no
  business logic (that lives in the backend/runtime).

## Review

A PR merges when: tests green, `DESIGN.md` accurate, scope focused, and (for
protocol changes) a version bump is included.

## Production release — owned & enforced by Agent-Deploy

Everything up to merge can be **agent-driven**: issues, PRs, tests, review, and
merge to the default branch. **Production release is not owned by this repo.**

- **Agent-Deploy** owns and **enforces** CI/CD, release/rollback, centralized
  governance validation, operational-readiness gates, and the **human-approval**
  step for production promotion. The already-defined rules live there and are
  enforced there — automatically, not by convention.
- A **human must approve** any production promotion (hard gate), via Agent-Deploy's
  protected deploy environment / required approver.
- Agents may *prepare* a release (cut a branch, draft notes, get CI green) but may
  **not** approve or trigger the production deploy.

Agent-Bench's part is to **conform** and to **supply the evidence** Agent-Deploy's
gates consume — the certified evaluation result (how good / what next). It does
not define or run the release process.

> The `platform/deploy/` manifests here are for local/dev and as a reference;
> production deployment, gating, and rule enforcement are Agent-Deploy's.
