# AxiaVote Retirement Notice

> AxiaVote is retired and replaced by VoteOS.

---

## Why AxiaVote Is Being Replaced

AxiaVote was an early attempt at election functionality within the Axia ecosystem.
It is being retired because:

1. **Not sovereign** — it was too tightly coupled to AxiaSystem internals
2. **Not auditable enough** — lacked the strict proof discipline proven in CivilOS
3. **Not interoperable** — did not account for shared-truth deployment with CivilOS
4. **Not strict enough** — did not enforce ballot integrity, privacy, or ambiguity handling
5. **Not modular** — monolithic design that couldn't be extended cleanly

## What VoteOS Does Differently

| Concern | AxiaVote | VoteOS |
|---------|----------|--------|
| Architecture | Monolithic | Module-based (6 domains) |
| Identity | Own identity management | AxiaSystem shared truth |
| Build discipline | Ad-hoc | Harness-driven (proven in CivilOS) |
| Testing | Basic | Election-integrity-gated |
| Auditability | Partial | Full evidence reconstruction |
| Privacy | Implicit | Explicit ballot secrecy model |
| Interoperability | None | CivilOS + AxiaSystem coexistence |
| Ambiguity | Suppressed | Explicitly handled |

## Migration

No code from AxiaVote is carried into VoteOS.
VoteOS is a clean-room implementation using proven build discipline from CivilOS.

If legacy AxiaVote data needs migration, VoteOS will handle it through a wrapper layer
(similar to CivilOS's migration wrapper), designed specifically for election data migration.
