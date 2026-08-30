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

- [0005 — Allocation failure is an error code, not an abort](0005-allocation-failure-is-an-error-code-not-an-abort.md) — Accepted, 2026-08-30.

> **Numbering note.** 0001-0004 are taken by `adr-001`..`adr-004`, which live in
> [`../architecture/`](../architecture/) under an older `adr-NNN-` scheme that
> predates these conventions. They are decisions and belong here; they have not
> been moved because "never renumber" makes relocation a separate, deliberate
> change. New ADRs start at 0005 and follow the conventions above.
