//! Weather scene: thunder, rain, and wind layered together.
//!
//! Demonstrates combining multiple garjan synthesizers to create
//! a complete weather soundscape.

use garjan::prelude::*;

fn main() {
    let sr = 44100.0;

    // Create weather components
    let mut thunder = Thunder::new(800.0, sr).unwrap();
    let mut rain = Rain::new(RainIntensity::Heavy, sr).unwrap();
    let mut wind = Wind::new(18.0, 0.7, sr).unwrap();

    // Synthesize each layer
    let thunder_audio = thunder.synthesize(5.0).unwrap();
    let rain_audio = rain.synthesize(5.0).unwrap();
    let wind_audio = wind.synthesize(5.0).unwrap();

    // Mix layers (simple sum — in production, use dhvani for proper mixing)
    let num_samples = thunder_audio
        .len()
        .min(rain_audio.len())
        .min(wind_audio.len());
    let mut mix = vec![0.0f32; num_samples];
    for i in 0..num_samples {
        mix[i] = thunder_audio[i] * 0.5 + rain_audio[i] * 0.3 + wind_audio[i] * 0.2;
    }

    // In a real application, write `mix` to an audio output
    let peak = mix.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    let rms = (mix.iter().map(|s| s * s).sum::<f32>() / num_samples as f32).sqrt();
    println!(
        "Weather scene: {} samples, peak={:.4}, rms={:.4}",
        num_samples, peak, rms
    );
}
