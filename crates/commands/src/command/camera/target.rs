use anyhow::bail;
use usage::Args;
use xsa_core::simulation::BodyCategory;

use super::TargetSelection;
use crate::command::{ClientCommand, Routable, Route};
use crate::completion::{complete_body, complete_body_category};
use crate::value::Altitude;

#[derive(Args)]
#[usage(group("target", required))]
pub struct CameraTargetCommand {
    #[usage(complete = complete_body, group = "target", help = "The body to orbit, by id")]
    pub body: Option<String>,
    #[usage(long, group = "target", help = "Orbit the body after the one the camera is about")]
    pub next: bool,
    #[usage(long, group = "target", help = "Orbit the body before the one the camera is about")]
    pub previous: bool,
    #[usage(long, group = "target", help = "Orbit the body the one the camera is about orbits")]
    pub parent: bool,
    #[usage(
        long,
        complete = complete_body_category,
        conflicts("body", "--parent"),
        help = "With --next or --previous, only stop at bodies of this category, e.g. moon"
    )]
    pub category: Option<BodyCategory>,
    #[usage(
        long,
        allow_negative_numbers,
        help = "Feet above the ground below the camera, e.g. 100m (m, km, Mm, Gm; a bare number is km)"
    )]
    pub altitude: Option<Altitude>,
    #[usage(long, allow_negative_numbers, help = "Pitch in degrees; positive looks up")]
    pub pitch: Option<f64>,
    #[usage(long, allow_negative_numbers, help = "Yaw in degrees around the ecliptic north pole")]
    pub yaw: Option<f64>,
}

impl CameraTargetCommand {
    pub fn selection(&self) -> anyhow::Result<TargetSelection> {
        let category = self.category;

        match (&self.body, self.next, self.previous, self.parent) {
            (Some(id), false, false, false) if category.is_none() => Ok(TargetSelection::Body { id: id.clone() }),
            (None, true, false, false) => Ok(TargetSelection::Next { category }),
            (None, false, true, false) => Ok(TargetSelection::Previous { category }),
            (None, false, false, true) if category.is_none() => Ok(TargetSelection::Parent),
            _ => bail!(
                "name one body, or one of --next, --previous or --parent; --category goes with --next or --previous"
            ),
        }
    }
}

impl Routable for CameraTargetCommand {
    fn route(self) -> Route {
        Route::Client(ClientCommand::CameraTarget(self))
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use super::*;
    use crate::command::{CameraAction, Command, CommandLine};

    fn parse(line: &str) -> Option<CameraTargetCommand> {
        let words: Vec<&OsStr> = line.split_whitespace().map(OsStr::new).collect();
        let Ok(CommandLine {
            command: Command::Camera(camera),
        }) = CommandLine::parse_from(&words)
        else {
            return None;
        };
        let CameraAction::Target(target) = camera.action else {
            return None;
        };

        Some(target)
    }

    #[test]
    fn a_bare_body_id_selects_that_body() {
        let selection = parse("camera target luna").unwrap().selection().unwrap();
        assert_eq!(selection, TargetSelection::Body { id: "luna".to_string() });
    }

    #[test]
    fn a_step_takes_its_category() {
        let selection = parse("camera target --next --category moon")
            .unwrap()
            .selection()
            .unwrap();
        assert_eq!(
            selection,
            TargetSelection::Next {
                category: Some(BodyCategory::Moon)
            }
        );
    }

    #[test]
    fn exactly_one_target_is_required() {
        assert!(parse("camera target").is_none());
        assert!(parse("camera target --next --parent").is_none());
        assert!(parse("camera target luna --previous").is_none());
    }

    #[test]
    fn a_category_goes_only_with_a_step() {
        assert!(parse("camera target luna --category moon").is_none());
        assert!(parse("camera target --parent --category moon").is_none());
    }
}
