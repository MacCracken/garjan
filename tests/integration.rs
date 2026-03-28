//! Integration tests for garjan.

use garjan::prelude::*;
use garjan::texture::TextureType;

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
