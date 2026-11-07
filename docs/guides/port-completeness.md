# Port completeness

The verified result lives in [`state.md`](../development/state.md#port-completeness). This is
the **method**, because it must be re-run after any upstream change, and because the first
pass got it wrong: it asserted completeness it had not established.

**1. Module level** — note `rglob`, not `glob`; the first pass used `glob` and
silently skipped `rust-old/src/integration/`.

```sh
find rust-old/src -name '*.rs' | while read f; do
  b=$(basename "$f" .rs); [ -f "src/$b.cyr" ] || echo "NO .cyr: $f"; done
```

**2. Enum discriminants — the check that matters.** A missing `pub fn` fails the
build at its call site; a missing or mis-valued enum *variant* does not, because
ported enums are plain integers. Compare Rust's declaration order against the
Cyrius constant's value, and pin families whose variant names collide
(`Small`/`Medium`/`Large`, `Moderate`/`Heavy`) by exact prefix, not suffix match.

**3. Surface beyond fn/struct/enum.** Confirm these stay empty:

```sh
grep -rnE '^\s*pub\s+(const|static|trait|type)\s' rust-old/src/
grep -rn ' for ' rust-old/src/ | grep -E '^\S+:[0-9]+:\s*impl'   # manual trait impls
grep -rn 'derive(.*Default' rust-old/src/                        # non-zero defaults
```

**4. Account for the dropped fallback**, which the module count hides:

```sh
grep -rc 'cfg(not(feature = "naad-backend"))' rust-old/src/
```

**5. "A symbol exists" is not "it does the same thing."** Verify formulas and
constants — `DcBlocker`'s clamp bounds were checked as bit patterns
(`0x3FECCCCCCCCCCCCD` == 0.9), and the transcendentals measured against libm
rather than assumed.

**6. Check properties, not diffs.** Every defect in the 2.5.1 sweep came from
asking a question across the whole surface — *"does every deserialize entry
point validate its sample rate?"*, *"does every vector-building function have
the size guard?"* — rather than from re-reading code already reviewed. A
scripted rollout is only as good as its pattern: 2.4.0's size guard missed
`impact_synthesize_velocity` because its operands were reversed.
