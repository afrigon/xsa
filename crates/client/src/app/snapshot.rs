use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, ensure};
use image::{ImageFormat, RgbaImage, imageops};
use xsa_commands::value::Region;
use xsa_units::SimulationTime;

use crate::renderer::CapturedImage;

const DIRECTORY_NAME: &str = "xsa";
const FILE_PREFIX: &str = "xsa-";
const FILE_EXTENSION: &str = "png";
const PICTURES_VARIABLE: &str = "XDG_PICTURES_DIR";
const USER_DIRS_FILE: &str = "user-dirs.dirs";
const DEFAULT_PICTURES: &str = "Pictures";

pub(super) struct Snapshot {
    image: RgbaImage,
}

impl Snapshot {
    pub fn new(captured: CapturedImage) -> anyhow::Result<Snapshot> {
        let image = RgbaImage::from_raw(captured.width, captured.height, captured.rgba)
            .context("the captured image does not match its size")?;

        Ok(Snapshot { image })
    }

    pub fn crop(self, region: Region) -> anyhow::Result<Snapshot> {
        let fits = u64::from(region.x) + u64::from(region.width) <= u64::from(self.image.width())
            && u64::from(region.y) + u64::from(region.height) <= u64::from(self.image.height());
        ensure!(
            fits,
            "the region does not fit in the {}x{} frame",
            self.image.width(),
            self.image.height()
        );
        let image = imageops::crop_imm(&self.image, region.x, region.y, region.width, region.height).to_image();

        Ok(Snapshot { image })
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<String> {
        if let Some(directory) = path.parent().filter(|directory| !directory.as_os_str().is_empty()) {
            fs::create_dir_all(directory).with_context(|| format!("creating {}", directory.display()))?;
        }

        self.image
            .save_with_format(path, ImageFormat::Png)
            .with_context(|| format!("writing {}", path.display()))?;

        Ok(format!(
            "saved {} ({}x{})",
            path.display(),
            self.image.width(),
            self.image.height()
        ))
    }

    // Named after the wall-clock time, with the colons Windows forbids replaced.
    pub fn default_path() -> anyhow::Result<PathBuf> {
        let time = SimulationTime::now()?.to_string();
        let stamp = time.trim_end_matches('Z').replace(':', "-");
        let file = format!("{FILE_PREFIX}{stamp}.{FILE_EXTENSION}");

        Ok(Snapshot::pictures_directory()?.join(DIRECTORY_NAME).join(file))
    }

    // XDG_PICTURES_DIR is usually only in user-dirs.dirs, written by xdg-user-dirs, not in the environment.
    fn pictures_directory() -> anyhow::Result<PathBuf> {
        if let Some(directory) = env::var_os(PICTURES_VARIABLE).filter(|value| !value.is_empty()) {
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
        let from_user_dirs = fs::read_to_string(config.join(USER_DIRS_FILE))
            .ok()
            .and_then(|text| Snapshot::user_dirs_pictures(&text, &home));

        Ok(from_user_dirs.unwrap_or_else(|| home.join(DEFAULT_PICTURES)))
    }

    // A line looks like XDG_PICTURES_DIR="$HOME/Pictures".
    fn user_dirs_pictures(text: &str, home: &Path) -> Option<PathBuf> {
        let line = text
            .lines()
            .find_map(|line| line.trim().strip_prefix(PICTURES_VARIABLE)?.strip_prefix('='))?;
        let value = line.trim().trim_matches('"');
        let path = match value.strip_prefix("$HOME") {
            Some(rest) => home.join(rest.trim_start_matches('/')),
            None => PathBuf::from(value),
        };

        Some(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pictures_directory_comes_from_user_dirs() {
        let home = Path::new("/home/someone");
        let text = "# comment\nXDG_DESKTOP_DIR=\"$HOME/Desktop\"\nXDG_PICTURES_DIR=\"$HOME/Images\"\n";
        assert_eq!(
            Snapshot::user_dirs_pictures(text, home),
            Some(PathBuf::from("/home/someone/Images"))
        );
        assert_eq!(
            Snapshot::user_dirs_pictures("XDG_PICTURES_DIR=\"/data/pics\"", home),
            Some(PathBuf::from("/data/pics"))
        );
        assert_eq!(Snapshot::user_dirs_pictures("", home), None);
    }

    #[test]
    fn crops_must_fit_in_the_frame() {
        let snapshot = || Snapshot {
            image: RgbaImage::new(100, 50),
        };
        let inside = Region {
            x: 10,
            y: 10,
            width: 90,
            height: 40,
        };
        let outside = Region {
            x: 10,
            y: 10,
            width: 91,
            height: 40,
        };
        assert_eq!(snapshot().crop(inside).unwrap().image.dimensions(), (90, 40));
        assert!(snapshot().crop(outside).is_err());
    }
}
