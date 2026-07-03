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

fn bench_footstep_gravel_walk_1s(c: &mut Criterion) {
    c.bench_function("footstep_gravel_walk_1s", |b| {
        let mut fs = garjan::footstep::Footstep::new(
            garjan::contact::Terrain::Gravel,
            garjan::contact::MovementType::Walk,
            SR,
        )
        .unwrap();
        b.iter(|| {
            let samples = fs.synthesize(1.0).unwrap();
            black_box(samples);
        });
    });
}

fn bench_friction_scrape_metal_1s(c: &mut Criterion) {
    c.bench_function("friction_scrape_metal_1s", |b| {
        let mut f = garjan::friction::Friction::new(
            garjan::contact::FrictionType::Scrape,
            Material::Metal,
            SR,
        )
        .unwrap();
        f.set_velocity(0.5);
        f.set_pressure(0.5);
        b.iter(|| {
            let samples = f.synthesize(1.0).unwrap();
            black_box(samples);
        });
    });
}

fn bench_creak_door_1s(c: &mut Criterion) {
    c.bench_function("creak_door_1s", |b| {
        let mut c_synth =
            garjan::creak::Creak::new(garjan::contact::CreakSource::Door, SR).unwrap();
        c_synth.set_tension(0.5);
        c_synth.set_speed(0.5);
        b.iter(|| {
            let samples = c_synth.synthesize(1.0).unwrap();
            black_box(samples);
        });
    });
}

fn bench_rolling_wheel_wood_1s(c: &mut Criterion) {
    c.bench_function("rolling_wheel_wood_1s", |b| {
        let mut r =
            garjan::rolling::Rolling::new(garjan::contact::RollingBody::Wheel, Material::Wood, SR)
                .unwrap();
        r.set_velocity(0.5);
        b.iter(|| {
            let samples = r.synthesize(1.0).unwrap();
            black_box(samples);
        });
    });
}

fn bench_foliage_rustle_1s(c: &mut Criterion) {
    c.bench_function("foliage_rustle_1s", |b| {
        let mut f =
            garjan::foliage::Foliage::new(garjan::contact::FoliageType::LeafRustle, SR).unwrap();
        f.set_wind_speed(0.5);
        b.iter(|| {
            let samples = f.synthesize(1.0).unwrap();
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
    bench_footstep_gravel_walk_1s,
    bench_friction_scrape_metal_1s,
    bench_creak_door_1s,
    bench_rolling_wheel_wood_1s,
    bench_foliage_rustle_1s,
    bench_whoosh_swing,
    bench_whistle_pipe_1s,
    bench_cloth_flag_1s,
    bench_insect_swarm_1s,
    bench_wingflap_medium_1s,
    bench_bubble_boiling_1s,
    bench_precipitation_hail_1s,
    bench_underwater_medium_1s,
    bench_surf_moderate_2s,
);

fn bench_insect_swarm_1s(c: &mut Criterion) {
    c.bench_function("insect_swarm_5_1s", |b| {
        let mut swarm =
            garjan::insect::Insect::new_swarm(garjan::creature::InsectType::WingBuzz, 5, SR)
                .unwrap();
        swarm.set_intensity(0.6);
        b.iter(|| {
            let samples = swarm.synthesize(1.0).unwrap();
            black_box(samples);
        });
    });
}

fn bench_wingflap_medium_1s(c: &mut Criterion) {
    c.bench_function("wingflap_medium_1s", |b| {
        let mut wf =
            garjan::wingflap::WingFlap::new(garjan::wingflap::BirdSize::Medium, SR).unwrap();
        wf.set_intensity(0.8);
        b.iter(|| {
            let samples = wf.synthesize(1.0).unwrap();
            black_box(samples);
        });
    });
}

fn bench_bubble_boiling_1s(c: &mut Criterion) {
    c.bench_function("bubble_boiling_1s", |b| {
        let mut bub =
            garjan::bubble::Bubble::new(garjan::creature::BubbleType::Boiling, SR).unwrap();
        bub.set_intensity(0.8);
        b.iter(|| {
            let samples = bub.synthesize(1.0).unwrap();
            black_box(samples);
        });
    });
}

fn bench_whoosh_swing(c: &mut Criterion) {
    c.bench_function("whoosh_swing", |b| {
        let mut w = garjan::whoosh::Whoosh::new(garjan::aero::WhooshType::Swing, SR).unwrap();
        w.set_speed(0.8);
        b.iter(|| {
            let samples = w.synthesize(0.5).unwrap();
            black_box(samples);
        });
    });
}

fn bench_whistle_pipe_1s(c: &mut Criterion) {
    c.bench_function("whistle_pipe_1s", |b| {
        let mut w = garjan::whistle::Whistle::new(garjan::aero::WhistleSource::Pipe, SR).unwrap();
        w.set_wind_speed(0.5);
        b.iter(|| {
            let samples = w.synthesize(1.0).unwrap();
            black_box(samples);
        });
    });
}

fn bench_cloth_flag_1s(c: &mut Criterion) {
    c.bench_function("cloth_flag_1s", |b| {
        let mut c_synth = garjan::cloth::Cloth::new(garjan::aero::ClothType::Flag, SR).unwrap();
        c_synth.set_wind_speed(0.6);
        b.iter(|| {
            let samples = c_synth.synthesize(1.0).unwrap();
            black_box(samples);
        });
    });
}

fn bench_precipitation_hail_1s(c: &mut Criterion) {
    c.bench_function("precipitation_hail_1s", |b| {
        let mut p = garjan::precipitation::Precipitation::new(
            garjan::precipitation::PrecipitationType::Hail,
            garjan::precipitation::StoneSize::Medium,
            garjan::contact::Terrain::Metal,
            SR,
        )
        .unwrap();
        p.set_intensity(0.8);
        b.iter(|| {
            let samples = p.synthesize(1.0).unwrap();
            black_box(samples);
        });
    });
}

fn bench_underwater_medium_1s(c: &mut Criterion) {
    c.bench_function("underwater_medium_1s", |b| {
        let mut u =
            garjan::underwater::Underwater::new(garjan::underwater::UnderwaterDepth::Medium, SR)
                .unwrap();
        u.set_intensity(0.7);
        b.iter(|| {
            let samples = u.synthesize(1.0).unwrap();
            black_box(samples);
        });
    });
}

fn bench_surf_moderate_2s(c: &mut Criterion) {
    c.bench_function("surf_moderate_2s", |b| {
        let mut s = garjan::surf::Surf::new(garjan::surf::SurfIntensity::Moderate, SR).unwrap();
        b.iter(|| {
            let samples = s.synthesize(2.0).unwrap();
            black_box(samples);
        });
    });
}

criterion_main!(benches);
