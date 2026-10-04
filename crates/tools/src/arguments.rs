use anyhow::{Context, bail};

const DEFAULT_EXPOSURE: f32 = 1.0;
const USAGE: &str = "usage: convert-skybox <input.exr> <output.dds> [exposure]";

pub struct Arguments {
    pub input: String,
    pub output: String,
    pub exposure: f32,
}

impl Arguments {
    pub fn parse() -> anyhow::Result<Arguments> {
        let mut arguments = std::env::args().skip(1);
        let Some(input) = arguments.next() else {
            bail!(USAGE);
        };
        let Some(output) = arguments.next() else {
            bail!(USAGE);
        };
        let exposure = match arguments.next() {
            Some(exposure) => exposure.parse().context("parsing the exposure")?,
            None => DEFAULT_EXPOSURE,
        };

        Ok(Arguments {
            input,
            output,
            exposure,
        })
    }
}
