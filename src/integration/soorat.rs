//! Soorat integration — visualization data structures for garjan sound sources.
//!
//! Provides spatial and visual parameter structs that soorat can render
//! alongside garjan's audio synthesis. Each struct captures the visual
//! aspect of a sound source at a point in time.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

// ── Rain / Snow particle field ─────────────────────────────────────────────

/// A single precipitation particle for visual rendering.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PrecipitationParticle {
    /// World-space position `[x, y, z]` in metres.
    pub position: [f32; 3],
    /// Velocity `[vx, vy, vz]` in m/s (primarily downward).
    pub velocity: [f32; 3],
    /// Particle size in metres (raindrop diameter or snowflake extent).
    pub size: f32,
    /// Remaining lifetime in seconds (0 = hit ground, despawn).
    pub life: f32,
}

/// A field of precipitation particles for visual rendering.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrecipitationField {
    /// Active particles.
    pub particles: Vec<PrecipitationParticle>,
    /// Whether this is rain (elongated streaks) or snow (slow, tumbling).
    pub is_snow: bool,
    /// Intensity (0.0–1.0) for density/opacity scaling.
    pub intensity: f32,
    /// Wind bias `[wx, wz]` in m/s applied to all particles (horizontal).
    pub wind_bias: [f32; 2],
}

impl PrecipitationField {
    /// Create a rain field with the given number of particles in a volume.
    ///
    /// `bounds_min` / `bounds_max`: world-space AABB for particle spawning.
    /// `intensity`: 0.0–1.0 controlling density and drop size.
    /// `wind_x`, `wind_z`: horizontal wind in m/s.
    #[must_use]
    pub fn rain(
        count: usize,
        bounds_min: [f32; 3],
        bounds_max: [f32; 3],
        intensity: f32,
        wind_x: f32,
        wind_z: f32,
    ) -> Self {
        let mut particles = Vec::with_capacity(count);
        let dx = bounds_max[0] - bounds_min[0];
        let dy = bounds_max[1] - bounds_min[1];
        let dz = bounds_max[2] - bounds_min[2];

        for i in 0..count {
            // Deterministic spread (consumer can jitter with their own RNG)
            let t = i as f32 / count.max(1) as f32;
            let x = bounds_min[0] + (t * 7.13).fract() * dx;
            let y = bounds_min[1] + (t * 3.37).fract() * dy;
            let z = bounds_min[2] + (t * 11.79).fract() * dz;

            let drop_size = 0.001 + intensity * 0.004; // 1–5mm
            let fall_speed = -4.0 - intensity * 5.0; // 4–9 m/s

            particles.push(PrecipitationParticle {
                position: [x, y, z],
                velocity: [wind_x * 0.3, fall_speed, wind_z * 0.3],
                size: drop_size,
                life: y - bounds_min[1], // time to hit ground ≈ height / speed
            });
        }

        Self {
            particles,
            is_snow: false,
            intensity: intensity.clamp(0.0, 1.0),
            wind_bias: [wind_x, wind_z],
        }
    }

    /// Create a snow field.
    #[must_use]
    pub fn snow(
        count: usize,
        bounds_min: [f32; 3],
        bounds_max: [f32; 3],
        intensity: f32,
        wind_x: f32,
        wind_z: f32,
    ) -> Self {
        let mut field = Self::rain(count, bounds_min, bounds_max, intensity, wind_x, wind_z);
        field.is_snow = true;
        // Snow falls slower and is larger
        for p in &mut field.particles {
            p.velocity[1] *= 0.2; // much slower fall
            p.size *= 3.0; // larger flakes
        }
        field
    }
}

// ── Fire emitter ───────────────────────────────────────────────────────────

/// Visual parameters for a fire source at a point in time.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FireEmitter {
    /// World-space center position `[x, y, z]`.
    pub position: [f32; 3],
    /// Fire intensity (0.0–1.0): controls flame height, particle density, brightness.
    pub intensity: f32,
    /// Color temperature in Kelvin (small fire ≈ 1200K, large fire ≈ 1800K).
    pub color_temperature_k: f32,
    /// Flame height in metres (for particle spawn volume).
    pub flame_height: f32,
    /// Base radius in metres.
    pub base_radius: f32,
    /// Ember count per second (for particle emission rate).
    pub ember_rate: f32,
}

impl FireEmitter {
    /// Create a fire emitter from intensity (0.0–1.0).
    #[must_use]
    pub fn from_intensity(position: [f32; 3], intensity: f32) -> Self {
        let i = intensity.clamp(0.0, 1.0);
        Self {
            position,
            intensity: i,
            color_temperature_k: 1200.0 + i * 600.0,
            flame_height: 0.3 + i * 2.0,
            base_radius: 0.1 + i * 0.5,
            ember_rate: i * 50.0,
        }
    }
}

// ── Wind flow field ────────────────────────────────────────────────────────

/// A 2D grid of wind velocity vectors for visualization.
///
/// Grid is in the XZ plane at a given Y height. Each cell stores
/// a horizontal velocity `[vx, vz]` in m/s.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindField {
    /// Wind velocity at each grid point `[vx, vz]` in m/s.
    /// Flattened row-major: `velocities[z * nx + x]`.
    pub velocities: Vec<[f32; 2]>,
    /// Grid dimensions (nx, nz).
    pub dimensions: [usize; 2],
    /// World-space origin of the grid (min corner) `[x, y, z]`.
    pub origin: [f32; 3],
    /// Grid cell spacing in metres.
    pub spacing: f32,
    /// Overall wind speed (m/s) for intensity scaling.
    pub speed: f32,
}

impl WindField {
    /// Create a uniform wind field (constant direction).
    #[must_use]
    pub fn uniform(
        nx: usize,
        nz: usize,
        origin: [f32; 3],
        spacing: f32,
        wind_vx: f32,
        wind_vz: f32,
    ) -> Self {
        let count = nx * nz;
        let velocities = alloc::vec![[wind_vx, wind_vz]; count];
        let speed = (wind_vx * wind_vx + wind_vz * wind_vz).sqrt();
        Self {
            velocities,
            dimensions: [nx, nz],
            origin,
            spacing,
            speed,
        }
    }

    /// Create a wind field with a simple gradient (speed increases along X).
    #[must_use]
    pub fn gradient(
        nx: usize,
        nz: usize,
        origin: [f32; 3],
        spacing: f32,
        min_speed: f32,
        max_speed: f32,
        direction_z: f32,
    ) -> Self {
        let count = nx * nz;
        let mut velocities = Vec::with_capacity(count);
        let avg_speed = (min_speed + max_speed) * 0.5;

        for iz in 0..nz {
            for ix in 0..nx {
                let t = if nx > 1 {
                    ix as f32 / (nx - 1) as f32
                } else {
                    0.5
                };
                let speed = min_speed + t * (max_speed - min_speed);
                let _ = iz; // uniform across Z
                velocities.push([0.0, speed * direction_z.signum()]);
            }
        }

        Self {
            velocities,
            dimensions: [nx, nz],
            origin,
            spacing,
            speed: avg_speed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rain_field_particle_count() {
        let field =
            PrecipitationField::rain(100, [0.0, 0.0, 0.0], [10.0, 20.0, 10.0], 0.5, 2.0, 0.0);
        assert_eq!(field.particles.len(), 100);
        assert!(!field.is_snow);
        assert!((field.intensity - 0.5).abs() < 0.01);
    }

    #[test]
    fn rain_field_empty() {
        let field = PrecipitationField::rain(0, [0.0; 3], [10.0; 3], 0.5, 0.0, 0.0);
        assert!(field.particles.is_empty());
    }

    #[test]
    fn snow_field_slower() {
        let rain = PrecipitationField::rain(10, [0.0; 3], [10.0, 20.0, 10.0], 0.5, 0.0, 0.0);
        let snow = PrecipitationField::snow(10, [0.0; 3], [10.0, 20.0, 10.0], 0.5, 0.0, 0.0);
        assert!(snow.is_snow);
        // Snow falls slower
        assert!(snow.particles[0].velocity[1].abs() < rain.particles[0].velocity[1].abs());
        // Snow is larger
        assert!(snow.particles[0].size > rain.particles[0].size);
    }

    #[test]
    fn fire_emitter_low_intensity() {
        let fire = FireEmitter::from_intensity([0.0, 0.0, 0.0], 0.0);
        assert_eq!(fire.intensity, 0.0);
        assert!(fire.color_temperature_k >= 1200.0);
        assert!(fire.flame_height > 0.0);
    }

    #[test]
    fn fire_emitter_high_intensity() {
        let fire = FireEmitter::from_intensity([5.0, 0.0, 3.0], 1.0);
        assert_eq!(fire.intensity, 1.0);
        assert!(fire.color_temperature_k > 1500.0);
        assert!(fire.flame_height > 1.0);
        assert!(fire.ember_rate > 0.0);
    }

    #[test]
    fn fire_emitter_clamps() {
        let fire = FireEmitter::from_intensity([0.0; 3], 5.0);
        assert_eq!(fire.intensity, 1.0);
    }

    #[test]
    fn wind_field_uniform() {
        let field = WindField::uniform(4, 4, [0.0; 3], 1.0, 5.0, 0.0);
        assert_eq!(field.velocities.len(), 16);
        assert_eq!(field.dimensions, [4, 4]);
        for v in &field.velocities {
            assert_eq!(v[0], 5.0);
            assert_eq!(v[1], 0.0);
        }
        assert!((field.speed - 5.0).abs() < 0.01);
    }

    #[test]
    fn wind_field_gradient() {
        let field = WindField::gradient(5, 3, [0.0; 3], 2.0, 1.0, 10.0, 1.0);
        assert_eq!(field.velocities.len(), 15);
        // First column should be slower than last
        let first = field.velocities[0][1].abs();
        let last = field.velocities[4][1].abs();
        assert!(last > first);
    }

    #[test]
    fn wind_field_single_cell() {
        let field = WindField::uniform(1, 1, [0.0; 3], 1.0, 3.0, 4.0);
        assert_eq!(field.velocities.len(), 1);
        assert!((field.speed - 5.0).abs() < 0.01); // 3-4-5 triangle
    }

    #[test]
    fn precipitation_particles_within_bounds() {
        let min = [-5.0, 0.0, -5.0];
        let max = [5.0, 20.0, 5.0];
        let field = PrecipitationField::rain(50, min, max, 0.5, 0.0, 0.0);
        for p in &field.particles {
            assert!(p.position[0] >= min[0] && p.position[0] <= max[0]);
            assert!(p.position[1] >= min[1] && p.position[1] <= max[1]);
            assert!(p.position[2] >= min[2] && p.position[2] <= max[2]);
        }
    }
}
