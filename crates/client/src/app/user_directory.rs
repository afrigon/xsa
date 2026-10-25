use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context;
use xsa_units::SimulationTime;

const APP_DIRECTORY: &str = "xsa";
const FILE_PREFIX: &str = "xsa-";
const USER_DIRS_FILE: &str = "user-dirs.dirs";

// A folder of the user's that the game writes into, as the XDG user directories name it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum UserDirectory {
    Pictures,
    Videos,
}

impl UserDirectory {
    // A new file in the game's folder of this directory, named after the wall-clock time.
    pub fn new_file(self, extension: &str) -> anyhow::Result<PathBuf> {
        let time = SimulationTime::now()?.to_string();
        let stamp = time.trim_end_matches('Z').replace(':', "-");

        Ok(self
            .path()?
            .join(APP_DIRECTORY)
            .join(format!("{FILE_PREFIX}{stamp}.{extension}")))
    }

    // The variable is usually only in user-dirs.dirs, written by xdg-user-dirs, not in the environment.
    fn path(self) -> anyhow::Result<PathBuf> {
        if let Some(directory) = env::var_os(self.variable()).filter(|value| !value.is_empty()) {
            return Ok(PathBuf::from(directory));
        }

        let home = PathBuf::from(
            env::var_os("HOME")
                .or_else(|| env::var_os("USERPROFILE"))
                .context("neither HOME nor USERPROFILE is set")?,
        );
        let config = env::var_os("XDG_CONFIG_HOME")
            .filter(|value| !value.is_empty())
            .map_or_else(|| home.join(".config"), PathBuf::from);
        let read_user_dirs = fs::read_to_string(config.join(USER_DIRS_FILE))
            .ok()
            .and_then(|text| self.read_user_dirs(&text, &home));

        Ok(read_user_dirs.unwrap_or_else(|| home.join(self.default_name())))
    }

    // A line looks like XDG_PICTURES_DIR="$HOME/Pictures".
    fn read_user_dirs(self, text: &str, home: &Path) -> Option<PathBuf> {
        let line = text
            .lines()
            .find_map(|line| line.trim().strip_prefix(self.variable())?.strip_prefix('='))?;
        let value = line.trim().trim_matches('"');
        let path = match value.strip_prefix("$HOME") {
            Some(rest) => home.join(rest.trim_start_matches('/')),
            None => PathBuf::from(value),
        };

        Some(path)
    }

    fn variable(self) -> &'static str {
        match self {
            UserDirectory::Pictures => "XDG_PICTURES_DIR",
            UserDirectory::Videos => "XDG_VIDEOS_DIR",
        }
    }

    fn default_name(self) -> &'static str {
        match self {
            UserDirectory::Pictures => "Pictures",
            UserDirectory::Videos => "Videos",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directories_come_read_user_dirs() {
        let home = Path::new("/home/someone");
        let text = "# comment\nXDG_DESKTOP_DIR=\"$HOME/Desktop\"\nXDG_PICTURES_DIR=\"$HOME/Images\"\n";
        assert_eq!(
            UserDirectory::Pictures.read_user_dirs(text, home),
            Some(PathBuf::from("/home/someone/Images"))
        );
        assert_eq!(
            UserDirectory::Pictures.read_user_dirs("XDG_PICTURES_DIR=\"/data/pics\"", home),
            Some(PathBuf::from("/data/pics"))
        );
        assert_eq!(UserDirectory::Videos.read_user_dirs(text, home), None);
    }
}
