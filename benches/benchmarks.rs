//! Criterion benchmarks for garjan environmental sound synthesis.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use garjan::prelude::*;

const SR: f32 = 44100.0;

fn bench_thunder_1s(c: &mut Criterion) {
    c.bench_function("thunder_1s", |b| {
        b.iter(|| {
            let mut thunder = Thunder::new(500.0, SR).unwrap();
            let samples = thunder.synthesize(1.0).unwrap();
            black_box(samples);
        });
    });
}

fn bench_rain_moderate_1s(c: &mut Criterion) {
    c.bench_function("rain_moderate_1s", |b| {
        let mut rain = Rain::new(RainIntensity::Moderate, SR).unwrap();
        b.iter(|| {
            let samples = rain.synthesize(1.0).unwrap();
            black_box(samples);
        });
    });
}

fn bench_wind_1s(c: &mut Criterion) {
    c.bench_function("wind_1s", |b| {
        let mut wind = Wind::new(15.0, 0.5, SR).unwrap();
        b.iter(|| {
            let samples = wind.synthesize(1.0).unwrap();
            black_box(samples);
        });
    });
}

fn bench_impact_wood_strike(c: &mut Criterion) {
    c.bench_function("impact_wood_strike", |b| {
        let mut impact = Impact::new(Material::Wood, SR).unwrap();
        b.iter(|| {
            let samples = impact.synthesize(ImpactType::Strike).unwrap();
            black_box(samples);
        });
    });
}

fn bench_fire_1s(c: &mut Criterion) {
    c.bench_function("fire_1s", |b| {
        let mut fire = Fire::new(0.7, SR).unwrap();
        b.iter(|| {
            let samples = fire.synthesize(1.0).unwrap();
            black_box(samples);
        });
    });
}

fn bench_water_stream_1s(c: &mut Criterion) {
    c.bench_function("water_stream_1s", |b| {
        let mut water = Water::new(WaterType::Stream, 0.5, SR).unwrap();
        b.iter(|| {
            let samples = water.synthesize(1.0).unwrap();
            black_box(samples);
        });
    });
}

fn bench_water_waves_1s(c: &mut Criterion) {
    c.bench_function("water_waves_1s", |b| {
        let mut water = Water::new(WaterType::Waves, 0.5, SR).unwrap();
        b.iter(|| {
            let samples = water.synthesize(1.0).unwrap();
            black_box(samples);
        });
    });
}

fn bench_forest_texture_1s(c: &mut Criterion) {
    c.bench_function("forest_texture_1s", |b| {
        let mut tex = AmbientTexture::new(garjan::texture::TextureType::Forest, 0.5, SR).unwrap();
        b.iter(|| {
            let samples = tex.synthesize(1.0).unwrap();
            black_box(samples);
        });
    });
}

fn bench_process_block_wind_512(c: &mut Criterion) {
    c.bench_function("process_block_wind_512", |b| {
        let mut wind = Wind::new(15.0, 0.5, SR).unwrap();
        let mut buf = vec![0.0f32; 512];
        b.iter(|| {
            wind.process_block(&mut buf);
            black_box(&buf);
        });
    });
}

fn bench_modal_bank_8_modes_512(c: &mut Criterion) {
    c.bench_function("modal_bank_8_modes_512", |b| {
        let specs: Vec<_> = (1..=8)
            .map(|k| garjan::modal::ModeSpec {
                frequency: 440.0 * k as f32,
                amplitude: 1.0 / k as f32,
                decay: 0.5,
            })
            .collect();
        let mut bank = garjan::modal::ModalBank::new(&specs, SR).unwrap();
        let excitation = vec![0.0f32; 512];
        let mut output = vec![0.0f32; 512];
        // Prime with impulse
        bank.process_sample(1.0);
        b.iter(|| {
            bank.process_block(&excitation, &mut output);
            black_box(&output);
        });
    });
}

fn bench_impact_metal_strike(c: &mut Criterion) {
    c.bench_function("impact_metal_strike", |b| {
        let mut impact = Impact::new(Material::Metal, SR).unwrap();
        b.iter(|| {
            let samples = impact.synthesize(ImpactType::Strike).unwrap();
            black_box(samples);
        });
    });
}

fn bench_impact_glass_shatter(c: &mut Criterion) {
    c.bench_function("impact_glass_shatter", |b| {
        let mut impact = Impact::new(Material::Glass, SR).unwrap();
        b.iter(|| {
            let samples = impact.synthesize(ImpactType::Shatter).unwrap();
            black_box(samples);
        });
    });
}

criterion_group!(
    benches,
    bench_thunder_1s,
    bench_rain_moderate_1s,
    bench_wind_1s,
    bench_impact_wood_strike,
    bench_fire_1s,
    bench_water_stream_1s,
    bench_water_waves_1s,
    bench_forest_texture_1s,
    bench_process_block_wind_512,
    bench_modal_bank_8_modes_512,
    bench_impact_metal_strike,
    bench_impact_glass_shatter,
);

criterion_main!(benches);
