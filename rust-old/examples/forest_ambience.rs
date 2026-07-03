//! Forest ambience: texture, foliage, insects, bird wings, and a branch snap.
//!
//! Demonstrates layering environmental sounds for a forest scene
//! with both continuous and triggered one-shot events.

use garjan::prelude::*;

fn main() {
    let sr = 44100.0;
    let duration = 5.0;

    // Continuous layers
    let mut texture = AmbientTexture::new(TextureType::Forest, 0.4, sr).unwrap();
    let mut foliage = Foliage::new(FoliageType::LeafRustle, sr).unwrap();
    foliage.set_wind_speed(0.3);
    foliage.set_contact_intensity(0.2);
    let mut insects = Insect::new_swarm(InsectType::CricketChirp, 3, sr).unwrap();
    insects.set_intensity(0.4);

    // One-shot: bird wing flap
    let mut wings = WingFlap::new(BirdSize::Medium, sr).unwrap();
    wings.set_intensity(0.6);

    // Synthesize layers
    let texture_audio = texture.synthesize(duration).unwrap();
    let foliage_audio = foliage.synthesize(duration).unwrap();
    let insect_audio = insects.synthesize(duration).unwrap();
    let wing_audio = wings.synthesize(1.0).unwrap();

    // Mix
    let num_samples = (sr * duration) as usize;
    let mut mix = vec![0.0f32; num_samples];
    for i in 0..num_samples {
        mix[i] = texture_audio[i] * 0.3 + foliage_audio[i] * 0.25 + insect_audio[i] * 0.2;
        // Layer wing flaps starting at 2 seconds
        let wing_offset = (sr * 2.0) as usize;
        if i >= wing_offset && (i - wing_offset) < wing_audio.len() {
            mix[i] += wing_audio[i - wing_offset] * 0.25;
        }
    }

    let peak = mix.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    println!(
        "Forest ambience: {} samples ({:.1}s), peak={:.4}",
        num_samples, duration, peak
    );
}
