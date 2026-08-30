# 002 — Where the transcendentals come from (and how accurate they are)

garjan's DSP calls six float functions. Only **one** of them is ganita's.

| Call | Uses | Provider |
|---|---|---|
| `f64_exp` | 14 | **cycc intrinsic** |
| `f64_sin` | 12 | **cycc intrinsic** |
| `f64_cos` | 2 | **cycc intrinsic** |
| `f64_sqrt` | 1 | **cycc intrinsic** |
| `f64_abs` | 1 | **cycc intrinsic** |
| `f64_pow` | 2 | **ganita** (`f64_pow` → `ganita_f64_pow`) |

Verify rather than trust — none of the intrinsics is defined in `lib/`:

```sh
grep -l '^fn f64_sin(' lib/*.cyr     # no match
grep -l '^fn f64_pow(' lib/*.cyr     # lib/ganita.cyr
```

Of ganita's **133** functions, garjan calls exactly **one**.

## ganita is still a required dependency — for the deps, not for garjan

Removing it breaks the build, but through the dependency bundles rather than
garjan's own code:

- **hisab** needs 19 ganita symbols — `f64_acos`, `f64_atan2`, `f64_cosh`,
  `f64_sinh`, `f64_pow`, and the whole `mat_*` linear-algebra family
  (`mat_lu`, `mat_qr`, `mat_svd`, `mat_eigen_sym`, …)
- **goonj** needs 5, **naad** needs 2

So the entry in `[deps].stdlib` is load-bearing. It is just not load-bearing
for the reason the manifest comment used to give.

## Measured accuracy

Sampled against correctly-rounded libm — 401 trig points over `[0, 4π]` and 241
`exp` points over `[-12, 0]`, which is the range garjan's envelopes actually
evaluate (`exp(-k·t)`, k ∈ {2,3,5,10}, t ∈ [0,1]):

| Function | Max absolute error | Notes |
|---|---|---|
| `f64_exp` | **0.0** | bit-exact across the whole envelope range |
| `f64_sqrt` | **0.0** | exact |
| `f64_cos` | 1.4e-20 | |
| `f64_sin` | 2.8e-17 | ~1/8 of one f64 epsilon at magnitude 1 |
| `f64_pow` | ≤ 4 ULP | relative ~6e-16 |

For scale: one f64 epsilon is 2.2e-16, and a 24-bit audio LSB is 1.2e-7. The
worst deviation is roughly **four billion times smaller than the smallest
representable audio step**. It is inaudible and irrelevant to synthesis.

Beware raw ULP counts here. `sin(π)` reports ~2.5e11 ULP of "error", which
looks alarming and means nothing: the true value is 1.2246e-16, so a 4e-21
absolute difference spans an enormous number of exponent-adjusted ULPs. Judge
these functions by absolute error near zeros, relative error elsewhere.

The constants are not the problem either — `F64_PI` and `F64_TAU` are
bit-identical to libm's `pi` and `2*pi`.

## The parity consequence

**The port can never be bit-identical to `rust-old`, and it is not supposed to
be.** Rust computed in `f32`, whose epsilon is 1.2e-7; the port computes in
`f64` at 2.2e-16. The port is roughly nine orders of magnitude more precise
than the oracle it is checked against.

So "parity with Rust" means *structural* parity — same formulas, same
constants, same control flow — never sample-for-sample equality. When a parity
check needs numeric comparison, compare against a tolerance derived from f32
epsilon, not from f64. `GARJAN_EPSILON` in `src/error.cyr` is deliberately
`f32::EPSILON` (2^-23) for exactly this reason.

Bit-exactness *is* required in one direction: garjan-against-itself across
refactors. That is what [`scripts/audio-hash.cyr`](../../scripts/audio-hash.cyr)
enforces.
