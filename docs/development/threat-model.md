# Threat Model

## Trust Boundaries

garjan operates at the **library boundary**. It trusts the calling application
to:

- handle negative error codes rather than dereferencing them as pointers
- mix and spatialize the raw mono output it produces
- not call `alloc_reset()` while any garjan object is still live — that
  invalidates every pointer the allocator has ever handed out

garjan does NOT trust:

- **sample rate** — validated in every constructor: rejects ≤ 0, NaN, infinity,
  and anything outside 1 Hz – 768 kHz
- **duration** — validated in every `synthesize`: rejects ≤ 0, NaN, infinity,
  and anything over 600 s
- **the resulting sample count** — checked separately, because two individually
  legal inputs can still imply an illegal buffer (600 s at 768 kHz is 461 M
  samples). Capped at 44.1 M
- **enum ids** — validated at 21 constructors and 10 table dispatchers. Ported
  Rust enums are plain integers, so an unvalidated id would fall through an
  `if`/`elif` chain's final `else` and silently select the last variant
- **deserialized JSON** — the highest-risk surface; see below. Since 2.5.1 the
  deserialize path validates *everything the constructor validates*: sample
  rate, enum ids, and the size of any buffer it implies. It no longer relies on
  a downstream naad component happening to reject bad input

## Attack surface: deserialization

`*_from_json_str` is the only entry point taking fully attacker-controlled
input. Three defects were found and fixed here in 2.0.2 (see
[the audit](../audit/2026-08-30-audit.md)):

- `*_from_json_str` discarded `*_build_naad`'s error code at 20 of 21 sites, so
  a document with `"sample_rate":0.0` returned a **non-negative pointer** whose
  naad component fields were never assigned — the caller's `garjan_is_err`
  check passed and the first `process_block` dereferenced null. Reproduced as a
  SIGSEGV.
- `voice_pool_from_json_str` sized the pool from the `max_voices` scalar rather
  than the `slots` array, so a ~40-byte document could exhaust memory.
- `insect_from_json_str` restored `swarm_count` without its 1..8 clamp, making
  one `process_block` unbounded.

All three are pinned by regression tests driving genuinely hostile documents.

The 2.5.1 sweep found two more of the same shape, both now fixed:

- **no deserialize path validated `sample_rate`.** They relied on
  `*_build_naad` failing — which `bubble`'s cannot, since it only calls
  `noise_new`. A document with `"sample_rate":0.0` constructed a bubble that
  silently synthesized an empty buffer.
- **no deserialize path re-validated enum ids**, so a hostile document could
  restore an out-of-range id that the constructor rejects, reaching the
  wrong-table behaviour ADR-0006 exists to prevent.

The 2.5.0 `"dsp"` state loaders were tested against twelve hostile documents
(type confusion, over-length arrays, truncation, malformed JSON) with no crash
and no non-finite output.

## Attack surface: resource exhaustion

- **The arena never frees.** Nothing is reclaimed until `alloc_reset()`, which
  invalidates every outstanding pointer. Long-running consumers should treat
  synth construction as an epoch-scoped operation.
- `process_block` allocates nothing, pinned by test — **except `whistle`**,
  which allocates 32 B/sample via naad's `SvfOutput` (~5 GB/hour, arena
  exhausted in ~25 min). Blocked on an upstream naad addition.
- `*_synthesize` grows its output vector by doubling from capacity 16, ~3x
  overhead, all of it retained. Blocked on a stdlib `vec_with_capacity`.

## Failure modes

- **Allocation failure is detectable.** `alloc` returns `0` on exhaustion and
  `0` is `GARJAN_OK`, so a raw `alloc` failure would pass every error check and
  be written through as a null pointer. All allocation goes through
  `garjan_alloc`, which maps it to `GARJAN_ERR_ALLOCATION`
  ([ADR-0005](../adr/0005-allocation-failure-is-an-error-code-not-an-abort.md)).
  **Invariant: `src/*.cyr` contains no raw `alloc(` outside `garjan_alloc`.**
- **No panics or aborts.** Failures are returned as codes; garjan never
  terminates the host process. Rust's allocator aborted on OOM; the port
  deliberately does not.
- garjan emits log events through sakshi but installs no sink; verbosity is a
  runtime setting (`sakshi_set_level`), not a build feature.

## Non-goals

garjan does not sandbox the caller, authenticate input, or defend against a
hostile *host*. It defends against malformed **data** — bad parameters and
hostile JSON — not against a compromised process.
