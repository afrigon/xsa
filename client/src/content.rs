use std::fs;

use anyhow::Context;
use xsa_packs::{Id, PackStack};

use crate::renderer::{
    BLOOM_DOWNSAMPLE_SHADER_PATH, BLOOM_UPSAMPLE_SHADER_PATH, HISTOGRAM_SHADER_PATH, Shader, ShaderBinaries,
    TONEMAP_SHADER_PATH,
};

const BASE_NAMESPACE: &str = "base";
const SHADERS: &str = "shaders";
const SHADER_EXTENSION: &str = "spv";

pub fn load_shaders(stack: &PackStack) -> anyhow::Result<ShaderBinaries> {
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
    })
}
