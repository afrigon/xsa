use std::fmt;
use std::str::FromStr;

use anyhow::ensure;

// A camera, by name; `main` is the one the window shows.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CameraId {
    pub name: String,
}

impl CameraId {
    pub const MAIN: &str = "main";

    pub fn main() -> CameraId {
        CameraId {
            name: CameraId::MAIN.to_string(),
        }
    }

    pub fn is_main(&self) -> bool {
        self.name == CameraId::MAIN
    }
}

impl FromStr for CameraId {
    type Err = anyhow::Error;

    fn from_str(text: &str) -> anyhow::Result<CameraId> {
        let kebab = text
            .split('-')
            .all(|word| !word.is_empty() && word.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()));
        ensure!(kebab, "{text:?} is not a camera id like main or chase-cam");

        Ok(CameraId { name: text.to_string() })
    }
}

impl fmt::Display for CameraId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.name)
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use super::*;
    use crate::command::{Command, CommandLine};

    fn camera(line: &str) -> Option<CameraId> {
        let words: Vec<&OsStr> = line.split_whitespace().map(OsStr::new).collect();
        let Ok(CommandLine {
            command: Command::Camera(camera),
        }) = CommandLine::parse_from(&words)
        else {
            return None;
        };

        Some(camera.id)
    }

    #[test]
    fn ids_are_kebab_case() {
        assert_eq!("chase-cam".parse::<CameraId>().unwrap().name, "chase-cam");
        assert!("Chase".parse::<CameraId>().is_err());
        assert!("chase--cam".parse::<CameraId>().is_err());
        assert!("".parse::<CameraId>().is_err());
    }

    #[test]
    fn camera_commands_default_to_the_main_camera() {
        assert_eq!(camera("camera capture"), Some(CameraId::main()));
        assert_eq!(
            camera("camera --id chase-cam capture"),
            Some("chase-cam".parse().unwrap())
        );
    }
}
