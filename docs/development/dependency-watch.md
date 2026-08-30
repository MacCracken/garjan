# Dependency Watch

Tracked dependency version constraints, known incompatibilities, and upgrade
paths. Pins live in `cyrius.cyml`; resolved commits in `cyrius.lock`. Current
versions are in [`state.md`](state.md) — this file is the *why* and the
*gotchas*, not the version table.

> Rewritten for the Cyrius port at 2.0.1. The previous revision still described
> the Rust crate graph (serde / thiserror / libm / tracing, "naad pinned to
> 1.x", `#[cfg(feature = "naad-backend")]`) — none of which survived the port.

## The flattened-namespace constraint (read this first)

garjan compiles `dist/garjan.cyr` together with the *monolithic dist bundles*
of naad, hisab, goonj and sakshi. Everything lands in **one flat namespace**.
Two consequences drive every rule below:

1. **Transitive pins must agree.** garjan re-declares hisab, goonj and sakshi
   itself, even though naad pulls them — because the bundle resolves those
   symbols from *this* manifest. If garjan's hisab tag differs from the one
   naad was built against, you get the wrong hisab. On any bump, read each
   dependency's own `cyrius.cyml` and match it:

   ```sh
   for r in naad goonj hisab; do curl -sSL "https://raw.githubusercontent.com/MacCracken/$r/<tag>/cyrius.cyml" | grep -A2 '^\[deps\.'; done
   ```

2. **A duplicate top-level `var` draws NO diagnostic** from cycc or cyrlint —
   it shadows silently. Duplicate `fn`s are caught, `var`s are not. Re-run the
   collision audit on every dependency bump:

   ```sh
   grep -hoE '^(fn|var) [A-Za-z_][A-Za-z0-9_]*' src/*.cyr | awk '{print $2}' | sort -u > /tmp/g.txt
   for d in naad hisab goonj sakshi bayan ganita math; do
     echo "$d: $(comm -12 /tmp/g.txt <(grep -hoE '^(fn|var) [A-Za-z_][A-Za-z0-9_]*' lib/$d.cyr | awk '{print $2}' | sort -u) | wc -l)"
   done
   ```

   Expected: `0` for every row. It was 0 at 2.0.1 across garjan's 399 symbols.

## naad

**Role:** garjan's DSP backend — noise generators (white/pink/brown), biquad and
state-variable filters, LFOs, delay lines. Not optional; there is no fallback
path in the Cyrius port (the Rust `naad-backend` feature gate is gone).

**Upgrade gotcha — symbol prefixing.** naad has been progressively prefixing its
top-level symbols to keep the flat namespace disjoint from goonj/hisab/sakshi.
Two waves so far:

- 2.1.3 — error constants took the `NAAD_ERR_` prefix.
- ≤2.2.2 — filter modes took the `NAAD_FILTER_` prefix. This broke garjan at
  2.0.1 (30 call sites, hard compile error). It was a *pure* rename —
  LOWPASS/HIGHPASS/BANDPASS/NOTCH/ALLPASS/LOWSHELF/HIGHSHELF/PEAK are still
  0-7 in that order — but **verify that** rather than assuming it, because a
  reordered enum would silently retune every filter:

  ```sh
  git show HEAD:lib/naad.cyr | grep -E '^var .*FILTER_[A-Z]+'
  grep -E '^var .*FILTER_[A-Z]+' lib/naad.cyr
  ```

Expect more prefixing. Budget a rename pass on each minor bump.

## hisab / goonj

**Role:** transitively required by naad's bundle — hisab for `HVec3`/`HComplex`/
FFT/spline, goonj for the `acoustics/*` modules naad ships. garjan calls
neither directly; both are declared solely so naad's bundle resolves.

**Note:** hisab's manifest carries a bundle-size figure quoted against cycc's
`input_buf` cap. That cap moved from 1 MB to 16 MB in cyrius 6.5.22, and the
comment was stale by a wide margin before 2.11.2. Treat any dependency-derived
constant in a comment as a measurement that goes stale silently — re-derive on
toolchain bumps rather than trusting it.

## sakshi

**Role:** structured logging. The one transitive dep garjan *also* calls
directly — `src/logging.cyr` wraps `sakshi_warn`/`info`/`debug`/`error`/`fatal`,
wiring the layer the Rust crate gated behind `tracing`. Always compiled;
gate at runtime with `sakshi_set_level`.

**Note:** as in the Rust era, garjan emits events but installs no subscriber —
that stays the consuming application's job.

## Cyrius toolchain + vendored stdlib

The `[package].cyrius` pin is the single source of truth; CI reads it and never
hardcodes a version. `cyrius deps` re-vendors `lib/` from that pin, so a
toolchain bump is a stdlib bump too.

**Upgrade gotcha — the `_str` suffix is reserved.** Cyrius routes a call
`X(a, ...)` to `X_str` whenever `a` is Str-typed at the call site and `X_str`
exists (same dispatch that routes `&IDENT` to `_ptr`). A cstr+len function may
therefore never be named `X_str`. bayan hit this and renamed
`bayan_json_v_parse_str(buf, len)` → `bayan_json_v_parse_buf(buf, len)`;
before the rename, every `bayan_json_v_parse(someStr)` in the ecosystem was
silently rewritten into a 1-arg call to the 2-arg function and returned 0 for
valid JSON. garjan's `voice_pool_from_json_str` was updated at 2.0.1.

**Watch the warnings, not just the errors.** An undefined *variable* is a hard
error, but an undefined *function* is only a warning — it links as a null call
and fails at runtime. After any bump, the build must be warning-clean, not
merely `OK`.

## Upgrade checklist

```sh
cyrius deps                              # re-resolve + re-vendor lib/
cyrius distlib                           # regenerate dist/garjan.cyr
cyrius build src/main.cyr build/garjan   # must be WARNING-clean, not just OK
cyrius test                              # all suites green
cyrius fuzz && cyrius bench tests/garjan.bcyr
cyrius lint src/*.cyr && cyrius vet src/main.cyr && cyrius distlib --check
```

Then: run the collision audit above, diff the arity of every library function
garjan calls (old bundle vs new), and refresh `state.md` + `BENCHMARKS.md`.
