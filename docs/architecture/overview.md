# Architecture Overview

## Module Structure

```
garjan/
├── Core synthesis
│   ├── weather.rs      Thunder, Rain, Wind
│   ├── fire.rs         Fire (crackle + roar)
│   ├── water.rs        Stream, Drip, Splash, Waves
│   ├── texture.rs      AmbientTexture (6 environments)
│   ├── impact.rs       Impact (10 materials, modal synthesis)
│   ├── precipitation.rs Hail, Snow, SurfaceRain
│   ├── underwater.rs   Submerged ambience
│   └── surf.rs         Breaking wave cycle
│
├── Contact & surface
│   ├── footstep.rs     Terrain-aware step sequences
│   ├── friction.rs     Stick-slip (scrape, slide, grind)
│   ├── creak.rs        Low-freq stick-slip (door, hinge, rope)
│   ├── rolling.rs      Ball, wheel, boulder, barrel
│   └── foliage.rs      Leaf rustle, grass swish, branch snap
│
├── Aerodynamic
│   ├── whoosh.rs       Object pass-by / swing
│   ├── whistle.rs      Wind through openings
│   └── cloth.rs        Fabric flapping
│
├── Creature & fluid
│   ├── insect.rs       Wing buzz, chirp, cicada + swarm
│   ├── wingflap.rs     Bird wing displacement
│   └── bubble.rs       Minnaert resonance bubbles
│
├── Engine
│   ├── modal.rs        Modal bank (SoA resonator array)
│   ├── voice.rs        VoicePool (priority polyphony)
│   ├── lod.rs          Quality scaling
│   ├── bridge.rs       Science crate parameter conversion
│   └── builder.rs      Ergonomic constructors
│
├── Shared types
│   ├── contact.rs      Terrain, MovementType, FrictionType, etc.
│   ├── aero.rs         WhooshType, WhistleSource, ClothType
│   ├── creature.rs     InsectType, BubbleType
│   ├── material.rs     Material, MaterialProperties, mode configs
│   └── error.rs        GarjanError
│
└── Internal
    ├── dsp.rs          DcBlocker, validate_sample_rate/duration
    ├── math.rs         no_std compat (sin, cos, exp, sqrt, powf)
    └── rng.rs          PCG32 PRNG with Poisson distribution
```

## Synthesizer Pattern

Every synthesizer follows the same pattern:

```rust
pub struct MySynth {
    sample_rate: f32,
    rng: Rng,
    dc_blocker: DcBlocker,
    sample_position: usize,
    // naad types behind cfg gate
    #[cfg(feature = "naad-backend")]
    filter: naad::filter::BiquadFilter,
}

impl MySynth {
    // Constructor: validates sample_rate, returns Result
    pub fn new(..., sample_rate: f32) -> Result<Self> { ... }

    // One-shot: allocates output, calls process_block
    pub fn synthesize(&mut self, duration: f32) -> Result<Vec<f32>> { ... }

    // Streaming: writes into caller's buffer, zero allocation
    pub fn process_block(&mut self, output: &mut [f32]) { ... }
}
```

## Dual Code Paths

All DSP operations have two implementations:

- **`naad-backend` (default)**: Uses naad's filters (BiquadFilter, StateVariableFilter), noise generators (White, Pink, Brown), and LFOs for proper spectral shaping.
- **Fallback**: Manual DSP using the internal Rng and math module. Lower quality but works in `no_std` environments.

Both paths produce finite, deterministic output from the same seeded PRNG.

## Data Flow

```
Constructor params ──> Synthesizer struct (stores state)
                           │
Game loop:                 │
  set_velocity(0.5) ──────┤  (real-time parameter updates)
  set_intensity(0.8) ─────┤
                           │
  process_block(&mut buf) ─┤──> Noise/oscillator generation
                           │──> Filter chain (naad or fallback)
                           │──> Modal bank (if applicable)
                           │──> DC blocker
                           └──> Output samples in buf
```
