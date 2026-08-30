# 0005 — Allocation failure is an error code, not an abort

**Status**: Accepted
**Date**: 2026-08-30

## Context

The 2.0.2 P-1 audit found that a failed allocation was indistinguishable from a
successful one:

- the stdlib `alloc(size)` signals failure by returning `0` (`lib/alloc.cyr` —
  on `size <= 0`, on `size > ALLOC_MAX` (2 GiB), or on `mmap` failure);
- `garjan_is_err(code)` returns 1 only for `code < 0`, and `GARJAN_OK` **is** `0`;
- so `garjan_is_err(alloc(n)) == 0` on failure — the error check passes.

Every constructor wrote through its allocation on the very next line
(`var self = alloc(sizeof(Fire)); Fire_set_intensity(self, inten);`), so an
out-of-memory condition became a store to address 0 rather than a diagnosable
failure. All 85 direct allocation sites were unchecked, plus ~70 more where a
helper result (`rng_new`, `garjan_dcblocker_new`, `voice_slot_new`,
`modal_modespec_new`, `exciter_new`, the `*_config` table builders) was consumed
without a check.

This has no counterpart in the parity oracle. Rust's global allocator calls
`handle_alloc_error`, which **aborts the process**; `rust-old` never had an
allocation error to represent, and its `GarjanError` enum has exactly three
variants, all of which the port already mirrors 1:1.

So the port cannot be both memory-safe under OOM and an exact mirror of the Rust
error surface. That is what makes this a decision rather than a bug fix.

## Decision

Add a fourth error code, `GARJAN_ERR_ALLOCATION` (`-4`), with no Rust
counterpart, and route every allocation in `src/` through a guard:

```cyrius
fn garjan_alloc(size) {
    var p = alloc(size);
    if (p == 0) { return GARJAN_ERR_ALLOCATION; }
    return p;
}
```

Every call site checks the result with the `if (garjan_is_err(p) == 1) { return p; }`
idiom the constructors already used for naad init failures, so an OOM propagates
out through the existing error channel instead of being written through.

In scope: all 85 direct `alloc` sites and every consumer of a pointer-returning
helper. Out of scope: arena lifetime (nothing is ever freed), which is a
separate problem tracked in `docs/development/state.md`.

Rejected alternatives:

- **Abort on OOM, matching Rust exactly.** Closest parity, needs no new error
  code, and cannot be circumvented by a missed call site. Rejected because
  aborting the host process is a policy decision belonging to the application
  embedding garjan, not to a library — Rust only behaves that way because
  `std`'s allocator does, not because the crate chose it. The parity bar exists
  to protect *audio behavior*; OOM policy is not audio behavior.
- **Reuse `GARJAN_ERR_SYNTHESIS_FAILED` (-2).** Keeps the public error surface
  byte-identical to Rust's three variants. Rejected because "synthesis failed"
  is actively misleading in a log when the cause is heap exhaustion, and callers
  may reasonably want to distinguish a retryable resource failure from a bad
  parameter.

## Consequences

**Positive**

- An OOM is now detectable and diagnosable at the point of failure instead of
  being a store to address 0 somewhere downstream.
- Constructor signatures are unchanged — they already returned "pointer or
  negative code", so this rides the existing channel.
- `garjan_err_name` reports `"allocation failed"`, so logs name the real cause.

**Negative**

- The public error surface gains a code with no Rust counterpart. It is additive
  and `garjan_is_err`-compatible, so callers testing `garjan_is_err` are
  unaffected; only callers exhaustively switching on the three known codes must
  learn the fourth.
- Constructors that previously could only fail on parameter validation or naad
  init can now also fail on OOM, so callers have one more reason to see an error.
- `garjan_alloc` deliberately does **not** log. It lives in `src/error.cyr`, the
  L0 module, and the `aero`, `creature`, `rng` and `voice` test entries include
  `error.cyr` *without* `logging.cyr`; since cyrius 6.5.36 a reachable undefined
  function is a hard build error, so a `garjan_log_error` call here would break
  those builds. Callers wanting an OOM logged must log on the error path.

**Neutral**

- The guarantee is only as good as its coverage. A future raw `alloc(`, or a
  helper result used without a check, silently reopens the hole. The invariant
  to preserve is that `src/*.cyr` contains no raw `alloc(` outside
  `garjan_alloc` itself:

  ```sh
  grep -rnE '(^|[^_])alloc\(' src/*.cyr | grep -v garjan_alloc | grep -vE 'alloc_init|alloc_reset|alloc_used'
  ```

  `tests/error.tcyr` pins the wrapper's own behavior.
