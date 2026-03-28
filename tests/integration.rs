//! Integration tests for garjan.

use garjan::prelude::*;

const SR: f32 = 44100.0;

// ---------------------------------------------------------------------------
// Weather
// ---------------------------------------------------------------------------

#[test]
fn test_thunder_close() {
    let mut thunder = Thunder::new(200.0, SR).unwrap();
    let samples = thunder.synthesize(2.0).unwrap();
    assert!(!samples.is_empty());
    assert!(samples.iter().all(|s| s.is_finite()));
    assert!(samples.iter().any(|&s| s.abs() > 0.01));
}

#[test]
fn test_thunder_distant() {
    let mut thunder = Thunder::new(5000.0, SR).unwrap();
    let samples = thunder.synthesize(20.0).unwrap();
    assert!(!samples.is_empty());
    assert!(samples.iter().all(|s| s.is_finite()));
}

#[test]
fn test_rain_all_intensities() {
    let intensities = [
        RainIntensity::Light,
        RainIntensity::Moderate,
        RainIntensity::Heavy,
        RainIntensity::Torrential,
    ];
    for intensity in &intensities {
        let mut rain = Rain::new(*intensity, SR).unwrap();
        let samples = rain.synthesize(1.0).unwrap();
        assert!(!samples.is_empty());
        assert!(samples.iter().all(|s| s.is_finite()));
    }
}

#[test]
fn test_rain_heavier_is_louder() {
    let mut light = Rain::new(RainIntensity::Light, SR).unwrap();
    let mut heavy = Rain::new(RainIntensity::Heavy, SR).unwrap();
    let light_samples = light.synthesize(2.0).unwrap();
    let heavy_samples = heavy.synthesize(2.0).unwrap();
    let light_energy: f32 = light_samples.iter().map(|s| s * s).sum();
    let heavy_energy: f32 = heavy_samples.iter().map(|s| s * s).sum();
    assert!(heavy_energy > light_energy);
}

#[test]
fn test_wind() {
    let mut wind = Wind::new(15.0, 0.5, SR).unwrap();
    let samples = wind.synthesize(1.0).unwrap();
    assert!(!samples.is_empty());
    assert!(samples.iter().all(|s| s.is_finite()));
}

// ---------------------------------------------------------------------------
// Impact
// ---------------------------------------------------------------------------

#[test]
fn test_impact_all_materials() {
    let materials = [
        Material::Metal,
        Material::Wood,
        Material::Stone,
        Material::Earth,
        Material::Glass,
        Material::Fabric,
        Material::Leaf,
        Material::Water,
        Material::Plastic,
        Material::Ceramic,
    ];
    for mat in &materials {
        let mut impact = Impact::new(*mat, SR).unwrap();
        let samples = impact.synthesize(ImpactType::Strike).unwrap();
        assert!(!samples.is_empty(), "failed for {:?}", mat);
        assert!(samples.iter().all(|s| s.is_finite()));
    }
}

#[test]
fn test_impact_types() {
    let types = [
        ImpactType::Tap,
        ImpactType::Strike,
        ImpactType::Crash,
        ImpactType::Shatter,
    ];
    for t in &types {
        let mut impact = Impact::new(Material::Wood, SR).unwrap();
        let samples = impact.synthesize(*t).unwrap();
        assert!(!samples.is_empty());
        assert!(samples.iter().all(|s| s.is_finite()));
    }
}

// ---------------------------------------------------------------------------
// Fire
// ---------------------------------------------------------------------------

#[test]
fn test_fire() {
    let mut fire = Fire::new(0.7, SR).unwrap();
    let samples = fire.synthesize(1.0).unwrap();
    assert!(!samples.is_empty());
    assert!(samples.iter().all(|s| s.is_finite()));
}

// ---------------------------------------------------------------------------
// Water
// ---------------------------------------------------------------------------

#[test]
fn test_water_all_types() {
    let types = [
        WaterType::Stream,
        WaterType::Drip,
        WaterType::Splash,
        WaterType::Waves,
    ];
    for wt in &types {
        let mut water = Water::new(*wt, 0.5, SR).unwrap();
        let samples = water.synthesize(0.5).unwrap();
        assert!(!samples.is_empty(), "failed for {:?}", wt);
        assert!(samples.iter().all(|s| s.is_finite()));
    }
}

// ---------------------------------------------------------------------------
// Ambient textures
// ---------------------------------------------------------------------------

#[test]
fn test_ambient_textures() {
    let types = [
        TextureType::Forest,
        TextureType::City,
        TextureType::Ocean,
        TextureType::Cave,
        TextureType::Desert,
        TextureType::Night,
    ];
    for tt in &types {
        let mut tex = AmbientTexture::new(*tt, 0.5, SR).unwrap();
        let samples = tex.synthesize(1.0).unwrap();
        assert!(!samples.is_empty(), "failed for {:?}", tt);
        assert!(samples.iter().all(|s| s.is_finite()));
    }
}

// ---------------------------------------------------------------------------
// Serde roundtrips
// ---------------------------------------------------------------------------

#[test]
fn test_serde_roundtrip_material() {
    let json = serde_json::to_string(&Material::Glass).unwrap();
    let m2: Material = serde_json::from_str(&json).unwrap();
    assert_eq!(m2, Material::Glass);
}

#[test]
fn test_serde_roundtrip_rain_intensity() {
    let json = serde_json::to_string(&RainIntensity::Torrential).unwrap();
    let r2: RainIntensity = serde_json::from_str(&json).unwrap();
    assert_eq!(r2, RainIntensity::Torrential);
}

#[test]
fn test_serde_roundtrip_impact_type() {
    let json = serde_json::to_string(&ImpactType::Shatter).unwrap();
    let i2: ImpactType = serde_json::from_str(&json).unwrap();
    assert_eq!(i2, ImpactType::Shatter);
}

#[test]
fn test_serde_roundtrip_water_type() {
    let json = serde_json::to_string(&WaterType::Waves).unwrap();
    let w2: WaterType = serde_json::from_str(&json).unwrap();
    assert_eq!(w2, WaterType::Waves);
}

#[test]
fn test_serde_roundtrip_texture_type() {
    let json = serde_json::to_string(&TextureType::Cave).unwrap();
    let t2: TextureType = serde_json::from_str(&json).unwrap();
    assert_eq!(t2, TextureType::Cave);
}

#[test]
fn test_serde_roundtrip_material_properties() {
    let props = Material::Metal.properties();
    let json = serde_json::to_string(&props).unwrap();
    let p2: garjan::material::MaterialProperties = serde_json::from_str(&json).unwrap();
    assert_eq!(props.resonance, p2.resonance);
    assert_eq!(props.decay, p2.decay);
}

#[test]
fn test_serde_roundtrip_error() {
    let err = GarjanError::SynthesisFailed("test".into());
    let json = serde_json::to_string(&err).unwrap();
    let e2: GarjanError = serde_json::from_str(&json).unwrap();
    assert_eq!(err.to_string(), e2.to_string());
}

#[test]
fn test_serde_roundtrip_thunder() {
    let thunder = Thunder::new(500.0, SR).unwrap();
    let json = serde_json::to_string(&thunder).unwrap();
    let t2: Thunder = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&t2).unwrap();
    assert_eq!(json, json2);
}

#[test]
fn test_serde_roundtrip_rain() {
    let rain = Rain::new(RainIntensity::Heavy, SR).unwrap();
    let json = serde_json::to_string(&rain).unwrap();
    let r2: Rain = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&r2).unwrap();
    assert_eq!(json, json2);
}

#[test]
fn test_serde_roundtrip_wind() {
    let wind = Wind::new(10.0, 0.3, SR).unwrap();
    let json = serde_json::to_string(&wind).unwrap();
    let w2: Wind = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&w2).unwrap();
    assert_eq!(json, json2);
}

#[test]
fn test_serde_roundtrip_fire() {
    let fire = Fire::new(0.5, SR).unwrap();
    let json = serde_json::to_string(&fire).unwrap();
    let f2: Fire = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&f2).unwrap();
    assert_eq!(json, json2);
}

#[test]
fn test_serde_roundtrip_water() {
    let water = Water::new(WaterType::Waves, 0.7, SR).unwrap();
    let json = serde_json::to_string(&water).unwrap();
    let w2: Water = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&w2).unwrap();
    assert_eq!(json, json2);
}

#[test]
fn test_serde_roundtrip_impact() {
    let impact = Impact::new(Material::Glass, SR).unwrap();
    let json = serde_json::to_string(&impact).unwrap();
    let i2: Impact = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&i2).unwrap();
    assert_eq!(json, json2);
}

#[test]
fn test_serde_roundtrip_ambient_texture() {
    let tex = AmbientTexture::new(TextureType::Ocean, 0.6, SR).unwrap();
    let json = serde_json::to_string(&tex).unwrap();
    let t2: AmbientTexture = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&t2).unwrap();
    assert_eq!(json, json2);
}

// ---------------------------------------------------------------------------
// process_block streaming
// ---------------------------------------------------------------------------

#[test]
fn test_process_block_wind() {
    let mut wind = Wind::new(15.0, 0.5, SR).unwrap();
    let mut buf = vec![0.0f32; 512];
    wind.process_block(&mut buf);
    assert!(buf.iter().all(|s| s.is_finite()));
    assert!(buf.iter().any(|&s| s.abs() > 0.001));
}

#[test]
fn test_process_block_rain() {
    let mut rain = Rain::new(RainIntensity::Heavy, SR).unwrap();
    let mut buf = vec![0.0f32; 4410]; // 100ms
    rain.process_block(&mut buf);
    assert!(buf.iter().all(|s| s.is_finite()));
}

#[test]
fn test_process_block_fire() {
    let mut fire = Fire::new(0.8, SR).unwrap();
    let mut buf = vec![0.0f32; 4410];
    fire.process_block(&mut buf);
    assert!(buf.iter().all(|s| s.is_finite()));
    assert!(buf.iter().any(|&s| s.abs() > 0.001));
}

#[test]
fn test_process_block_texture() {
    let mut tex = AmbientTexture::new(TextureType::Forest, 0.5, SR).unwrap();
    let mut buf = vec![0.0f32; 512];
    tex.process_block(&mut buf);
    assert!(buf.iter().all(|s| s.is_finite()));
    assert!(buf.iter().any(|&s| s.abs() > 0.001));
}

#[test]
fn test_process_block_empty() {
    let mut wind = Wind::new(10.0, 0.3, SR).unwrap();
    let mut buf: [f32; 0] = [];
    wind.process_block(&mut buf); // should not panic
}

// ---------------------------------------------------------------------------
// Parameter validation
// ---------------------------------------------------------------------------

#[test]
fn test_invalid_sample_rate_zero() {
    assert!(Thunder::new(500.0, 0.0).is_err());
    assert!(Rain::new(RainIntensity::Moderate, 0.0).is_err());
    assert!(Wind::new(10.0, 0.5, 0.0).is_err());
    assert!(Fire::new(0.5, 0.0).is_err());
    assert!(Water::new(WaterType::Stream, 0.5, 0.0).is_err());
    assert!(Impact::new(Material::Wood, 0.0).is_err());
    assert!(AmbientTexture::new(TextureType::Forest, 0.5, 0.0).is_err());
}

#[test]
fn test_invalid_sample_rate_negative() {
    assert!(Thunder::new(500.0, -44100.0).is_err());
}

#[test]
fn test_invalid_sample_rate_nan() {
    assert!(Thunder::new(500.0, f32::NAN).is_err());
}

#[test]
fn test_invalid_sample_rate_inf() {
    assert!(Thunder::new(500.0, f32::INFINITY).is_err());
}

#[test]
fn test_invalid_duration() {
    let mut wind = Wind::new(10.0, 0.5, SR).unwrap();
    assert!(wind.synthesize(-1.0).is_err());
    assert!(wind.synthesize(0.0).is_err());
    assert!(wind.synthesize(f32::NAN).is_err());
    assert!(wind.synthesize(f32::INFINITY).is_err());
}

// ---------------------------------------------------------------------------
// Deterministic replay
// ---------------------------------------------------------------------------

#[test]
fn test_deterministic_replay_all_synths() {
    // Every synth with same constructor params must produce identical output
    macro_rules! check_deterministic {
        ($name:expr, $a:expr, $b:expr, $dur:expr) => {{
            let sa = $a.synthesize($dur).unwrap();
            let sb = $b.synthesize($dur).unwrap();
            assert_eq!(sa.len(), sb.len(), "{} length mismatch", $name);
            assert!(
                sa.iter().zip(sb.iter()).all(|(a, b)| (a - b).abs() < 1e-10),
                "{} not deterministic",
                $name
            );
        }};
    }

    check_deterministic!(
        "Rain",
        Rain::new(RainIntensity::Moderate, SR).unwrap(),
        Rain::new(RainIntensity::Moderate, SR).unwrap(),
        0.5
    );
    check_deterministic!(
        "Thunder",
        Thunder::new(500.0, SR).unwrap(),
        Thunder::new(500.0, SR).unwrap(),
        1.0
    );
    check_deterministic!(
        "Wind",
        Wind::new(10.0, 0.5, SR).unwrap(),
        Wind::new(10.0, 0.5, SR).unwrap(),
        0.5
    );
    check_deterministic!(
        "Fire",
        Fire::new(0.5, SR).unwrap(),
        Fire::new(0.5, SR).unwrap(),
        0.5
    );
    check_deterministic!(
        "Water",
        Water::new(WaterType::Stream, 0.5, SR).unwrap(),
        Water::new(WaterType::Stream, 0.5, SR).unwrap(),
        0.5
    );
    check_deterministic!(
        "AmbientTexture",
        AmbientTexture::new(TextureType::Forest, 0.5, SR).unwrap(),
        AmbientTexture::new(TextureType::Forest, 0.5, SR).unwrap(),
        0.5
    );
    check_deterministic!(
        "Impact",
        Impact::new(Material::Metal, SR).unwrap(),
        Impact::new(Material::Metal, SR).unwrap(),
        ImpactType::Strike
    );
    {
        let mut a = Friction::new(FrictionType::Scrape, Material::Metal, SR).unwrap();
        let mut b = Friction::new(FrictionType::Scrape, Material::Metal, SR).unwrap();
        a.set_velocity(0.5);
        b.set_velocity(0.5);
        check_deterministic!("Friction", a, b, 0.5);
    }
    {
        let mut a = Bubble::new(BubbleType::Boiling, SR).unwrap();
        let mut b = Bubble::new(BubbleType::Boiling, SR).unwrap();
        a.set_intensity(0.8);
        b.set_intensity(0.8);
        check_deterministic!("Bubble", a, b, 0.5);
    }
}

// ---------------------------------------------------------------------------
// Modal synthesis
// ---------------------------------------------------------------------------

#[test]
fn test_modal_bank_impulse_response() {
    let specs = vec![
        ModeSpec {
            frequency: 440.0,
            amplitude: 1.0,
            decay: 0.5,
        },
        ModeSpec {
            frequency: 880.0,
            amplitude: 0.5,
            decay: 0.3,
        },
    ];
    let mut bank = ModalBank::new(&specs, SR).unwrap();
    assert_eq!(bank.mode_count(), 2);

    // Feed impulse
    let first = bank.process_sample(1.0);
    assert!(first.is_finite());
    assert!(first.abs() > 0.0);

    // Feed zeros — should decay
    let mut last = first;
    for _ in 0..1000 {
        last = bank.process_sample(0.0);
        assert!(last.is_finite());
    }
    // After many samples, should be decaying
    assert!(last.abs() < first.abs());
}

#[test]
fn test_modal_bank_all_patterns() {
    let props = Material::Metal.properties();
    let patterns = [
        (ModePattern::Harmonic, 6),
        (ModePattern::Beam, 8),
        (ModePattern::Plate, 10),
        (ModePattern::StiffString, 6),
        (ModePattern::Damped, 4),
    ];
    for (pattern, count) in &patterns {
        let specs = garjan::modal::generate_modes(&props, *pattern, *count, 1.0);
        assert!(!specs.is_empty(), "no modes for {:?}", pattern);
        let mut bank = ModalBank::new(&specs, SR).unwrap();
        let out = bank.process_sample(1.0);
        assert!(out.is_finite(), "NaN for {:?}", pattern);
    }
}

#[test]
fn test_modal_bank_nyquist_guard() {
    let specs = vec![
        ModeSpec {
            frequency: 100.0,
            amplitude: 1.0,
            decay: 0.5,
        },
        ModeSpec {
            frequency: 30000.0, // above Nyquist at 44100
            amplitude: 1.0,
            decay: 0.5,
        },
    ];
    let bank = ModalBank::new(&specs, SR).unwrap();
    assert_eq!(bank.mode_count(), 1); // only the 100 Hz mode
}

#[test]
fn test_modal_bank_reset() {
    let specs = vec![ModeSpec {
        frequency: 440.0,
        amplitude: 1.0,
        decay: 1.0,
    }];
    let mut bank = ModalBank::new(&specs, SR).unwrap();
    bank.process_sample(1.0);
    assert!(bank.process_sample(0.0).abs() > 0.0);
    bank.reset();
    assert_eq!(bank.process_sample(0.0), 0.0);
}

#[test]
fn test_exciter_impulse() {
    let mut exc = Exciter::new(ExcitationType::Impulse, 1.0);
    exc.trigger();
    assert!(exc.is_active());
    let s0 = exc.next_sample();
    assert_eq!(s0, 1.0);
    let s1 = exc.next_sample();
    assert_eq!(s1, 0.0);
    assert!(!exc.is_active());
}

#[test]
fn test_exciter_noise_burst() {
    let mut exc = Exciter::new(
        ExcitationType::NoiseBurst {
            duration_samples: 100,
        },
        0.5,
    );
    exc.trigger();
    let mut nonzero = 0;
    for _ in 0..100 {
        if exc.next_sample().abs() > 0.0 {
            nonzero += 1;
        }
    }
    assert!(nonzero > 50); // most samples should be nonzero
    assert_eq!(exc.next_sample(), 0.0); // spent
}

#[test]
fn test_exciter_half_sine() {
    let mut exc = Exciter::new(
        ExcitationType::HalfSine {
            duration_samples: 50,
        },
        1.0,
    );
    exc.trigger();
    let mut peak = 0.0f32;
    for _ in 0..50 {
        peak = peak.max(exc.next_sample());
    }
    assert!(peak > 0.5); // should reach near 1.0 at midpoint
    assert_eq!(exc.next_sample(), 0.0); // spent
}

// ---------------------------------------------------------------------------
// Impact with modal bank
// ---------------------------------------------------------------------------

#[test]
fn test_impact_interaction() {
    let mut impact = Impact::new_interaction(Material::Metal, Material::Glass, SR).unwrap();
    let samples = impact.synthesize(ImpactType::Strike).unwrap();
    assert!(!samples.is_empty());
    assert!(samples.iter().all(|s| s.is_finite()));
}

#[test]
fn test_impact_velocity() {
    let mut soft = Impact::new(Material::Metal, SR).unwrap();
    let mut hard = Impact::new(Material::Metal, SR).unwrap();
    let soft_samples = soft.synthesize_velocity(ImpactType::Strike, 0.1).unwrap();
    let hard_samples = hard.synthesize_velocity(ImpactType::Strike, 1.0).unwrap();
    let soft_energy: f32 = soft_samples.iter().map(|s| s * s).sum();
    let hard_energy: f32 = hard_samples.iter().map(|s| s * s).sum();
    assert!(hard_energy > soft_energy);
}

#[test]
fn test_impact_shatter_has_debris() {
    let mut impact = Impact::new(Material::Glass, SR).unwrap();
    let shatter = impact.synthesize(ImpactType::Shatter).unwrap();
    let mut impact2 = Impact::new(Material::Glass, SR).unwrap();
    let tap = impact2.synthesize(ImpactType::Tap).unwrap();
    let shatter_energy: f32 = shatter.iter().map(|s| s * s).sum();
    let tap_energy: f32 = tap.iter().map(|s| s * s).sum();
    assert!(shatter_energy > tap_energy);
}

#[test]
fn test_impact_deterministic() {
    let mut a = Impact::new(Material::Wood, SR).unwrap();
    let mut b = Impact::new(Material::Wood, SR).unwrap();
    let sa = a.synthesize(ImpactType::Strike).unwrap();
    let sb = b.synthesize(ImpactType::Strike).unwrap();
    assert_eq!(sa.len(), sb.len());
    assert!(sa.iter().zip(sb.iter()).all(|(a, b)| (a - b).abs() < 1e-10));
}

#[test]
fn test_serde_roundtrip_modal_bank() {
    let specs = vec![ModeSpec {
        frequency: 440.0,
        amplitude: 1.0,
        decay: 0.5,
    }];
    let bank = ModalBank::new(&specs, SR).unwrap();
    let json = serde_json::to_string(&bank).unwrap();
    let b2: ModalBank = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&b2).unwrap();
    assert_eq!(json, json2);
}

#[test]
fn test_serde_roundtrip_mode_spec() {
    let spec = ModeSpec {
        frequency: 440.0,
        amplitude: 0.8,
        decay: 0.3,
    };
    let json = serde_json::to_string(&spec).unwrap();
    let s2: ModeSpec = serde_json::from_str(&json).unwrap();
    assert_eq!(spec.frequency, s2.frequency);
}

#[test]
fn test_serde_roundtrip_mode_pattern() {
    let json = serde_json::to_string(&ModePattern::Plate).unwrap();
    let p2: ModePattern = serde_json::from_str(&json).unwrap();
    assert_eq!(p2, ModePattern::Plate);
}

#[test]
fn test_serde_roundtrip_exciter() {
    let exc = Exciter::new(
        ExcitationType::NoiseBurst {
            duration_samples: 100,
        },
        0.5,
    );
    let json = serde_json::to_string(&exc).unwrap();
    let e2: Exciter = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&e2).unwrap();
    assert_eq!(json, json2);
}

#[test]
fn test_serde_roundtrip_material_mode_config() {
    let cfg = Material::Metal.mode_config();
    let json = serde_json::to_string(&cfg).unwrap();
    let c2: garjan::material::MaterialModeConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(cfg, c2);
}

#[test]
fn test_serde_roundtrip_excitation_type() {
    let et = ExcitationType::NoiseBurst {
        duration_samples: 100,
    };
    let json = serde_json::to_string(&et).unwrap();
    let e2: ExcitationType = serde_json::from_str(&json).unwrap();
    assert_eq!(et, e2);
}

// ---------------------------------------------------------------------------
// Footstep
// ---------------------------------------------------------------------------

#[test]
fn test_footstep_all_terrains() {
    let terrains = [
        Terrain::Gravel,
        Terrain::Sand,
        Terrain::Mud,
        Terrain::Snow,
        Terrain::Wood,
        Terrain::Metal,
        Terrain::Tile,
        Terrain::Wet,
    ];
    for terrain in &terrains {
        let mut fs = Footstep::new(*terrain, MovementType::Walk, SR).unwrap();
        let samples = fs.synthesize(1.0).unwrap();
        assert!(!samples.is_empty(), "failed for {:?}", terrain);
        assert!(samples.iter().all(|s| s.is_finite()));
    }
}

#[test]
fn test_footstep_all_movements() {
    for mov in &[
        MovementType::Walk,
        MovementType::Run,
        MovementType::Sneak,
        MovementType::JumpLand,
    ] {
        let mut fs = Footstep::new(Terrain::Gravel, *mov, SR).unwrap();
        let samples = fs.synthesize(0.5).unwrap();
        assert!(samples.iter().all(|s| s.is_finite()));
    }
}

#[test]
fn test_footstep_trigger_step() {
    let mut fs = Footstep::new(Terrain::Wood, MovementType::Walk, SR).unwrap();
    fs.trigger_step();
    let mut buf = vec![0.0f32; 512];
    fs.process_block(&mut buf);
    assert!(buf.iter().any(|&s| s.abs() > 0.001));
}

// ---------------------------------------------------------------------------
// Friction
// ---------------------------------------------------------------------------

#[test]
fn test_friction_all_types() {
    for ft in &[
        FrictionType::Scrape,
        FrictionType::Slide,
        FrictionType::Grind,
    ] {
        let mut f = Friction::new(*ft, Material::Metal, SR).unwrap();
        f.set_velocity(0.5);
        f.set_pressure(0.5);
        let samples = f.synthesize(0.5).unwrap();
        assert!(samples.iter().all(|s| s.is_finite()));
        assert!(samples.iter().any(|&s| s.abs() > 0.001));
    }
}

#[test]
fn test_friction_zero_velocity_is_silent() {
    let mut f = Friction::new(FrictionType::Scrape, Material::Wood, SR).unwrap();
    f.set_velocity(0.0);
    let samples = f.synthesize(0.1).unwrap();
    assert!(samples.iter().all(|&s| s.abs() < 0.001));
}

// ---------------------------------------------------------------------------
// Creak
// ---------------------------------------------------------------------------

#[test]
fn test_creak_all_sources() {
    for src in &[
        CreakSource::Door,
        CreakSource::Hinge,
        CreakSource::Rope,
        CreakSource::WoodStress,
    ] {
        let mut c = Creak::new(*src, SR).unwrap();
        c.set_tension(0.5);
        c.set_speed(0.5);
        let samples = c.synthesize(0.5).unwrap();
        assert!(samples.iter().all(|s| s.is_finite()));
        assert!(samples.iter().any(|&s| s.abs() > 0.001));
    }
}

#[test]
fn test_creak_zero_speed_is_silent() {
    let mut c = Creak::new(CreakSource::Door, SR).unwrap();
    c.set_speed(0.0);
    let samples = c.synthesize(0.1).unwrap();
    assert!(samples.iter().all(|&s| s.abs() < 0.001));
}

// ---------------------------------------------------------------------------
// Rolling
// ---------------------------------------------------------------------------

#[test]
fn test_rolling_all_bodies() {
    for body in &[
        RollingBody::Ball,
        RollingBody::Wheel,
        RollingBody::Boulder,
        RollingBody::Barrel,
    ] {
        let mut r = Rolling::new(*body, Material::Wood, SR).unwrap();
        r.set_velocity(0.5);
        let samples = r.synthesize(0.5).unwrap();
        assert!(samples.iter().all(|s| s.is_finite()));
        assert!(samples.iter().any(|&s| s.abs() > 0.001));
    }
}

#[test]
fn test_rolling_zero_velocity_is_silent() {
    let mut r = Rolling::new(RollingBody::Wheel, Material::Stone, SR).unwrap();
    r.set_velocity(0.0);
    let samples = r.synthesize(0.1).unwrap();
    assert!(samples.iter().all(|&s| s.abs() < 0.001));
}

// ---------------------------------------------------------------------------
// Foliage
// ---------------------------------------------------------------------------

#[test]
fn test_foliage_all_types() {
    for ft in &[
        FoliageType::LeafRustle,
        FoliageType::GrassSwish,
        FoliageType::BranchSnap,
    ] {
        let mut f = Foliage::new(*ft, SR).unwrap();
        if *ft == FoliageType::BranchSnap {
            f.trigger_snap();
        } else {
            f.set_wind_speed(0.5);
        }
        let samples = f.synthesize(0.5).unwrap();
        assert!(samples.iter().all(|s| s.is_finite()), "NaN for {:?}", ft);
    }
}

#[test]
fn test_foliage_branch_snap_trigger() {
    let mut f = Foliage::new(FoliageType::BranchSnap, SR).unwrap();
    f.trigger_snap();
    let mut buf = vec![0.0f32; 512];
    f.process_block(&mut buf);
    assert!(buf.iter().any(|&s| s.abs() > 0.001));
}

// ---------------------------------------------------------------------------
// Contact enum serde roundtrips
// ---------------------------------------------------------------------------

#[test]
fn test_serde_roundtrip_terrain() {
    let json = serde_json::to_string(&Terrain::Gravel).unwrap();
    let t2: Terrain = serde_json::from_str(&json).unwrap();
    assert_eq!(t2, Terrain::Gravel);
}

#[test]
fn test_serde_roundtrip_movement_type() {
    let json = serde_json::to_string(&MovementType::Run).unwrap();
    let m2: MovementType = serde_json::from_str(&json).unwrap();
    assert_eq!(m2, MovementType::Run);
}

#[test]
fn test_serde_roundtrip_friction_type() {
    let json = serde_json::to_string(&FrictionType::Scrape).unwrap();
    let f2: FrictionType = serde_json::from_str(&json).unwrap();
    assert_eq!(f2, FrictionType::Scrape);
}

#[test]
fn test_serde_roundtrip_rolling_body() {
    let json = serde_json::to_string(&RollingBody::Barrel).unwrap();
    let r2: RollingBody = serde_json::from_str(&json).unwrap();
    assert_eq!(r2, RollingBody::Barrel);
}

#[test]
fn test_serde_roundtrip_foliage_type() {
    let json = serde_json::to_string(&FoliageType::GrassSwish).unwrap();
    let f2: FoliageType = serde_json::from_str(&json).unwrap();
    assert_eq!(f2, FoliageType::GrassSwish);
}

#[test]
fn test_serde_roundtrip_footstep() {
    let fs = Footstep::new(Terrain::Wood, MovementType::Walk, SR).unwrap();
    let json = serde_json::to_string(&fs).unwrap();
    let f2: Footstep = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&f2).unwrap();
    assert_eq!(json, json2);
}

#[test]
fn test_serde_roundtrip_friction() {
    let f = Friction::new(FrictionType::Scrape, Material::Metal, SR).unwrap();
    let json = serde_json::to_string(&f).unwrap();
    let f2: Friction = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&f2).unwrap();
    assert_eq!(json, json2);
}

#[test]
fn test_serde_roundtrip_creak() {
    let c = Creak::new(CreakSource::Door, SR).unwrap();
    let json = serde_json::to_string(&c).unwrap();
    let c2: Creak = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&c2).unwrap();
    assert_eq!(json, json2);
}

#[test]
fn test_serde_roundtrip_rolling() {
    let r = Rolling::new(RollingBody::Barrel, Material::Wood, SR).unwrap();
    let json = serde_json::to_string(&r).unwrap();
    let r2: Rolling = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&r2).unwrap();
    assert_eq!(json, json2);
}

#[test]
fn test_serde_roundtrip_foliage() {
    let f = Foliage::new(FoliageType::LeafRustle, SR).unwrap();
    let json = serde_json::to_string(&f).unwrap();
    let f2: Foliage = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&f2).unwrap();
    assert_eq!(json, json2);
}

#[test]
fn test_serde_roundtrip_creak_source() {
    let json = serde_json::to_string(&CreakSource::Hinge).unwrap();
    let c2: CreakSource = serde_json::from_str(&json).unwrap();
    assert_eq!(c2, CreakSource::Hinge);
}

// ---------------------------------------------------------------------------
// Whoosh
// ---------------------------------------------------------------------------

#[test]
fn test_whoosh_all_types() {
    for wt in &[
        WhooshType::Swing,
        WhooshType::Projectile,
        WhooshType::Vehicle,
        WhooshType::Throw,
    ] {
        let mut w = Whoosh::new(*wt, SR).unwrap();
        w.set_speed(0.7);
        let samples = w.synthesize(0.5).unwrap();
        assert!(samples.iter().all(|s| s.is_finite()));
        assert!(samples.iter().any(|&s| s.abs() > 0.001));
    }
}

#[test]
fn test_whoosh_trigger() {
    let mut w = Whoosh::new(WhooshType::Swing, SR).unwrap();
    w.set_speed(0.8);
    w.trigger();
    let mut buf = vec![0.0f32; 512];
    w.process_block(&mut buf);
    assert!(buf.iter().any(|&s| s.abs() > 0.001));
}

// ---------------------------------------------------------------------------
// Whistle
// ---------------------------------------------------------------------------

#[test]
fn test_whistle_all_sources() {
    for src in &[
        WhistleSource::Gap,
        WhistleSource::Pipe,
        WhistleSource::Bottle,
        WhistleSource::Wire,
    ] {
        let mut w = Whistle::new(*src, SR).unwrap();
        w.set_wind_speed(0.5);
        let samples = w.synthesize(0.5).unwrap();
        assert!(samples.iter().all(|s| s.is_finite()));
        assert!(samples.iter().any(|&s| s.abs() > 0.001));
    }
}

#[test]
fn test_whistle_zero_wind_is_silent() {
    let mut w = Whistle::new(WhistleSource::Pipe, SR).unwrap();
    w.set_wind_speed(0.0);
    let samples = w.synthesize(0.1).unwrap();
    assert!(samples.iter().all(|&s| s.abs() < 0.001));
}

// ---------------------------------------------------------------------------
// Cloth
// ---------------------------------------------------------------------------

#[test]
fn test_cloth_all_types() {
    for ct in &[
        ClothType::Flag,
        ClothType::Cape,
        ClothType::Sail,
        ClothType::Tarp,
    ] {
        let mut c = Cloth::new(*ct, SR).unwrap();
        c.set_wind_speed(0.6);
        let samples = c.synthesize(1.0).unwrap();
        assert!(samples.iter().all(|s| s.is_finite()));
    }
}

#[test]
fn test_cloth_zero_wind_is_silent() {
    let mut c = Cloth::new(ClothType::Flag, SR).unwrap();
    c.set_wind_speed(0.0);
    let samples = c.synthesize(0.1).unwrap();
    assert!(samples.iter().all(|&s| s.abs() < 0.001));
}

// ---------------------------------------------------------------------------
// Aero serde roundtrips
// ---------------------------------------------------------------------------

#[test]
fn test_serde_roundtrip_whoosh_type() {
    let json = serde_json::to_string(&WhooshType::Projectile).unwrap();
    let w2: WhooshType = serde_json::from_str(&json).unwrap();
    assert_eq!(w2, WhooshType::Projectile);
}

#[test]
fn test_serde_roundtrip_whistle_source() {
    let json = serde_json::to_string(&WhistleSource::Bottle).unwrap();
    let w2: WhistleSource = serde_json::from_str(&json).unwrap();
    assert_eq!(w2, WhistleSource::Bottle);
}

#[test]
fn test_serde_roundtrip_cloth_type() {
    let json = serde_json::to_string(&ClothType::Sail).unwrap();
    let c2: ClothType = serde_json::from_str(&json).unwrap();
    assert_eq!(c2, ClothType::Sail);
}

#[test]
fn test_serde_roundtrip_whoosh() {
    let w = Whoosh::new(WhooshType::Swing, SR).unwrap();
    let json = serde_json::to_string(&w).unwrap();
    let w2: Whoosh = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&w2).unwrap();
    assert_eq!(json, json2);
}

#[test]
fn test_serde_roundtrip_whistle() {
    let w = Whistle::new(WhistleSource::Gap, SR).unwrap();
    let json = serde_json::to_string(&w).unwrap();
    let w2: Whistle = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&w2).unwrap();
    assert_eq!(json, json2);
}

#[test]
fn test_serde_roundtrip_cloth() {
    let c = Cloth::new(ClothType::Tarp, SR).unwrap();
    let json = serde_json::to_string(&c).unwrap();
    let c2: Cloth = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&c2).unwrap();
    assert_eq!(json, json2);
}

// ---------------------------------------------------------------------------
// Insect
// ---------------------------------------------------------------------------

#[test]
fn test_insect_all_types() {
    for it in &[
        InsectType::WingBuzz,
        InsectType::CricketChirp,
        InsectType::CicadaDrone,
    ] {
        let mut ins = Insect::new(*it, SR).unwrap();
        ins.set_intensity(0.8);
        let samples = ins.synthesize(0.5).unwrap();
        assert!(samples.iter().all(|s| s.is_finite()), "NaN for {:?}", it);
        assert!(
            samples.iter().any(|&s| s.abs() > 0.001),
            "silent for {:?}",
            it
        );
    }
}

#[test]
fn test_insect_swarm() {
    let mut swarm = Insect::new_swarm(InsectType::WingBuzz, 5, SR).unwrap();
    swarm.set_intensity(0.6);
    let samples = swarm.synthesize(0.5).unwrap();
    assert!(samples.iter().all(|s| s.is_finite()));
    assert!(samples.iter().any(|&s| s.abs() > 0.001));
}

#[test]
fn test_insect_zero_intensity_is_silent() {
    let mut ins = Insect::new(InsectType::WingBuzz, SR).unwrap();
    ins.set_intensity(0.0);
    let samples = ins.synthesize(0.1).unwrap();
    assert!(samples.iter().all(|&s| s.abs() < 0.001));
}

// ---------------------------------------------------------------------------
// WingFlap
// ---------------------------------------------------------------------------

#[test]
fn test_wingflap_all_sizes() {
    for size in &[BirdSize::Small, BirdSize::Medium, BirdSize::Large] {
        let mut wf = WingFlap::new(*size, SR).unwrap();
        wf.set_intensity(0.8);
        let samples = wf.synthesize(0.5).unwrap();
        assert!(samples.iter().all(|s| s.is_finite()), "NaN for {:?}", size);
        assert!(
            samples.iter().any(|&s| s.abs() > 0.001),
            "silent for {:?}",
            size
        );
    }
}

#[test]
fn test_wingflap_zero_intensity_is_silent() {
    let mut wf = WingFlap::new(BirdSize::Medium, SR).unwrap();
    wf.set_intensity(0.0);
    let samples = wf.synthesize(0.1).unwrap();
    assert!(samples.iter().all(|&s| s.abs() < 0.001));
}

// ---------------------------------------------------------------------------
// Bubble
// ---------------------------------------------------------------------------

#[test]
fn test_bubble_all_types() {
    for bt in &[
        BubbleType::Underwater,
        BubbleType::Boiling,
        BubbleType::Viscous,
        BubbleType::Pouring,
    ] {
        let mut b = Bubble::new(*bt, SR).unwrap();
        b.set_intensity(0.8);
        let samples = b.synthesize(1.0).unwrap();
        assert!(samples.iter().all(|s| s.is_finite()), "NaN for {:?}", bt);
    }
}

#[test]
fn test_bubble_zero_intensity_is_silent() {
    let mut b = Bubble::new(BubbleType::Underwater, SR).unwrap();
    b.set_intensity(0.0);
    let samples = b.synthesize(0.1).unwrap();
    assert!(samples.iter().all(|&s| s.abs() < 0.001));
}

// ---------------------------------------------------------------------------
// Creature/fluid serde roundtrips
// ---------------------------------------------------------------------------

#[test]
fn test_serde_roundtrip_insect_type() {
    let json = serde_json::to_string(&InsectType::CricketChirp).unwrap();
    let i2: InsectType = serde_json::from_str(&json).unwrap();
    assert_eq!(i2, InsectType::CricketChirp);
}

#[test]
fn test_serde_roundtrip_bubble_type() {
    let json = serde_json::to_string(&BubbleType::Boiling).unwrap();
    let b2: BubbleType = serde_json::from_str(&json).unwrap();
    assert_eq!(b2, BubbleType::Boiling);
}

#[test]
fn test_serde_roundtrip_bird_size() {
    let json = serde_json::to_string(&BirdSize::Large).unwrap();
    let b2: BirdSize = serde_json::from_str(&json).unwrap();
    assert_eq!(b2, BirdSize::Large);
}

#[test]
fn test_serde_roundtrip_insect() {
    let ins = Insect::new(InsectType::WingBuzz, SR).unwrap();
    let json = serde_json::to_string(&ins).unwrap();
    let i2: Insect = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&i2).unwrap();
    assert_eq!(json, json2);
}

#[test]
fn test_serde_roundtrip_wingflap() {
    let wf = WingFlap::new(BirdSize::Small, SR).unwrap();
    let json = serde_json::to_string(&wf).unwrap();
    let w2: WingFlap = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&w2).unwrap();
    assert_eq!(json, json2);
}

#[test]
fn test_serde_roundtrip_bubble() {
    let b = Bubble::new(BubbleType::Pouring, SR).unwrap();
    let json = serde_json::to_string(&b).unwrap();
    let b2: Bubble = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&b2).unwrap();
    assert_eq!(json, json2);
}

// ---------------------------------------------------------------------------
// Bridge (science crate parameter conversions)
// ---------------------------------------------------------------------------

#[test]
fn test_bridge_rain_intensity_from_rate() {
    use garjan::bridge::rain_intensity_from_rate;
    assert!(rain_intensity_from_rate(0.0).is_none());
    assert_eq!(rain_intensity_from_rate(1.0), Some(RainIntensity::Light));
    assert_eq!(rain_intensity_from_rate(5.0), Some(RainIntensity::Moderate));
    assert_eq!(rain_intensity_from_rate(20.0), Some(RainIntensity::Heavy));
    assert_eq!(
        rain_intensity_from_rate(100.0),
        Some(RainIntensity::Torrential)
    );
}

#[test]
fn test_bridge_wind_speed_normalized() {
    use garjan::bridge::wind_speed_normalized;
    assert_eq!(wind_speed_normalized(0.0), 0.0);
    assert_eq!(wind_speed_normalized(10.0), 0.5);
    assert_eq!(wind_speed_normalized(20.0), 1.0);
    assert_eq!(wind_speed_normalized(40.0), 1.0);
}

#[test]
fn test_bridge_thunder_distance() {
    use garjan::bridge::thunder_distance_from_flash;
    let d = thunder_distance_from_flash(3.0, 20.0);
    assert!(d > 1000.0 && d < 1100.0);
}

#[test]
fn test_bridge_fire_intensity() {
    use garjan::bridge::fire_intensity_from_temperature;
    assert_eq!(fire_intensity_from_temperature(500.0), 0.0);
    assert_eq!(fire_intensity_from_temperature(3000.0), 1.0);
    let campfire = fire_intensity_from_temperature(1200.0);
    assert!(campfire > 0.2 && campfire < 0.4);
}

#[test]
fn test_bridge_weather_from_threat() {
    use garjan::bridge::weather_from_threat_level;
    let (_dist, _speed, rain) = weather_from_threat_level(0);
    assert_eq!(rain, None);
    let (dist, speed, rain) = weather_from_threat_level(5);
    assert!(dist < 200.0);
    assert!(speed > 25.0);
    assert_eq!(rain, Some(RainIntensity::Torrential));
}

#[test]
fn test_bridge_gain_from_distance() {
    use garjan::bridge::gain_from_distance;
    assert_eq!(gain_from_distance(1.0, 1.0), 1.0);
    assert!((gain_from_distance(1.0, 10.0) - 0.1).abs() < 0.001);
    assert_eq!(gain_from_distance(1.0, 0.5), 1.0);
}

// ---------------------------------------------------------------------------
// Precipitation
// ---------------------------------------------------------------------------

#[test]
fn test_precipitation_all_types() {
    for pt in &[
        PrecipitationType::Hail,
        PrecipitationType::Snow,
        PrecipitationType::SurfaceRain,
    ] {
        let mut p = Precipitation::new(*pt, StoneSize::Medium, Terrain::Gravel, SR).unwrap();
        p.set_intensity(0.8);
        let samples = p.synthesize(1.0).unwrap();
        assert!(samples.iter().all(|s| s.is_finite()), "NaN for {:?}", pt);
    }
}

#[test]
fn test_precipitation_hail_on_metal() {
    let mut p = Precipitation::new(
        PrecipitationType::Hail,
        StoneSize::Large,
        Terrain::Metal,
        SR,
    )
    .unwrap();
    p.set_intensity(1.0);
    let samples = p.synthesize(1.0).unwrap();
    assert!(samples.iter().all(|s| s.is_finite()));
}

#[test]
fn test_precipitation_zero_intensity() {
    let mut p =
        Precipitation::new(PrecipitationType::Snow, StoneSize::Small, Terrain::Sand, SR).unwrap();
    p.set_intensity(0.0);
    let samples = p.synthesize(0.1).unwrap();
    assert!(samples.iter().all(|&s| s.abs() < 0.001));
}

// ---------------------------------------------------------------------------
// Underwater
// ---------------------------------------------------------------------------

#[test]
fn test_underwater_all_depths() {
    for depth in &[
        UnderwaterDepth::Shallow,
        UnderwaterDepth::Medium,
        UnderwaterDepth::Deep,
    ] {
        let mut u = Underwater::new(*depth, SR).unwrap();
        u.set_intensity(0.7);
        let samples = u.synthesize(1.0).unwrap();
        assert!(samples.iter().all(|s| s.is_finite()), "NaN for {:?}", depth);
        assert!(
            samples.iter().any(|&s| s.abs() > 0.001),
            "silent for {:?}",
            depth
        );
    }
}

#[test]
fn test_underwater_zero_intensity() {
    let mut u = Underwater::new(UnderwaterDepth::Medium, SR).unwrap();
    u.set_intensity(0.0);
    let samples = u.synthesize(0.1).unwrap();
    assert!(samples.iter().all(|&s| s.abs() < 0.001));
}

// ---------------------------------------------------------------------------
// Surf
// ---------------------------------------------------------------------------

#[test]
fn test_surf_all_intensities() {
    for si in &[
        SurfIntensity::Calm,
        SurfIntensity::Moderate,
        SurfIntensity::Heavy,
        SurfIntensity::Storm,
    ] {
        let mut s = Surf::new(*si, SR).unwrap();
        let samples = s.synthesize(2.0).unwrap();
        assert!(samples.iter().all(|s| s.is_finite()), "NaN for {:?}", si);
        assert!(
            samples.iter().any(|&s| s.abs() > 0.001),
            "silent for {:?}",
            si
        );
    }
}

// ---------------------------------------------------------------------------
// v0.7 serde roundtrips
// ---------------------------------------------------------------------------

#[test]
fn test_serde_roundtrip_precipitation_type() {
    let json = serde_json::to_string(&PrecipitationType::Hail).unwrap();
    let p2: PrecipitationType = serde_json::from_str(&json).unwrap();
    assert_eq!(p2, PrecipitationType::Hail);
}

#[test]
fn test_serde_roundtrip_stone_size() {
    let json = serde_json::to_string(&StoneSize::Large).unwrap();
    let s2: StoneSize = serde_json::from_str(&json).unwrap();
    assert_eq!(s2, StoneSize::Large);
}

#[test]
fn test_serde_roundtrip_underwater_depth() {
    let json = serde_json::to_string(&UnderwaterDepth::Deep).unwrap();
    let u2: UnderwaterDepth = serde_json::from_str(&json).unwrap();
    assert_eq!(u2, UnderwaterDepth::Deep);
}

#[test]
fn test_serde_roundtrip_surf_intensity() {
    let json = serde_json::to_string(&SurfIntensity::Storm).unwrap();
    let s2: SurfIntensity = serde_json::from_str(&json).unwrap();
    assert_eq!(s2, SurfIntensity::Storm);
}

#[test]
fn test_serde_roundtrip_precipitation() {
    let p = Precipitation::new(
        PrecipitationType::Hail,
        StoneSize::Medium,
        Terrain::Tile,
        SR,
    )
    .unwrap();
    let json = serde_json::to_string(&p).unwrap();
    let p2: Precipitation = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&p2).unwrap();
    assert_eq!(json, json2);
}

#[test]
fn test_serde_roundtrip_underwater() {
    let u = Underwater::new(UnderwaterDepth::Shallow, SR).unwrap();
    let json = serde_json::to_string(&u).unwrap();
    let u2: Underwater = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&u2).unwrap();
    assert_eq!(json, json2);
}

#[test]
fn test_serde_roundtrip_surf() {
    let s = Surf::new(SurfIntensity::Heavy, SR).unwrap();
    let json = serde_json::to_string(&s).unwrap();
    let s2: Surf = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&s2).unwrap();
    assert_eq!(json, json2);
}

// ---------------------------------------------------------------------------
// Voice management
// ---------------------------------------------------------------------------

#[test]
fn test_voice_pool_basic() {
    let mut pool = VoicePool::new(4, StealPolicy::Oldest);
    assert_eq!(pool.max_voices(), 4);
    assert_eq!(pool.active_count(), 0);

    let idx = pool.allocate(5, 100).unwrap();
    assert_eq!(idx, 0);
    assert_eq!(pool.active_count(), 1);
    assert!(pool.slot(0).unwrap().active);
    assert_eq!(pool.slot(0).unwrap().tag, 100);

    pool.release(0);
    assert_eq!(pool.active_count(), 0);
}

#[test]
fn test_voice_pool_fill_and_steal_oldest() {
    let mut pool = VoicePool::new(2, StealPolicy::Oldest);
    pool.allocate(5, 1).unwrap(); // slot 0
    pool.tick();
    pool.allocate(5, 2).unwrap(); // slot 1
    assert_eq!(pool.active_count(), 2);

    // Pool full — should steal oldest (slot 0, age=1)
    let stolen = pool.allocate(5, 3).unwrap();
    assert_eq!(stolen, 0);
    assert_eq!(pool.slot(0).unwrap().tag, 3);
}

#[test]
fn test_voice_pool_steal_lowest_priority() {
    let mut pool = VoicePool::new(2, StealPolicy::LowestPriority);
    pool.allocate(10, 1).unwrap(); // slot 0, high priority
    pool.allocate(1, 2).unwrap(); // slot 1, low priority

    // New voice with priority 5 — should steal slot 1 (priority 1)
    let stolen = pool.allocate(5, 3).unwrap();
    assert_eq!(stolen, 1);
    assert_eq!(pool.slot(1).unwrap().tag, 3);
}

#[test]
fn test_voice_pool_no_steal_policy() {
    let mut pool = VoicePool::new(1, StealPolicy::None);
    pool.allocate(5, 1).unwrap();
    assert!(pool.allocate(5, 2).is_none()); // rejected
}

#[test]
fn test_voice_pool_no_steal_higher_priority() {
    let mut pool = VoicePool::new(1, StealPolicy::LowestPriority);
    pool.allocate(10, 1).unwrap(); // high priority
    // Lower priority voice should not steal
    assert!(pool.allocate(5, 2).is_none());
}

#[test]
fn test_voice_pool_tick_ages() {
    let mut pool = VoicePool::new(2, StealPolicy::Oldest);
    pool.allocate(5, 1).unwrap();
    assert_eq!(pool.slot(0).unwrap().age, 0);
    pool.tick();
    assert_eq!(pool.slot(0).unwrap().age, 1);
    pool.tick();
    assert_eq!(pool.slot(0).unwrap().age, 2);
}

#[test]
fn test_voice_pool_release_all() {
    let mut pool = VoicePool::new(4, StealPolicy::Oldest);
    pool.allocate(5, 1).unwrap();
    pool.allocate(5, 2).unwrap();
    pool.allocate(5, 3).unwrap();
    assert_eq!(pool.active_count(), 3);
    pool.release_all();
    assert_eq!(pool.active_count(), 0);
}

#[test]
fn test_voice_pool_active_voices_iterator() {
    let mut pool = VoicePool::new(4, StealPolicy::Oldest);
    pool.allocate(5, 10).unwrap();
    pool.allocate(3, 20).unwrap();
    let tags: Vec<u32> = pool.active_voices().map(|(_, s)| s.tag).collect();
    assert_eq!(tags, vec![10, 20]);
}

#[test]
fn test_serde_roundtrip_voice_pool() {
    let mut pool = VoicePool::new(4, StealPolicy::LowestPriority);
    pool.allocate(5, 42).unwrap();
    pool.tick();
    let json = serde_json::to_string(&pool).unwrap();
    let p2: VoicePool = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&p2).unwrap();
    assert_eq!(json, json2);
}

#[test]
fn test_serde_roundtrip_steal_policy() {
    let json = serde_json::to_string(&StealPolicy::LowestPriority).unwrap();
    let s2: StealPolicy = serde_json::from_str(&json).unwrap();
    assert_eq!(s2, StealPolicy::LowestPriority);
}
