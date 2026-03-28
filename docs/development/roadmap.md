# garjan Roadmap

> Environmental and nature sound synthesis for AGNOS.

## v1.0.0 (Released)

See [CHANGELOG.md](../../CHANGELOG.md) for the full release history.

---

## Future (v2.0+)

### Deferred from v1.0

- Parameter smoothing (one-pole filter on control parameters)
- Fade-in/fade-out on all continuous synthesizers (click-free start/stop)
- Destruction/fracture: sustained collapse sequences
- Karplus-Strong for metallic ping and plucked-string transients
- Wind turbulence model with terrain spectral variation
- Waveguide wind (extends whistle module with physical tube model)
- Thunder multi-bolt sequences
- Real-time setters for Weather v1 synths (Rain intensity, Wind speed/gustiness)

### Performance

- SIMD explicit intrinsics for ModalBank (currently relies on auto-vectorization)
- Object pool for transient events (drops, crackles, debris)

### Potential new sound categories

- Explosion synthesis (layered: initial burst, debris, rumble tail)
- Machinery integration points (garjan provides ambient hum, ghurni provides mechanical detail)
- Terrain-specific ambient textures (cave drip reverb, desert wind howl)

---

## Design Principles

- **No samples**: every sound is synthesized from math
- **No allocations on hot path**: pre-allocate at construction, stream into caller's buffer
- **Deterministic**: seeded RNG, reproducible output
- **`no_std` compatible**: `libm` fallback, `alloc` only
- **Composable**: synthesizers are independent, caller mixes
- **Physical grounding**: models rooted in acoustics, not arbitrary DSP chains
- **Leverage dependencies**: naad for audio primitives — don't reinvent

## Scope Boundaries (sibling crates)

| Domain | Owner | garjan's role |
|---|---|---|
| Vocal synthesis (bird song, growls, speech) | **prani** (via **svara**) | Not garjan's domain |
| Mechanical sounds (engines, gears, motors) | **ghurni** | Not garjan's domain |
| Acoustics (propagation, Doppler, reverb, RT60) | **goonj** | garjan generates source, goonj propagates |
| Audio engine (mixing, buses, scheduling, RTPC) | **dhvani** | garjan exposes params, dhvani maps them |
| Weather physics (rain rate, wind profiles) | **badal** / **pavan** | garjan consumes their outputs as params |
| Creature behavior (when/why sounds trigger) | **jantu** | jantu decides, garjan synthesizes |

## Non-Goals

These belong in **dhvani** (audio engine) or **kiran** (game engine):

- Spatial audio / 3D positioning / HRTF
- Reverb zones and room acoustics simulation
- Audio bus architecture and mixing
- Compression / limiting on master output
- Speaker layout and format conversion
- Asset management and sound banks
