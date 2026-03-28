# garjan

**garjan** (Sanskrit: गर्जन — roar / thunder) — Environmental and nature sound synthesis for Rust.

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

```rust
use garjan::prelude::*;

// Thunder 2km away
let mut thunder = Thunder::new(2000.0, 44100.0).unwrap();
let samples = thunder.synthesize(3.0).unwrap();

// Streaming: fill your own buffer
let mut wind = Wind::new(15.0, 0.5, 44100.0).unwrap();
let mut buf = [0.0f32; 512];
wind.process_block(&mut buf);

// Modal impact with velocity
let mut impact = Impact::new(Material::Metal, 44100.0).unwrap();
let samples = impact.synthesize_velocity(ImpactType::Strike, 0.8).unwrap();
```

## Performance

All synthesizers run well above real-time. Typical measurements at 44.1 kHz:

| Synthesizer | Time for 1s audio | Real-time factor |
|---|---|---|
| Cloth (Flag) | 111 µs | 9,000x |
| Thunder | 92 µs | 10,800x |
| Rain (Moderate) | 299 µs | 3,300x |
| Wind | 1.0 ms | 1,000x |
| Impact (Metal) | 1.4 ms | 710x |
| Surf (Moderate, 2s) | ~3 ms | 660x |

## Feature Flags

| Flag | Default | Description |
|------|---------|-------------|
| `std` | Yes | Standard library. Disable for `no_std` + `alloc` |
| `naad-backend` | Yes | Use naad for filters, noise generators, and LFOs |
| `logging` | No | Structured tracing via the `tracing` crate |

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
