# garjan

**garjan** (Sanskrit: गर्जन — roar / thunder) — Environmental and nature sound synthesis for Rust.

Procedural synthesis of weather, impacts, water, fire, and ambient textures. All sounds generated from physical models — stochastic particle impacts, resonant decay, turbulent noise shaping. No samples, no assets, pure math. Built on [hisab](https://crates.io/crates/hisab) for math.

## Features

- **Weather**: Thunder (distance-based crack + rumble), Rain (4 intensities, Poisson-distributed drops), Wind (speed + gustiness modulation)
- **Impact**: 10 materials (Metal, Wood, Stone, Earth, Glass, Fabric, Leaf, Water, Plastic, Ceramic) with material-specific resonance, decay, and brightness. 4 impact types (Tap, Strike, Crash, Shatter)
- **Water**: Stream, Drip, Splash, Waves — stochastic and periodic water synthesis
- **Fire**: Crackle (stochastic impulses) + Roar (broadband combustion noise), intensity-scaled
- **Ambient textures**: 6 environments (Forest, City, Ocean, Cave, Desert, Night) with multi-band spectral shaping
- **Performance**: ~1,000-625,000x real-time, `no_std` compatible, all types `Send + Sync`

## Quick Start

```rust
use garjan::prelude::*;

let mut thunder = Thunder::new(2000.0, 44100.0).unwrap(); // 2km away
let samples = thunder.synthesize(3.0).unwrap();

let mut rain = Rain::new(RainIntensity::Moderate, 44100.0).unwrap();
let samples = rain.synthesize(5.0).unwrap();
```

## Feature Flags

| Flag | Default | Description |
|------|---------|-------------|
| `std` | Yes | Standard library. Disable for `no_std` + `alloc` |
| `naad-backend` | Yes | Use naad for oscillators and filters |
| `logging` | No | Structured logging via tracing-subscriber |

## Consumers

- **kiran** — AGNOS game engine
- **joshua** — Game manager / simulation
- **dhvani** — AGNOS audio engine

## License

GPL-3.0-only
