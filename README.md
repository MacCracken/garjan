# garjan

**garjan** (Sanskrit: गर्जन — roar / thunder) — Environmental and nature sound synthesis for the [Cyrius](https://github.com/MacCracken/cyrius) language (AGNOS ecosystem).

> **Ported from Rust.** The original Rust crate is preserved under [`rust-old/`](rust-old/); the Cyrius port lives in `src/*.cyr`. Build with `cyrius build src/main.cyr build/garjan`, test with `cyrius test`.

Procedural synthesis of weather, impacts, surfaces, fluids, fire, creatures, and aerodynamics. All sounds generated from physical models — modal resonance, stochastic particle impacts, stick-slip friction, turbulent noise shaping. No samples, no assets, pure math.

## Features

### Weather & Water
- **Thunder**: distance-based crack + rumble with atmospheric filtering
- **Rain**: 4 intensities with Poisson-distributed stochastic drops
- **Wind**: pink noise through state variable filter with gust modulation
- **Precipitation**: hail, snow, surface rain with terrain-dependent character
- **Surf**: 3-phase breaking wave cycle (approach, crash, wash)
- **Underwater**: submerged ambience at 3 depths
- **Water**: stream, drip, splash, waves

### Impact & Contact
- **Impact**: 10 materials with modal synthesis (4-12 resonant modes), velocity-sensitive, shatter with debris cascade
- **Footstep**: 8 terrains, 4 movement types, auto-timed or game-driven
- **Friction**: stick-slip scraping, sliding, grinding
- **Creak**: doors, hinges, rope, wood stress
- **Rolling**: ball, wheel, boulder, barrel on surfaces
- **Foliage**: leaf rustle, grass swish, branch snap

### Aerodynamic
- **Whoosh**: sword swing, projectile, vehicle pass-by
- **Whistle**: wind through gaps, pipes, bottles, wires
- **Cloth**: flag, cape, sail, tarp flapping

### Creature & Fluid
- **Insect**: wing buzz, cricket chirp, cicada drone with swarm mode
- **WingFlap**: bird wings at 3 sizes
- **Bubble**: underwater, boiling, viscous, pouring

### Ambient
- **AmbientTexture**: forest, city, ocean, cave, desert, night
- **Fire**: crackle + roar, intensity-scaled

### Infrastructure
- **Modal synthesis engine** with SIMD-friendly SoA layout
- **Voice management** with priority-based polyphony
- **LOD** for CPU scaling of distant sources
- **Science bridge** mapping physical simulation outputs to synthesis parameters
- **Builder pattern** for complex constructors

## Quick Start

Consumers include the bundled `dist/garjan.cyr` and declare garjan's deps
(naad / hisab / goonj / sakshi + stdlib) in their own `cyrius.cyml`, exactly as
`src/main.cyr` does. Constructors return a heap pointer or a negative
`GARJAN_ERR_*` code; `garjan_is_err(x)` checks it.

```cyrius
include "dist/garjan.cyr"

fn main() {
    alloc_init();

    # Fire at half intensity, 44.1 kHz -> 0.5 s of audio
    var fire = fire_new(F64_HALF, f64_from(44100));
    if (garjan_is_err(fire) == 1) { return 1; }
    var samples = fire_synthesize(fire, F64_HALF);   # a vec of f64

    # Moderate rain generator
    var rain = rain_new(RAIN_INTENSITY_MODERATE, f64_from(44100));

    # Modal impact on metal. The table lookups return a pointer or a negative
    # code -- an out-of-range material id is rejected, not silently defaulted.
    var props = material_properties(MATERIAL_METAL);
    if (garjan_is_err(props) == 1) { return 1; }
    var mc = material_mode_config(MATERIAL_METAL);
    if (garjan_is_err(mc) == 1) { return 1; }
    var specs = generate_modes(props, MaterialModeConfig_pattern(mc),
      MaterialModeConfig_mode_count(mc),
      MaterialModeConfig_damping_factor(mc));
    var bank = modal_bank_new(specs, f64_from(44100));
    if (garjan_is_err(bank) == 1) { return 1; }

    return 0;
}
var r = main();
syscall(60, r);
```

Every public type also serializes to / from JSON (`<type>_to_json(p, sb)` /
`<type>_from_json_str(json)`), including the stateful synthesizers.

> **Round-tripping is a full session snapshot.** Since 2.5.0 the live DSP state
> travels too — biquad and SVF memory, noise-generator position, LFO phase,
> modal-bank resonators and exciters — so a synth saved mid-stream resumes
> sample-identically. Documents written before 2.5.0 still load, restoring
> parameters only. See
> [ADR-0008](docs/adr/0008-serde-carries-live-dsp-state.md).

## Performance

Every synthesizer runs comfortably above real-time. 26 benchmarks
(`cyrius bench tests/garjan.bcyr`, `process_block` of 1 s of audio at 44.1 kHz,
x86_64 Linux) — a representative slice, full table in
[`BENCHMARKS.md`](BENCHMARKS.md):

| Synthesizer | Time for 1 s audio | Real-time factor |
|---|---|---|
| Precipitation (hail) | 1.88 ms | ~530x |
| Cloth (Flag) | 2.02 ms | ~495x |
| Rain (Moderate) | 3.86 ms | ~259x |
| Fire | 4.24 ms | ~236x |
| Wind | 10.44 ms | ~96x |
| Texture (Forest) | 16.77 ms | ~60x |
| **Insect (swarm of 8)** | **42.57 ms** | **~23x** |

`insect` is the slowest: its inner loop runs once per swarm voice, so cost
scales linearly with `swarm_count` (max 8). Most of what remains there is
per-voice DSP the algorithm genuinely requires — see `BENCHMARKS.md` for the
component breakdown.

The Cyrius port widened `f32` → `f64` and calls the naad DSP bundle unoptimized,
so per-sample cost is higher than the original Rust crate — still far above
real-time for any realistic voice count.

Five runnable programs live in [`docs/examples/`](docs/examples/) —
weather layering, forest ambience, combat impacts, error handling, and logging.

## What the Rust feature flags became

Cyrius has no cargo-style features; the Rust flags map to always-on wiring:

| Rust flag | In the Cyrius port |
|-----------|--------------------|
| `naad-backend` | Always on — noise/filters/LFOs come from the naad bundle |
| `logging` | Always compiled, wired through **sakshi**; gate at runtime with `sakshi_set_level` |
| `std` | n/a — Cyrius targets AGNOS/bare-metal natively |
| `soorat-compat` | Deferred (kept in `rust-old/`) until soorat is ported |

## Design

- **No samples**: every sound is synthesized from math
- **No hot-path allocations**: `process_block` allocates nothing, pinned by a
  test. One documented exception — `whistle` allocates 32 B/sample because
  naad's `filter_svf_process_sample` returns a heap `SvfOutput` and exposes no
  non-allocating band-pass variant (it has one for low-pass). Blocked upstream;
  avoid `whistle` for long-running streams until it lands.
- **Deterministic**: seeded RNG, bit-identical replay guaranteed. Enforced
  across refactors by [`scripts/audio-hash.cyr`](scripts/audio-hash.cyr) — the
  test suite checks finiteness and energy, *not* exact sample values.
- **Validated at the boundary**: out-of-range enum ids, sample rates outside
  1 Hz–768 kHz, durations over 600 s and buffers over 44.1 M samples are
  rejected rather than silently producing the wrong thing
  ([ADR-0006](docs/adr/0006-out-of-range-enum-ids-are-rejected.md),
  [ADR-0007](docs/adr/0007-bounded-duration-and-sample-rate.md)). **Since 2.5.1
  deserialization enforces the same rules as the constructors** — it no longer
  relies on a downstream component happening to reject bad input
- **Composable**: synthesizers are independent, caller mixes
- **Physically grounded**: modal resonance, Poisson processes, stick-slip models

## AGNOS Ecosystem

garjan is one component of the AGNOS audio pipeline:

| Crate | Role |
|---|---|
| **garjan** | Environmental sound source generation |
| **ghurni** | Mechanical sound synthesis (engines, gears) |
| **prani** / **svara** | Creature vocal synthesis |
| **goonj** | Acoustics (propagation, Doppler, reverb) |
| **dhvani** | Audio engine (mixing, DSP chain, playback) |
| **naad** | Low-level synthesis primitives |

## License

GPL-3.0-only
