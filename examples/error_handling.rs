//! Error handling: demonstrates how garjan reports errors
//! and how to handle them in application code.

use garjan::prelude::*;

fn main() {
    let sr = 44100.0;

    // Invalid sample rate
    match Thunder::new(500.0, 0.0) {
        Ok(_) => unreachable!(),
        Err(e) => println!("Expected error: {e}"),
    }

    // Invalid duration
    let mut wind = Wind::new(10.0, 0.5, sr).unwrap();
    match wind.synthesize(-1.0) {
        Ok(_) => unreachable!(),
        Err(e) => println!("Expected error: {e}"),
    }

    // Pattern: propagate with ?
    if let Err(e) = run_synthesis(sr) {
        eprintln!("Synthesis failed: {e}");
    }

    // Pattern: match on error variant
    match Rain::new(RainIntensity::Heavy, f32::NAN) {
        Err(GarjanError::InvalidParameter(msg)) => {
            println!("Parameter error: {msg}");
        }
        Err(GarjanError::SynthesisFailed(msg)) => {
            println!("Synthesis error: {msg}");
        }
        Err(e) => println!("Other error: {e}"),
        Ok(_) => unreachable!(),
    }
}

fn run_synthesis(sr: f32) -> garjan::error::Result<()> {
    let mut fire = Fire::new(0.7, sr)?;
    let _samples = fire.synthesize(1.0)?;
    println!("Fire synthesis OK");
    Ok(())
}
