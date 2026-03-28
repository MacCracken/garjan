# ADR-003: Scope Boundaries with Sibling Crates

## Status

Accepted (v0.5)

## Context

The AGNOS ecosystem has multiple crates that touch audio. Without clear boundaries, garjan could duplicate work done by ghurni (mechanical sounds), svara/prani (vocal synthesis), goonj (acoustics), or dhvani (audio engine).

## Decision

garjan owns **environmental and nature sound source generation** only. Specifically:

| garjan owns | Another crate owns |
|---|---|
| Sound source synthesis (what it sounds like) | Sound propagation (how it travels) — **goonj** |
| Physical mechanism sounds (friction, impact, air) | Vocal/glottal sounds (speech, bird song, growls) — **prani/svara** |
| Environmental sounds (weather, water, foliage) | Mechanical sounds (engines, gears, motors) — **ghurni** |
| Raw parameter API (0-1 knobs) | RTPC mapping (game value → audio parameter) — **dhvani/kiran** |
| Individual synthesizer instances | Mixing, buses, scheduling — **dhvani** |
| Physical unit conversion (bridge module) | Actual physics simulation — **badal/pavan/ushma/vanaspati** |

### Key boundary: Doppler

garjan does NOT implement Doppler shift. The `goonj::propagation::doppler_shift()` function exists for this. garjan's whoosh synthesizer produces the source noise; the caller applies Doppler via goonj.

### Key boundary: creature sounds

Insect buzz and bird wing flaps are **physical mechanisms** (air displacement, stridulation) and belong in garjan. Bird song, growls, and speech are **vocal synthesis** and belong in prani/svara.

## Consequences

- garjan has zero dependencies on sibling science/audio crates (the bridge module takes primitive values).
- Game engines import garjan for sources, goonj for propagation, dhvani for mixing — clean layering.
- New sound categories are evaluated against this boundary before implementation.
