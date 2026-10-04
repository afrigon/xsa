// Starts and ends slowly: a smoothstep of `progress` in 0..1.
pub fn ease_in_out(progress: f32) -> f32 {
    let progress = progress.clamp(0.0, 1.0);
    progress * progress * (3.0 - 2.0 * progress)
}
