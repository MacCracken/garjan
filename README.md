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

    # Modal impact on metal
    var props = material_properties(MATERIAL_METAL);
    var mc = material_mode_config(MATERIAL_METAL);
    var specs = generate_modes(props, MaterialModeConfig_pattern(mc),
                               MaterialModeConfig_mode_count(mc),
                               MaterialModeConfig_damping_factor(mc));
    var bank = modal_bank_new(specs, f64_from(44100));

    return 0;
}
var r = main();
syscall(60, r);
```

Every public type also serializes to / from JSON (`<type>_to_json(p, sb)` /
`<type>_from_json_str(json)`), including the stateful synthesizers.

## Performance

Every synthesizer runs comfortably above real-time. Measurements from
`cyrius bench tests/garjan.bcyr` (`process_block` of 1 s of audio at 44.1 kHz,
x86_64 Linux):

| Synthesizer | Time for 1s audio | Real-time factor |
|---|---|---|
| Cloth (Flag) | 1.03 ms | ~970x |
| Rain (Moderate) | 3.84 ms | ~260x |
| Fire | 4.94 ms | ~200x |
| Thunder | 5.65 ms | ~180x |
| Wind | 11.6 ms | ~85x |

The Cyrius port widened `f32` → `f64` and calls the naad DSP bundle unoptimized,
so per-sample cost is higher than the original Rust crate — still far above
real-time for any realistic voice count.

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
- **No hot-path allocations**: `process_block` never allocates
- **Deterministic**: seeded RNG, bit-identical replay guaranteed
- **`no_std` compatible**: `libm` fallback when std unavailable
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
