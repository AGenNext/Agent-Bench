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

## Review

A PR merges when: tests green, `DESIGN.md` accurate, scope focused, and (for
protocol changes) a version bump is included.
