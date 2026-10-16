use xsa_units::TimeRate;
use xui::{Context, EdgeInsets, Text, View};

use super::{CameraReading, ClockReading, SurfaceReading};
use crate::camera::CameraMode;
use crate::renderer::FrameStatistics;
use crate::ui::ThemedView;

const PADDING: f32 = 8.0;
const METERS_PER_KILOMETER: f64 = 1000.0;
// Below this magnitude, distances and speeds read in meters.
const KILOMETER_THRESHOLD_METERS: f64 = 10_000.0;
const MISSING: &str = "–";
const ANGLE_PRECISION: f64 = 1000.0;
const FULL_TURN_DEGREES: f64 = 360.0;

pub struct DebugOverlayView {
    pub frames_per_second: Option<f64>,
    pub gpu_milliseconds: Option<f64>,
    pub statistics: FrameStatistics,
    pub camera: Option<CameraReading>,
    pub surface: Option<SurfaceReading>,
    pub clock: Option<ClockReading>,
}

impl DebugOverlayView {
    fn performance(&self) -> String {
        let frames_per_second = self
            .frames_per_second
            .map_or_else(|| MISSING.to_string(), |rate| format!("{rate:.0}"));
        let gpu_milliseconds = self
            .gpu_milliseconds
            .map_or_else(|| MISSING.to_string(), |milliseconds| format!("{milliseconds:.2}"));
        let depth = self
            .statistics
            .deepest_depth
            .map_or_else(String::new, |depth| format!(" · depth {depth}"));

        format!(
            "FPS: {frames_per_second} · GPU {gpu_milliseconds} ms\n{} faces · {} tiles{depth}",
            self.statistics.triangles, self.statistics.tiles
        )
    }

    fn camera(&self) -> String {
        let Some(camera) = &self.camera else {
            return format!("camera: {MISSING}");
        };
        let mode = match camera.mode {
            CameraMode::Orbit => "orbit".to_string(),
            CameraMode::Debug => format!("debug · {}/s", distance(camera.speed.unwrap_or_default())),
        };
        format!(
            "camera: {mode}\nrotation: yaw {} · pitch {} · roll {}",
            angle(camera.attitude.yaw),
            angle(camera.attitude.pitch),
            angle(camera.attitude.roll)
        )
    }

    fn surface(&self) -> String {
        let Some(surface) = &self.surface else {
            return format!("target: {MISSING}");
        };
        let above_ground = surface.above_ground.map_or_else(|| MISSING.to_string(), distance);
        format!(
            "target: {}\nlat: {} · lon: {}\naltitude: {above_ground} ground · {} sea\n\
             rotation: yaw {} · pitch {} · roll {}",
            surface.body,
            angle(surface.latitude),
            angle(surface.longitude),
            distance(surface.above_sea),
            heading(surface.attitude.yaw),
            angle(surface.attitude.pitch),
            angle(surface.attitude.roll)
        )
    }

    fn clock(&self) -> String {
        let Some(clock) = &self.clock else {
            return format!("date: {MISSING}\ntime: {MISSING}\nrate: {MISSING}");
        };
        // The simulation time formats as `YYYY-MM-DDTHH:MM:SS.mmmZ`.
        let timestamp = clock.time.to_string();
        let (date, time) = timestamp.split_once('T').unwrap_or((&timestamp, MISSING));

        let time_override = clock
            .time_flow
            .time_override
            .map_or_else(String::new, |rate| format!(" (override: {})", Self::rate(rate)));

        format!(
            "date: {date}\ntime: {} UTC\nrate: {}{time_override}",
            time.trim_end_matches('Z'),
            Self::rate(clock.time_flow.rate)
        )
    }

    fn rate(rate: TimeRate) -> String {
        if rate.is_paused() {
            "paused".to_string()
        } else {
            format!("{}×", rate.multiplier)
        }
    }
}

// Degrees to a thousandth; rounding first, then adding zero, keeps a value that rounds to zero from printing as
// -0.000.
fn angle(radians: f64) -> String {
    let rounded = (radians.to_degrees() * ANGLE_PRECISION).round() / ANGLE_PRECISION + 0.0;
    format!("{rounded:.3}")
}

// As `camera tp` takes it: a compass heading, clockwise from north, wrapped after rounding so it never reads 360.
fn heading(yaw: f64) -> String {
    let rounded = (-yaw.to_degrees() * ANGLE_PRECISION).round() / ANGLE_PRECISION;
    format!("{:.3}", rounded.rem_euclid(FULL_TURN_DEGREES) + 0.0)
}

fn distance(meters: f64) -> String {
    if meters.abs() < KILOMETER_THRESHOLD_METERS {
        format!("{meters:.1} m")
    } else {
        format!("{:.2} km", meters / METERS_PER_KILOMETER)
    }
}

impl View for DebugOverlayView {
    fn body(&self, _context: &Context) -> impl View {
        Text::new(format!(
            "{}\n\n{}\n\n{}\n\n{}",
            self.performance(),
            self.camera(),
            self.surface(),
            self.clock()
        ))
        .text_style("telemetry")
        .padding(EdgeInsets::all(PADDING))
    }
}
