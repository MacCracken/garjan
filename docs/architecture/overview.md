# Architecture Overview

## Module Structure

```
garjan/
├── Core synthesis
│   ├── weather.cyr     Thunder, Rain, Wind
│   ├── fire.cyr        Fire (crackle + roar)
│   ├── water.cyr       Stream, Drip, Splash, Waves
│   ├── texture.cyr     AmbientTexture (6 environments)
│   ├── impact.cyr      Impact (10 materials, modal synthesis)
│   ├── precipitation.cyr Hail, Snow, SurfaceRain
│   ├── underwater.cyr  Submerged ambience
│   └── surf.cyr        Breaking wave cycle
│
├── Contact & surface
│   ├── footstep.cyr    Terrain-aware step sequences
│   ├── friction.cyr    Stick-slip (scrape, slide, grind)
│   ├── creak.cyr       Low-freq stick-slip (door, hinge, rope)
│   ├── rolling.cyr     Ball, wheel, boulder, barrel
│   └── foliage.cyr     Leaf rustle, grass swish, branch snap
│
├── Aerodynamic
│   ├── whoosh.cyr      Object pass-by / swing
│   ├── whistle.cyr     Wind through openings
│   └── cloth.cyr       Fabric flapping
│
├── Creature & fluid
│   ├── insect.cyr      Wing buzz, chirp, cicada + swarm
│   ├── wingflap.cyr    Bird wing displacement
│   └── bubble.cyr      Minnaert resonance bubbles
│
├── Engine
│   ├── modal.cyr       Modal bank (SoA resonator array)
│   ├── voice.cyr       VoicePool (priority polyphony)
│   ├── lod.cyr         Quality scaling
│   ├── bridge.cyr      Science crate parameter conversion
│   └── builder.cyr     Ergonomic constructors
│
├── Shared types
│   ├── contact.cyr     Terrain, MovementType, FrictionType, etc.
│   ├── aero.cyr        WhooshType, WhistleSource, ClothType
│   ├── creature.cyr    InsectType, BubbleType
│   ├── material.cyr    Material, MaterialProperties, mode configs
│   └── error.cyr       GarjanError
│
└── Internal
    ├── dsp.cyr         DcBlocker, validate_sample_rate/duration
    ├── math.rs        no_std compat (sin, cos, exp, sqrt, powf)
    └── rng.cyr         PCG32 PRNG with Poisson distribution
```

## Synthesizer Pattern

Every synthesizer follows the same shape. Cyrius has no `Result`, no generics
and no module system, so the Rust pattern maps like this:

```cyrius
#derive(accessors)
struct MySynth { sample_rate; rng; dc_blocker; sample_position; filter; }

# Constructor: validates the enum id AND the sample rate, then returns a heap
# pointer or a NEGATIVE GARJAN_ERR_* code. Never a Result.
#must_use
fn my_synth_new(my_type, sample_rate) {
    if (garjan_enum_invalid(my_type, MY_TYPE_LAST) == 1) { return GARJAN_ERR_INVALID_PARAMETER; }
    var e = garjan_validate_sample_rate(sample_rate);
    if (garjan_is_err(e) == 1) { return e; }
    var self = garjan_alloc(sizeof(MySynth));     # never raw alloc
    if (garjan_is_err(self) == 1) { return self; }
    # ...
    var b = my_synth_build_naad(self);
    if (garjan_is_err(b) == 1) { return b; }
    return self;
}

# One-shot: allocates the output vec, then calls process_block.
#must_use
fn my_synth_synthesize(self, duration) {
    var e = garjan_validate_duration(duration);
    if (garjan_is_err(e) == 1) { return e; }
    var num = f64_to(f64_mul(MySynth_sample_rate(self), duration));
    var ga_ncheck = garjan_validate_sample_count(num);
    if (garjan_is_err(ga_ncheck) == 1) { return ga_ncheck; }
    # ...
}

# Streaming: writes into the caller's vec. Must allocate nothing.
fn my_synth_process_block(self, output) { ... }
```

## Things the port does differently from the Rust crate

| Rust | Cyrius port |
|---|---|
| `Result<T, GarjanError>` | pointer or negative `GARJAN_ERR_*`; check `garjan_is_err` |
| `enum Material` | `MATERIAL_*` integer constants, validated at the boundary |
| `#[cfg(feature = "naad-backend")]` dual paths | naad always on; the fallback path was dropped ([ADR-0002](../adr/0002-dual-code-paths.md)) |
| `#[derive(Serialize)]` over live DSP state | scalars only; naad components rebuilt ([note 001](001-deserialize-does-not-restore-dsp-state.md)) |
| `tracing` behind a feature | sakshi, always compiled, gated at runtime |
| f32 throughout | f64 throughout ([note 002](002-where-the-transcendentals-come-from.md)) |
| module system, `lib.rs`, prelude | one flat namespace; `[lib] modules` in `cyrius.cyml` sets the order |

Because the port is f64 and the oracle was f32, **parity with `rust-old` is
structural — same formulas, constants and control flow — never sample-for-sample
equality.** See note 002.
