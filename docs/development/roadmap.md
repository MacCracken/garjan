# garjan — Roadmap

> Milestone plan through v1.0. State lives in [`state.md`](state.md);
> this file is the sequencing — what ships, in what order, against
> what dependency gates.

## v1.0 criteria

_Define before tagging v0.1.0:_

- [ ] Rust → Cyrius surface parity verified (function-level diff against `rust-old/`)
- [ ] Test coverage adequate for the surface area
- [ ] Benchmarks captured in `docs/benchmarks.md`
- [ ] At least one downstream consumer green
- [ ] CHANGELOG complete from v0.1.0 onward
- [ ] Security audit pass (`docs/audit/YYYY-MM-DD-audit.md`)

## Milestones

### M0 — Port scaffold (v0.1.0) — ✅ shipped 2026-07-03

- `cyrius port` scaffold landed
- Rust source moved to `rust-old/`
- Doc-tree per [first-party-documentation.md](https://github.com/MacCracken/agnosticos/blob/main/docs/development/applications/first-party-documentation.md)

### M1 — Surface parity (v0.2.0)

_Pick a parseable Rust subset and verify the Cyrius port matches it function-for-function. Specify the dep gates and the acceptance criteria._

### M2 — _Title_ (v0.3.0)

_…_

## Out of scope (for v1.0)

_Capture what's deliberately NOT in scope for v1.0._
