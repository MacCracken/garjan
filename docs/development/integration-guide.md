# Integration Guide

How to use garjan in a game engine or audio application.

## Basic Usage

```rust
use garjan::prelude::*;

// Create a synthesizer (sample rate set once at construction)
let mut rain = Rain::new(RainIntensity::Heavy, 44100.0).unwrap();

// Option A: One-shot (allocates output buffer)
let samples = rain.synthesize(5.0).unwrap();

// Option B: Streaming (zero allocation, real-time safe)
let mut buffer = [0.0f32; 512]; // your audio callback buffer
rain.process_block(&mut buffer);
```

## Real-Time Audio Callback

For game engines, use `process_block` in the audio thread:

```rust
// In your audio callback (called ~86 times/sec at 44100 Hz, 512 samples)
fn audio_callback(output: &mut [f32]) {
    wind.process_block(output);
    // Mix other sources additively:
    let mut rain_buf = [0.0f32; 512];
    rain.process_block(&mut rain_buf);
    for (out, r) in output.iter_mut().zip(rain_buf.iter()) {
        *out += r * 0.5; // mix at 50%
    }
}
```

## Real-Time Parameter Control

Continuous synthesizers expose setters for live parameter changes:

```rust
// Update from game state each frame
friction.set_velocity(player_speed / max_speed);
friction.set_pressure(contact_force.clamp(0.0, 1.0));
wind.set_wind_speed(weather_system.wind_speed_normalized());
```

## Triggered One-Shot Events

Some synthesizers support triggered events:

```rust
// Footstep: auto-timed or manually triggered
footstep.trigger_step(); // call when animation foot hits ground

// Whoosh: trigger on weapon swing
whoosh.trigger();

// Branch snap: trigger on collision
foliage.trigger_snap();
```

## Voice Management

Use `VoicePool` to manage multiple concurrent sounds:

```rust
let mut pool = VoicePool::new(16, StealPolicy::LowestPriority);

// Allocate voices with priority (higher = harder to steal)
let slot = pool.allocate(5, tag_id);  // priority 5

// Each audio frame:
pool.tick(); // advance age counters

// When sound finishes:
pool.release(slot.unwrap());
```

## Using the Science Bridge

If your game uses AGNOS science crates, convert their outputs to garjan parameters:

```rust
use garjan::bridge;

// From badal (weather physics)
let rain_rate_mm_hr = badal::precipitation::rain_rate(cloud_type, cape);
if let Some(intensity) = bridge::rain_intensity_from_rate(rain_rate_mm_hr) {
    let mut rain = Rain::new(intensity, 44100.0).unwrap();
}

// From temperature to thunder distance
let distance = bridge::thunder_distance_from_flash(time_since_flash, temperature_c);
let mut thunder = Thunder::new(distance, 44100.0).unwrap();

// From flame temperature to fire intensity
let intensity = bridge::fire_intensity_from_temperature(flame_temp_k);
let mut fire = Fire::new(intensity, 44100.0).unwrap();
```

## Builder Pattern

For synthesizers with many options:

```rust
use garjan::builder::PrecipitationBuilder;

let mut hail = PrecipitationBuilder::new(44100.0)
    .precip_type(PrecipitationType::Hail)
    .stone_size(StoneSize::Large)
    .surface(Terrain::Metal)
    .build()
    .unwrap();
```

## LOD (Level of Detail)

Use `Quality` to reduce CPU for distant sources:

```rust
let quality = if distance > 100.0 {
    Quality::Minimal
} else if distance > 30.0 {
    Quality::Reduced
} else {
    Quality::Full
};

// Scale mode count for modal synthesis
let mode_count = quality.scale_modes(material.mode_config().mode_count);

// Scale event rate for stochastic sounds
let effective_rate = quality.scale_rate(base_event_rate);
```

## `no_std` Usage

Disable default features for embedded/WASM:

```toml
[dependencies]
garjan = { version = "1", default-features = false }
```

This disables naad (requires std) and uses the manual DSP fallback. All synthesizers work but with reduced audio quality.
