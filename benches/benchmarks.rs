//! Criterion benchmarks for garjan environmental sound synthesis.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use garjan::prelude::*;

fn bench_thunder_1s(c: &mut Criterion) {
    c.bench_function("thunder_1s", |b| {
        let mut thunder = Thunder::new(500.0);
        b.iter(|| {
            let samples = thunder.synthesize(44100.0, 1.0).unwrap();
            black_box(samples);
        });
    });
}

fn bench_rain_moderate_1s(c: &mut Criterion) {
    c.bench_function("rain_moderate_1s", |b| {
        let mut rain = Rain::new(RainIntensity::Moderate);
        b.iter(|| {
            let samples = rain.synthesize(44100.0, 1.0).unwrap();
            black_box(samples);
        });
    });
}

fn bench_wind_1s(c: &mut Criterion) {
    c.bench_function("wind_1s", |b| {
        let mut wind = Wind::new(15.0, 0.5);
        b.iter(|| {
            let samples = wind.synthesize(44100.0, 1.0).unwrap();
            black_box(samples);
        });
    });
}

fn bench_impact_wood_strike(c: &mut Criterion) {
    c.bench_function("impact_wood_strike", |b| {
        let mut impact = Impact::new(Material::Wood);
        b.iter(|| {
            let samples = impact.synthesize(ImpactType::Strike, 44100.0).unwrap();
            black_box(samples);
        });
    });
}

fn bench_fire_1s(c: &mut Criterion) {
    c.bench_function("fire_1s", |b| {
        let mut fire = Fire::new(0.7);
        b.iter(|| {
            let samples = fire.synthesize(44100.0, 1.0).unwrap();
            black_box(samples);
        });
    });
}

fn bench_water_stream_1s(c: &mut Criterion) {
    c.bench_function("water_stream_1s", |b| {
        let mut water = Water::new(WaterType::Stream, 0.5);
        b.iter(|| {
            let samples = water.synthesize(44100.0, 1.0).unwrap();
            black_box(samples);
        });
    });
}

fn bench_water_waves_1s(c: &mut Criterion) {
    c.bench_function("water_waves_1s", |b| {
        let mut water = Water::new(WaterType::Waves, 0.5);
        b.iter(|| {
            let samples = water.synthesize(44100.0, 1.0).unwrap();
            black_box(samples);
        });
    });
}

fn bench_forest_texture_1s(c: &mut Criterion) {
    c.bench_function("forest_texture_1s", |b| {
        let mut tex = AmbientTexture::new(garjan::texture::TextureType::Forest, 0.5);
        b.iter(|| {
            let samples = tex.synthesize(44100.0, 1.0).unwrap();
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
);

criterion_main!(benches);
