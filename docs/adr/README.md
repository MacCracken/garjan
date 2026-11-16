# Architecture Decision Records

Decisions about garjan — what we chose, the context, and the consequences we accept. Use these when a future reader would reasonably ask *"why did we do it this way?"*

## Conventions

- **Filename**: `NNNN-kebab-case-title.md`, zero-padded to four digits. Never renumber.
- **One decision per ADR.** If a decision supersedes a prior one, add a new ADR and set the old one's status to `Superseded by NNNN`.
- **Status lifecycle**: `Proposed` → `Accepted` → (optionally) `Superseded` or `Deprecated`.
- Use [`template.md`](template.md) as the starting point.

## ADR vs. architecture note vs. guide

| Kind | Lives in | Answers |
|---|---|---|
| ADR | `docs/adr/` | *Why did we choose X over Y?* |
| Architecture note | `docs/architecture/` | *What non-obvious constraint is true about the code?* |
| Guide | `docs/guides/` | *How do I do X?* |

## Index

- [0001 — Modal synthesis](0001-modal-synthesis.md) — Accepted
- [0002 — Dual code paths (naad + fallback)](0002-dual-code-paths.md) — Accepted. **Superseded in practice by the port**: the Cyrius port always uses the naad backend and dropped the fallback, so the `naad-backend` feature gate this ADR describes no longer exists. Retained as the record of why the Rust crate had two paths.
- [0003 — Scope boundaries with sibling crates](0003-scope-boundaries.md) — Accepted. The boundary table (garjan owns sources; goonj propagation, dhvani mixing, prani/svara vocal, ghurni mechanical).
- [0004 — Deterministic synthesis](0004-deterministic-synthesis.md) — Accepted
- [0005 — Allocation failure is an error code, not an abort](0005-allocation-failure-is-an-error-code-not-an-abort.md) — Accepted, 2026-08-30
- [0006 — Out-of-range enum ids are rejected, not absorbed](0006-out-of-range-enum-ids-are-rejected.md) — Accepted, 2026-08-30
- [0007 — Duration and sample rate are bounded](0007-bounded-duration-and-sample-rate.md) — Accepted, 2026-08-30
- [0008 — serde carries live DSP state](0008-serde-carries-live-dsp-state.md) — Accepted, 2026-08-30. Closes architecture note 001.
- [0009 — A NaN event rate means no events](0009-a-nan-event-rate-means-no-events.md) — Accepted, 2026-11-07. A deliberate divergence: `rust-old` hangs on this input.
- [0010 — A document restores only states the public API can produce](0010-a-document-restores-only-states-the-api-can-produce.md) — Accepted, 2026-11-07. Deserialize refuses or re-clamps what no public call reaches, and diverges from `rust-old` where serde accepted it.
- [0011 — Water's Stream LFO follows the raw intensity](0011-water-stream-lfo-follows-the-raw-intensity.md) — Accepted, 2026-11-07. A parity fix that moves output on valid input outside 0..1.
- [0012 — soorat's visualization data is ported ungated, with bounded sizes](0012-soorat-visualization-data-is-ported-ungated.md) — Accepted, 2026-11-16. The last Rust module, ported without waiting on soorat: flattened arrays, a 2^20-point cap, and loaders that refuse a missing member.
- [0013 — ModalBank and Exciter serialize standalone; a bank's document carries its modes](0013-modal-bank-and-exciter-serialize-standalone.md) — Accepted, 2026-11-16. A load rebuilds the bank through `modal_bank_new`, so its coefficients are re-derived bit for bit; an exciter refuses a negative length and the stuck-active state.
- [0014 — garjan's documents are not `rust-old`'s](0014-garjan-documents-are-not-rust-old-documents.md) — Accepted, 2026-11-16. Integer enums, flattened arrays and nested structs, a `"dsp"` member and f64 values: the table of differences, and no converter.

> **Format note.** 0001-0004 were written before these conventions and before
> the port; they use `## Status` / `Accepted (vN)` headings rather than the
> `**Status**` / `**Date**` form in [`template.md`](template.md). **Read them as
> historical records**: their version references are to the pre-port Rust line,
> and they cite `.rs` paths and Rust concepts (`no_std`, feature gates) that no
> longer exist — ADR-0002's dual code paths in particular were dropped by the
> port. The decisions they record still stand; the file paths do not. They were relocated here
> from `docs/architecture/` (where they had been filed against the README's own
> rule) without renumbering or reformatting — the content is a historical
> record, and rewriting it would blur what was decided when. New ADRs follow
> the template.
