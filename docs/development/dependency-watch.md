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

   Expected: `0` for every row. It was 0 at 2.6.2 across garjan's 426 symbols.

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

**Take the tag hisab declares, not the newest.** At 6.7.18 that is 2.5.10, which is
also the copy the toolchain folds. The two can differ: at 6.7.9 hisab declared 2.5.8,
the fold, and hisab found 2.5.10 warned on every compile that `lib/` shadowed the
toolchain's copy.

## Cyrius toolchain + vendored stdlib

**Status:** pinned to **6.7.18**
([tag `6.7.18`](https://github.com/MacCracken/cyrius/releases/tag/6.7.18),
2026-10-23). naad 2.3.3, hisab 3.4.1 and goonj 2.3.1 pin it exactly, and at 2.6.1
each was its project's newest tag. Check newest over HTTPS, `git ls-remote --tags
https://github.com/MacCracken/<repo>.git`: the `origin` of a local clone is SSH.
The pin tracks published releases only; never pin to
`main` or an unreleased tag, because CI installs the pin through the canonical
upstream installer, which resolves release artifacts
(`cyrius-<version>-<arch>-<os>.tar.gz` + `.sha256`). An unreleased pin makes CI
unresolvable even when it builds locally.

Check whether the pin is behind the newest release:

```sh
curl -s https://api.github.com/repos/MacCracken/cyrius/releases/latest | grep '"tag_name"'
grep '^cyrius = ' cyrius.cyml
```

The `[package].cyrius` pin is the single source of truth; CI reads it and never
hardcodes a version. `cyrius deps` re-vendors `lib/` from that pin, so a
toolchain bump is a stdlib bump too — and stdlib bundles move fast (the 6.3.44
→ 6.5.36 step rewrote ~12,700 lines of `lib/bayan.cyr` alone), so a toolchain
bump always warrants the full upgrade checklist below, not just a rebuild.

**Upgrade gotcha — the `_str` suffix is reserved.** Cyrius routes a call
`X(a, ...)` to `X_str` whenever `a` is Str-typed at the call site and `X_str`
exists (same dispatch that routes `&IDENT` to `_ptr`). A cstr+len function may
therefore never be named `X_str`. bayan hit this and renamed
`bayan_json_v_parse_str(buf, len)` → `bayan_json_v_parse_buf(buf, len)`;
before the rename, every `bayan_json_v_parse(someStr)` in the ecosystem was
silently rewritten into a 1-arg call to the 2-arg function and returned 0 for
valid JSON. garjan's `voice_pool_from_json_str` was updated at 2.0.1.

**Upgrade gotcha — `f64_to` saturates from 6.6.8.** +Inf and anything past 2^63
went from `INT64_MIN` to `INT64_MAX`, and NaN to 0. A floor-only guard (`if (n < 0)`)
in front of a length used to catch the old sentinel by accident and now walks past
it. Passing tests prove nothing here: this reopened `impact_build_naad` at 2.5.3 with
every assertion green. Any new `f64_to` that sizes a buffer or bounds a loop takes
`garjan_validate_sample_count`.

**Upgrade gotcha — `dist/garjan.cyr` compiles under the consumer's toolchain.** A clean
build here proves nothing about an older pin. The deprecation warning on
`bayan_json_v_obj_get` names `bayan_json_v_obj_get_by_cstr`, which first exists at
6.6.13, so following it broke dhvani on 6.6.10 while every local gate stayed green.
That is why `garjan_json_get` exists. Before replacing a stdlib call, check that the
replacement exists in the oldest consumer's `~/.cyrius/versions/<pin>/lib/`.

The same goes for syntax. `src/` must stay off 6.7.x features until dhvani moves:
`const` and `do` stop its 6.6.10 build, but a `!` is worse, because before 6.7.3 it
compiled as a silent no-op.

**Two deliberate couplings, each pinned by a test.** `whistle_svf_band_pass` copies naad's
filter step, and `tests/whistle.tcyr` holds it to naad bit for bit. `garjan_vec_zeros` writes
`lib/vec.cyr`'s header, and `tests/dsp.tcyr` holds that layout. A red test on either after a
bump means upstream moved: follow it, and do not loosen the test.

**Upgrade gotcha — a consumer's floor can move with no garjan change.** Since 2.6.0
garjan's dep set needs cyrius 6.7.2 or later: the bundles use `const`, including as
array sizes. The bundle `dist/garjan.cyr` does not carry them, so a consumer on older
tags is unaffected, and one that adopts this dep set gets a loud compile error below
6.7.2. Measure the floor at each bump, because upstream's figure is for its own bundle.

**Upgrade gotcha — the writer has to stay inside what the parser accepts.** bayan 1.11
refuses duplicate keys, leading zeros, integers outside i64 and malformed numbers.
`garjan_dsp_node` treats a failed parse as a pre-2.5.0 document with no `"dsp"`, so a
document the tree parser refuses still loads, with its live state silently gone. Compare
the writer's bytes across the bump and round-trip real state, as the checklist below says.

**Upgrade gotcha — a whole-bundle dependency can bring new leaves.** bayan 1.11.1 pulled
`slice`, `hashseed` and `lib/unicode/` into `lib/`, and goonj 2.3.1 `simd`. `cyrius deps`
vendors them, so nothing needs declaring, but check `git status lib`.

**Upgrade gotcha — `f64_le` / `f64_ge` left `lib/` at 6.6.13.** They are compiler
builtins now, and `f64_fmod` is gone from `lib/` too. So a diff of `lib/` alone reports
them as removed; compile a call before treating that as a break.

**Upgrade gotcha — `cyrius deps` only re-vendors what something still requires.** A
file that lost its last requirer stays in `lib/` at its old version, and the lock
keeps its old hash. `cyrius deps --verify` and `--locked` both pass over it, and from
6.7.18 `cyrius deps` names such files but still does not refresh them. After a bump,
compare `lib/` against the snapshot in `~/.cyrius/versions/<pin>/lib/`.

**Upgrade gotcha — 6.6.20 fails closed on stale lock rows.** Delete `cyrius.lock`
and re-run `cyrius deps` rather than editing it.

**Watch the warnings, not just the errors.** An undefined *variable* is a hard
error. An undefined *function* is a warning — and, since 6.5.36, additionally a
hard error *if the compiler can reach it* from the entry point:

```
warning: undefined function 'foo'
error: refusing to emit binary with 1 reachable undefined function(s) (pass --allow-undef to downgrade)
```

That reachability gate is why an undefined function can still slip through here.
garjan's entry (`src/main.cyr`) is a small smoke harness that does not call most
of the library, so a symbol broken by an upstream rename stays *unreachable*,
degrades to a bare warning, and the build reports `OK` — while every downstream
consumer that actually calls it gets the failure. That is exactly how
`bayan_json_v_parse_str` presented during the 2.0.1 bump: a lone warning next to
a passing build. **So after any bump the build must be warning-CLEAN, not merely
`OK`,** and never reach for `--allow-undef`.

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

Then build the bundle under the oldest consumer pin, with that consumer's dependency
tags. **Grep the output for `undefined function`.** `OK` is not enough: an entry that
does not reach a broken symbol still reports it, but only as a warning.

Then compare behaviour across the two toolchains, because the suite asserts neither
exact samples nor hostile input:

- **Samples.** `scripts/audio-hash.cyr` tells you *that* something moved. Dump the raw
  samples from both builds to measure *how far*, and look for a length, non-finite or
  sign-of-zero change.
- **Round trips.** Compare each synth's `*_to_json` bytes across the two trees. Then dump
  real state at several sample rates and points in a synth's life and check that the
  tree parser accepts it, `garjan_dsp_node` finds it, and `*_from_json_str` restores it.
- **Hostile input.** `cyrius fuzz` covers JSON documents, and a replay command is in its
  header. It does not cover constructors and setters, so build the old tree beside the
  new one and drive both with +Inf, NaN, ±MAX and past-2^63 values through every f64
  constructor argument and setter. Run each case in its own process under a timeout and
  a memory cap, and compare the outcomes. A case that was refused and now hangs or
  aborts is the regression this catches.
