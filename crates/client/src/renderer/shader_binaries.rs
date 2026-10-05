use std::fs;

use anyhow::Context;
use xsa_packs::{Id, PackStack};

use super::Shader;

const BASE_NAMESPACE: &str = "base";
const SHADERS: &str = "shaders";
const SHADER_EXTENSION: &str = "spv";
const TONEMAP_SHADER_PATH: &str = "tonemap";
const HISTOGRAM_SHADER_PATH: &str = "histogram";
const BLOOM_DOWNSAMPLE_SHADER_PATH: &str = "bloom-downsample";
const BLOOM_UPSAMPLE_SHADER_PATH: &str = "bloom-upsample";
const TAA_SHADER_PATH: &str = "taa";

pub struct ShaderBinaries {
    pub shaders: Vec<Vec<u8>>,
    pub tonemap: Vec<u8>,
    pub histogram: Vec<u8>,
    pub bloom_downsample: Vec<u8>,
    pub bloom_upsample: Vec<u8>,
    pub taa: Vec<u8>,
}

impl ShaderBinaries {
    pub fn load(stack: &PackStack) -> anyhow::Result<ShaderBinaries> {
        let read = |path: &str| {
            let id = Id {
                namespace: BASE_NAMESPACE.to_string(),
                path: path.to_string(),
            };
            let file = stack
                .resource(SHADERS, &id, SHADER_EXTENSION)
                .context("shaders are missing: run `mise run shaders`")?;

            fs::read(file).with_context(|| format!("reading {}", file.display()))
        };

        Ok(ShaderBinaries {
            shaders: Shader::ALL
                .iter()
                .map(|shader| read(shader.path()))
                .collect::<anyhow::Result<_>>()?,
            tonemap: read(TONEMAP_SHADER_PATH)?,
            histogram: read(HISTOGRAM_SHADER_PATH)?,
            bloom_downsample: read(BLOOM_DOWNSAMPLE_SHADER_PATH)?,
            bloom_upsample: read(BLOOM_UPSAMPLE_SHADER_PATH)?,
            taa: read(TAA_SHADER_PATH)?,
        })
    }
}
