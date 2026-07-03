//! Logging: demonstrates structured tracing output from garjan.
//!
//! Run with: cargo run --example logging --features logging
//!
//! To see trace output, set the RUST_LOG env var:
//!   RUST_LOG=garjan=trace cargo run --example logging --features logging

fn main() {
    // Initialize a tracing subscriber (requires tracing-subscriber in your app)
    // garjan itself does NOT depend on tracing-subscriber — only on tracing.
    // Your application provides the subscriber.
    //
    // Example with tracing-subscriber (add to your Cargo.toml):
    //   tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
    //
    // Then:
    //   tracing_subscriber::fmt()
    //       .with_env_filter("garjan=trace")
    //       .init();

    println!("garjan logging example");
    println!();
    println!("When the 'logging' feature is enabled, garjan emits tracing events:");
    println!("  - WARN on invalid parameters (sample_rate, duration)");
    println!("  - ERROR on naad backend initialization failures");
    println!();
    println!("To see these in your application:");
    println!("  1. Add 'logging' to garjan features in Cargo.toml:");
    println!("     garjan = {{ version = \"1\", features = [\"logging\"] }}");
    println!("  2. Initialize a tracing subscriber in your app's main()");
    println!("  3. Set RUST_LOG=garjan=trace (or warn/error)");
    println!();

    // Trigger a validation warning (this emits a tracing::warn! when logging is on)
    use garjan::prelude::*;
    match Thunder::new(500.0, -1.0) {
        Err(e) => println!("Got expected error: {e}"),
        Ok(_) => unreachable!(),
    }

    match Wind::new(10.0, 0.5, 44100.0) {
        Ok(mut w) => match w.synthesize(f32::INFINITY) {
            Err(e) => println!("Got expected error: {e}"),
            Ok(_) => unreachable!(),
        },
        Err(e) => println!("Unexpected: {e}"),
    }
}
