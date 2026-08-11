//! Deterministic gradient noise (Perlin-style) with fBm, ridges, and warp.
//!
//! Transcendental-free by rule (docs/13-worldgen.md): gradients come from a
//! constant direction table via the counter RNG, fades are polynomial — the
//! same inputs produce the same bits on every platform.

use sim_events::rng;
use sim_events::{SystemId, WorldSeed};
use world_schema::Tick;

const NOISE: SystemId = SystemId(3);

/// 16 unit directions (22.5° steps) as precomputed constants — no runtime trig.
const DIRS: [(f32, f32); 16] = [
    (1.0, 0.0),
    (0.923_879_5, 0.382_683_43),
    (0.707_106_77, 0.707_106_77),
    (0.382_683_43, 0.923_879_5),
    (0.0, 1.0),
    (-0.382_683_43, 0.923_879_5),
    (-0.707_106_77, 0.707_106_77),
    (-0.923_879_5, 0.382_683_43),
    (-1.0, 0.0),
    (-0.923_879_5, -0.382_683_43),
    (-0.707_106_77, -0.707_106_77),
    (-0.382_683_43, -0.923_879_5),
    (0.0, -1.0),
    (0.382_683_43, -0.923_879_5),
    (0.707_106_77, -0.707_106_77),
    (0.923_879_5, -0.382_683_43),
];

/// Distinct noise fields are separated by channel so octaves never collide.
#[derive(Debug, Clone, Copy)]
pub struct Channel(pub u32);

fn gradient(seed: WorldSeed, channel: u32, cx: i64, cy: i64) -> (f32, f32) {
    let key =
        (u64::from(channel) << 48) ^ ((cx as u64 & 0xFF_FFFF) << 24) ^ (cy as u64 & 0xFF_FFFF);
    DIRS[(rng::draw(seed, Tick::ZERO, NOISE, key) & 15) as usize]
}

fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// One octave of gradient noise, roughly in `[-1, 1]`.
#[must_use]
pub fn perlin(seed: WorldSeed, channel: u32, x: f32, y: f32) -> f32 {
    let x0 = x.floor();
    let y0 = y.floor();
    let (cx, cy) = (x0 as i64, y0 as i64);
    let (fx, fy) = (x - x0, y - y0);
    let dot = |gx: i64, gy: i64, dx: f32, dy: f32| {
        let (vx, vy) = gradient(seed, channel, gx, gy);
        vx * dx + vy * dy
    };
    let n00 = dot(cx, cy, fx, fy);
    let n10 = dot(cx + 1, cy, fx - 1.0, fy);
    let n01 = dot(cx, cy + 1, fx, fy - 1.0);
    let n11 = dot(cx + 1, cy + 1, fx - 1.0, fy - 1.0);
    let (u, v) = (fade(fx), fade(fy));
    lerp(lerp(n00, n10, u), lerp(n01, n11, u), v) * 1.414
}

/// Fractal Brownian motion over `octaves` octaves, roughly in `[-1, 1]`.
#[must_use]
pub fn fbm(seed: WorldSeed, channel: Channel, x: f32, y: f32, octaves: u32) -> f32 {
    let mut sum = 0.0;
    let mut amp = 1.0;
    let mut norm = 0.0;
    let mut freq = 1.0;
    for o in 0..octaves {
        sum += amp * perlin(seed, channel.0 + o, x * freq, y * freq);
        norm += amp;
        amp *= 0.5;
        freq *= 2.0;
    }
    sum / norm
}

/// Ridged multifractal in `[0, 1]` — sharp crests for mountain chains.
#[must_use]
pub fn ridged(seed: WorldSeed, channel: Channel, x: f32, y: f32, octaves: u32) -> f32 {
    let mut sum = 0.0;
    let mut amp = 1.0;
    let mut norm = 0.0;
    let mut freq = 1.0;
    for o in 0..octaves {
        let r = 1.0 - perlin(seed, channel.0 + o, x * freq, y * freq).abs();
        sum += amp * r * r;
        norm += amp;
        amp *= 0.5;
        freq *= 2.0;
    }
    sum / norm
}

/// fBm sampled through a domain warp — organic, Earth-like shapes.
#[must_use]
pub fn warped_fbm(
    seed: WorldSeed,
    base: Channel,
    warp: Channel,
    x: f32,
    y: f32,
    octaves: u32,
    strength: f32,
) -> f32 {
    let wx = fbm(seed, Channel(warp.0), x, y, 3);
    let wy = fbm(seed, Channel(warp.0 + 8), x, y, 3);
    fbm(seed, base, x + strength * wx, y + strength * wy, octaves)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // Exact float equality is the point here: same inputs, same bits.
    #[allow(clippy::float_cmp)]
    fn noise_is_deterministic_and_bounded() {
        let seed = WorldSeed(9);
        for sample in 0..500 {
            let px = sample as f32 * 0.173;
            let py = sample as f32 * 0.089;
            let first = fbm(seed, Channel(0), px, py, 5);
            let second = fbm(seed, Channel(0), px, py, 5);
            assert!(
                first == second,
                "identical inputs must produce identical bits"
            );
            assert!(first.abs() <= 1.5);
            let ridge = ridged(seed, Channel(32), px, py, 3);
            assert!((0.0..=1.0).contains(&ridge));
        }
    }
}
