use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, ensure};
use image::{ImageFormat, RgbaImage, imageops};
use xsa_commands::value::Region;

use super::user_directory::UserDirectory;
use crate::renderer::CapturedImage;

const FILE_EXTENSION: &str = "png";

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
        UserDirectory::Pictures.new_file(FILE_EXTENSION)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
