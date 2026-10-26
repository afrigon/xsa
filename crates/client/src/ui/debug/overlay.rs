use xsa_units::TimeRate;
use xui::{Context, EdgeInsets, Text, View};

use super::{CameraReading, ClockReading, PositionReading};
use crate::BuildIdentity;
use crate::renderer::{FrameStatistics, TerrainLayer};
use crate::ui::ThemedView;

const PADDING: f32 = 8.0;
const METERS_PER_KILOMETER: f64 = 1000.0;
// Below this magnitude, distances and speeds read in meters.
const KILOMETER_THRESHOLD_METERS: f64 = 10_000.0;
const MISSING: &str = "–";
const ANGLE_PRECISION: f64 = 1000.0;
const FULL_TURN_DEGREES: f64 = 360.0;
const BYTES_PER_MEBIBYTE: f64 = 1024.0 * 1024.0;
const MILLISECONDS_PER_SECOND: f64 = 1000.0;

pub struct DebugOverlayView {
    pub frames_per_second: Option<f64>,
    pub gpu_milliseconds: Option<f64>,
    pub build: BuildIdentity,
    pub statistics: FrameStatistics,
    pub camera: Option<CameraReading>,
    pub position: Option<PositionReading>,
    pub clock: Option<ClockReading>,
}

impl DebugOverlayView {
    fn performance(&self) -> String {
        let frames_per_second = self
            .frames_per_second
            .map_or_else(|| MISSING.to_string(), |rate| format!("{rate:.0}"));
        let frame_milliseconds = self.frames_per_second.map_or_else(
            || MISSING.to_string(),
            |rate| format!("{:.2}", MILLISECONDS_PER_SECOND / rate),
        );
        let gpu_milliseconds = self
            .gpu_milliseconds
            .map_or_else(|| MISSING.to_string(), |milliseconds| format!("{milliseconds:.2}"));
        let depth = self
            .statistics
            .deepest_depth
            .map_or_else(String::new, |depth| format!(" · depth {depth}"));

        format!(
            "FPS: {frames_per_second} · {frame_milliseconds} ms · GPU {gpu_milliseconds} ms\n\
             build: {}\n\
             {} faces · {} tiles{depth}",
            self.build, self.statistics.triangles, self.statistics.tiles
        )
    }

    // Each kind of terrain map over every terrain: the memory its resident levels hold, and how many maps still load
    // toward the levels the camera needs.
    fn terrain_maps(&self) -> String {
        let maps = &self.statistics.terrain_maps;
        let kinds = std::iter::once(("heights", &maps.heights))
            .chain(TerrainLayer::ALL.map(|layer| (layer.name(), &maps.layers[layer.index()])));

        kinds
            .map(|(name, map)| {
                format!(
                    "{name}: {:.1} MiB video · {:.1} MiB RAM · {} loading",
                    map.video_bytes as f64 / BYTES_PER_MEBIBYTE,
                    map.ram_bytes as f64 / BYTES_PER_MEBIBYTE,
                    map.loading
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn camera(&self) -> String {
        let Some(camera) = &self.camera else {
            return format!("camera: {MISSING}");
        };
        let mode = match camera.speed {
            Some(speed) => format!("{} · {}/s", camera.controls, distance(speed)),
            None => camera.controls.to_string(),
        };
        format!(
            "camera: {mode}\nrotation: yaw {} · pitch {} · roll {}",
            angle(camera.attitude.yaw),
            angle(camera.attitude.pitch),
            angle(camera.attitude.roll)
        )
    }

    fn position(&self) -> String {
        let Some(position) = &self.position else {
            return format!("target: {MISSING}");
        };
        let above_ground = position.above_ground.map_or_else(|| MISSING.to_string(), distance);
        format!(
            "target: {}\nlat: {} · lon: {}\naltitude: {above_ground} ground · {} sea\n\
             rotation: yaw {} · pitch {} · roll {}",
            position.body,
            angle(position.latitude),
            angle(position.longitude),
            distance(position.above_sea),
            heading(position.attitude.yaw),
            angle(position.attitude.pitch),
            angle(position.attitude.roll)
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
        let warp = clock.time_flow.warp.map_or_else(String::new, |warp| {
            format!(" (warp: {:.0}× to {})", clock.rate.multiplier, warp.until)
        });

        format!(
            "date: {date}\ntime: {} UTC\nrate: {}{time_override}{warp}",
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
            "{}\n\n{}\n\n{}\n\n{}\n\n{}",
            self.performance(),
            self.terrain_maps(),
            self.camera(),
            self.position(),
            self.clock()
        ))
        .text_style("telemetry")
        .padding(EdgeInsets::all(PADDING))
    }
}
