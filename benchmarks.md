# Benchmarks

Latest: **2026-03-28** — garjan v1.0.0

All benchmarks run at 44,100 Hz sample rate using criterion with 100 samples.

## Weather & Water

| Benchmark | Time | Real-time factor |
|---|---|---|
| `thunder_1s` | 92 µs | 10,800x |
| `rain_moderate_1s` | 299 µs | 3,300x |
| `wind_1s` | 947 µs | 1,060x |
| `water_stream_1s` | 901 µs | 1,110x |
| `water_waves_1s` | 549 µs | 1,820x |
| `precipitation_hail_1s` | 126 µs | 7,900x |
| `underwater_medium_1s` | 1.07 ms | 935x |
| `surf_moderate_2s` | 2.38 ms | 840x |

## Impact & Contact

| Benchmark | Time | Real-time factor |
|---|---|---|
| `impact_wood_strike` | 359 µs | ~5,000x (variable duration) |
| `impact_metal_strike` | 1.03 ms | ~2,800x |
| `impact_glass_shatter` | 559 µs | ~3,000x |
| `footstep_gravel_walk_1s` | 755 µs | 1,320x |
| `friction_scrape_metal_1s` | 500 µs | 2,000x |
| `creak_door_1s` | 1.10 ms | 910x |
| `rolling_wheel_wood_1s` | 872 µs | 1,150x |
| `foliage_rustle_1s` | 957 µs | 1,040x |

## Aerodynamic

| Benchmark | Time | Real-time factor |
|---|---|---|
| `whoosh_swing` (0.5s) | 469 µs | 1,070x |
| `whistle_pipe_1s` | 1.59 ms | 630x |
| `cloth_flag_1s` | 91 µs | 11,000x |

## Creature & Fluid

| Benchmark | Time | Real-time factor |
|---|---|---|
| `insect_swarm_5_1s` | 2.84 ms | 350x |
| `wingflap_medium_1s` | 242 µs | 4,130x |
| `bubble_boiling_1s` | 158 µs | 6,330x |

## Ambient

| Benchmark | Time | Real-time factor |
|---|---|---|
| `forest_texture_1s` | 1.35 ms | 740x |
| `fire_1s` | 570 µs | 1,750x |

## Engine

| Benchmark | Time | Notes |
|---|---|---|
| `modal_bank_8_modes_512` | 5.3 µs | 512 samples through 8-mode SoA bank |
| `process_block_wind_512` | 10.9 µs | 512-sample streaming block |

## Summary

- Slowest: `insect_swarm_5_1s` at 2.84 ms (350x real-time) — 5 detuned voices with per-sample AM
- Fastest: `cloth_flag_1s` at 91 µs (11,000x real-time) — sparse Poisson events
- All synthesizers are comfortably above real-time
- At 48 kHz with 512-sample blocks, the audio callback budget is ~10.7 ms — even the slowest synth uses <0.3 ms per block
