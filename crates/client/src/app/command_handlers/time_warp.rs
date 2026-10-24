use anyhow::Context;
use xsa_commands::command::{TimeWarpCommand, TimeWarpRequestCommand};
use xsa_units::{TimeWarp, WarpPace};

use crate::app::App;
use crate::app::client_command_handler::ClientCommandHandler;
use crate::app::command_progress::CommandProgress;

// An eased warp lasts this long per doubling of the simulation time it covers, from this much simulation time up: a
// minute takes a quarter second, a day about 2.6 s, a year about 4.8 s.
const SECONDS_PER_DOUBLING: f64 = 0.25;
const SHORTEST_SPAN_SECONDS: f64 = 60.0;

impl ClientCommandHandler for TimeWarpCommand {
    fn start(self, app: &mut App) -> CommandProgress {
        let plan = || -> anyhow::Result<TimeWarpRequestCommand> {
            let time_of_day = self.time_of_day.context("name a time of day, or --cancel")?;
            let world = app.world.as_ref().context("not joined to a simulation yet")?;
            let cameras = app.cameras.as_ref().context("the cameras are not ready")?;
            let body = cameras.anchor().context("the camera is around no body")?.body();
            let pose = cameras.controls_pose();
            let spot = world.position_reading(body, pose.position, pose.orientation);
            let star = world.star_for(body, self.star.as_deref())?;
            let now = world.time();
            let until = world
                .daylight(body, star)
                .next(time_of_day, spot.latitude, spot.longitude, now)?;
            let span = until.seconds_since(now);
            let scale = f64::from(app.config.animation.time_duration_scale());
            let pace = if self.instant || scale == 0.0 {
                WarpPace::Instant
            } else {
                WarpPace::Eased {
                    seconds: SECONDS_PER_DOUBLING * (1.0 + span / SHORTEST_SPAN_SECONDS).log2() * scale,
                }
            };
            let id = &world.body(body).id;
            let description = match pace {
                WarpPace::Instant => format!("time: {time_of_day} on {id}, {until}"),
                WarpPace::Eased { .. } => format!("time: warping to {time_of_day} on {id}, {until}"),
            };

            Ok(TimeWarpRequestCommand {
                warp: TimeWarp { until, pace },
                description,
            })
        };

        match plan() {
            Ok(request) => CommandProgress::Forward(Box::new(request)),
            Err(err) => CommandProgress::Done(Err(err)),
        }
    }
}
