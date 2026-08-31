# Security Policy

## Scope

garjan is a pure sound-synthesis library. It performs no I/O and no network
access, and all synthesis is deterministic from seeded PRNG state.

It is written in **Cyrius**, which is unsafe by default: there is no borrow
checker, raw `load64`/`store64` are ordinary operations, and an out-of-range
index is not caught for you. The guarantees below come from validation the code
performs explicitly, not from the language.

## Trust boundary

garjan validates every caller-supplied value that can reach an allocation, a
division, or a table lookup. It does **not** defend against a compromised host
process; it defends against malformed **data**.

The highest-risk surface is `*_from_json_str`, the only entry point taking
fully attacker-controlled input. Two audits have focused there:
[2.0.2](docs/audit/2026-08-30-audit.md) and
[2.5.1](docs/audit/2026-08-30-audit-2.md).

## Attack surface

| Area | Risk | Mitigation |
|---|---|---|
| Sample rate | Division by zero, NaN propagation, absurd buffer sizes | Rejected unless finite and within 1 Hz – 768 kHz, in **every constructor and every deserialize path** ([ADR-0007](docs/adr/0007-bounded-duration-and-sample-rate.md)) |
| Duration | Unbounded allocation — 1e8 s once implied a 4.4-trillion-sample buffer | Rejected unless finite, positive and ≤ 600 s |
| Sample count | Two individually legal inputs can still imply an illegal buffer (600 s at 768 kHz = 461 M) | Checked separately at all 21 `*_synthesize` sites; capped at 44.1 M |
| Enum ids | Ported Rust enums are plain integers, so an invalid id would fall through an `if`/`elif` chain and silently select the last variant's table | `garjan_enum_invalid` at 21 constructors, 10 table dispatchers, and — since 2.5.1 — every deserialize path ([ADR-0006](docs/adr/0006-out-of-range-enum-ids-are-rejected.md)) |
| Deserialized JSON | Crafted documents reaching allocation loops or wild pointers | Errors from component construction are propagated (not discarded); collection sizes come from the array present, never from a scalar; `swarm_count` re-clamped to 1..8 |
| Serialized DSP state (`"dsp"`) | Type confusion and over-length arrays writing past a vector | Every loader checks the tag and length; writes bounded by the destination's own length. Tested against 12 hostile documents |
| Allocation failure | `alloc` returns `0`, and `0` is `GARJAN_OK` — a failure would pass every error check and be written through as a null pointer | All allocation goes through `garjan_alloc`, mapping `0` to `GARJAN_ERR_ALLOCATION` ([ADR-0005](docs/adr/0005-allocation-failure-is-an-error-code-not-an-abort.md)). **Invariant: no raw `alloc(` in `src/` outside `garjan_alloc`** |
| Poisson distribution | Near-infinite loop on a large rate | Rate clamped to 0–30 |
| Modal bank coefficients | Numerical instability if radius ≥ 1 | Radius clamped to [0.0, 0.9999] |
| DC blocker coefficient | Oscillation at very low sample rates | R clamped to [0.9, 0.9999] |
| Real-time setters | Out-of-range intensity/velocity/pressure | Clamped on assignment |
| Hot-path allocation | Unbounded growth in a long-running stream, since the arena never frees | `process_block` allocates nothing, pinned by test — see the known exception below |

## Known issues

- **`whistle_process_block` allocates 32 B/sample** (~5 GB/hour, exhausting the
  2 GiB arena in ~25 minutes of continuous streaming). naad's
  `filter_svf_process_sample` returns a heap `SvfOutput` and exposes no
  non-allocating band-pass variant. Not fixable inside garjan; avoid `whistle`
  in long-running streams until the upstream API lands.
- **The arena never frees.** `alloc_reset()` invalidates *every* outstanding
  pointer, so it is only usable at a clean epoch boundary. A caller that
  constructs and discards synths in a loop grows memory regardless.

## Failure behaviour

garjan never panics, aborts, or terminates the host process. Every failure is a
negative `GARJAN_ERR_*` code returned to the caller. Rust's allocator aborted on
OOM; the port deliberately does not.

## Dependencies

| Dependency | Purpose | Notes |
|---|---|---|
| `naad` | DSP primitives — noise, filters, LFOs | Consumed as a single dist bundle; no I/O |
| `hisab`, `goonj` | naad's transitive maths/acoustics deps | Resolved from this manifest so the flat namespace stays consistent |
| `sakshi` | Structured logging | Always compiled, gated at runtime; garjan installs no sink |
| Cyrius stdlib | `alloc`, `vec`, `str`, `bayan` (JSON), … | Vendored into `lib/` from the toolchain pin |

Pinned versions are in `cyrius.cyml`; resolved commits in `cyrius.lock`. See
[`docs/development/dependency-watch.md`](docs/development/dependency-watch.md)
for the upgrade gotchas, including the flat-namespace symbol-collision audit.

## Reporting vulnerabilities

Report security issues to the repository maintainer via GitHub Security
Advisories. Do not file public issues for security vulnerabilities.
