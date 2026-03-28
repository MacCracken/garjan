//! Criterion benchmarks for garjan environmental sound synthesis.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use garjan::prelude::*;

const SR: f32 = 44100.0;

fn bench_thunder_1s(c: &mut Criterion) {
    c.bench_function("thunder_1s", |b| {
        let mut thunder = Thunder::new(500.0, SR).unwrap();
        b.iter(|| {
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
);

criterion_main!(benches);
