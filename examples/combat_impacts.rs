//! Combat impacts: sword swing, metal-on-metal, glass shatter.
//!
//! Demonstrates impact synthesis with modal resonance, whoosh for
//! weapon swings, and the voice pool for managing concurrent sounds.

use garjan::prelude::*;

fn main() {
    let sr = 44100.0;

    // Voice pool: max 8 concurrent impact sounds
    let mut pool = VoicePool::new(8, StealPolicy::LowestPriority);

    // Weapon whoosh (sword swing)
    let mut whoosh = Whoosh::new(WhooshType::Swing, sr).unwrap();
    whoosh.set_speed(0.9);
    let whoosh_audio = whoosh.synthesize(0.4).unwrap();

    // Metal-on-metal impact (sword hitting armor)
    let mut metal_impact = Impact::new_interaction(Material::Metal, Material::Metal, sr).unwrap();
    let metal_audio = metal_impact.synthesize(ImpactType::Crash).unwrap();

    // Glass shatter (potion bottle)
    let mut glass_impact = Impact::new(Material::Glass, sr).unwrap();
    let glass_audio = glass_impact.synthesize(ImpactType::Shatter).unwrap();

    // Velocity-sensitive hit (light tap vs heavy blow)
    let mut wood_impact = Impact::new(Material::Wood, sr).unwrap();
    let light_hit = wood_impact
        .synthesize_velocity(ImpactType::Strike, 0.2)
        .unwrap();
    let heavy_hit = wood_impact
        .synthesize_velocity(ImpactType::Strike, 1.0)
        .unwrap();

    // Allocate voices in the pool
    let _v1 = pool.allocate(5, 1); // whoosh
    let _v2 = pool.allocate(8, 2); // metal crash (high priority)
    let _v3 = pool.allocate(3, 3); // glass shatter
    pool.tick();

    println!("Combat impacts synthesized:");
    println!("  Whoosh:       {} samples", whoosh_audio.len());
    println!("  Metal crash:  {} samples", metal_audio.len());
    println!("  Glass shatter: {} samples", glass_audio.len());
    println!(
        "  Light hit:    {} samples (energy={:.4})",
        light_hit.len(),
        light_hit.iter().map(|s| s * s).sum::<f32>()
    );
    println!(
        "  Heavy hit:    {} samples (energy={:.4})",
        heavy_hit.len(),
        heavy_hit.iter().map(|s| s * s).sum::<f32>()
    );
    println!(
        "  Active voices: {}/{}",
        pool.active_count(),
        pool.max_voices()
    );
}
