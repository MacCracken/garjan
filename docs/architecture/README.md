# Architecture notes

Non-obvious constraints, quirks, and invariants that a reader cannot derive from the code alone. Numbered chronologically — never renumber.

Not decisions (those live in [`../adr/`](../adr/)) and not guides (those live in [`../guides/`](../guides/)). An item here describes *how the world is*, not *what we chose* or *how to do something*.

## Items

- [001 — Deserialize does not restore DSP state (and Rust's did)](001-deserialize-does-not-restore-dsp-state.md) — `*_from_json_str` rebuilds naad components from scalars instead of restoring their live state. Rust serialized that state; the port does not, and the source comments claiming a `#[serde(skip)]` mirror were describing an attribute that does not exist in `rust-old/`.

Add a numbered entry (`NNN-kebab-case-title.md`) whenever the code has a non-obvious invariant a reader can't derive. Numbered chronologically — never renumber. Do not write entries for decisions — those are ADRs.
