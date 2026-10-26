use animation::{EasedProgress, Easing};

// How long a change of field of view takes, before the camera's animation scale.
const CHANGE_SECONDS: f64 = 0.25;

// The camera's field of view moving toward its configured one, as the tangent of half its horizontal angle. It eases in
// the tangent's logarithm, so a zoom magnifies at an even pace.
pub struct FieldOfView {
    from: f64,
    to: f64,
    progress: EasedProgress,
}

impl FieldOfView {
    pub fn new(tangent: f64) -> FieldOfView {
        FieldOfView {
            from: tangent,
            to: tangent,
            progress: EasedProgress::new(0.0, Easing::Linear),
        }
    }

    pub fn aim(&mut self, tangent: f64, duration_scale: f64) {
        if tangent == self.to {
            return;
        }

        self.from = self.tangent();
        self.to = tangent;
        self.progress = EasedProgress::new(CHANGE_SECONDS * duration_scale, Easing::EASE_IN_OUT);
    }

    pub fn advance(&mut self, seconds: f64) {
        self.progress.advance(seconds);
    }

    pub fn tangent(&self) -> f64 {
        (self.from.ln() + (self.to.ln() - self.from.ln()) * self.progress.value()).exp()
    }

    // Radians.
    pub fn horizontal_angle(&self) -> f32 {
        (2.0 * self.tangent().atan()) as f32
    }
}
