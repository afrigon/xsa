use std::collections::HashMap;
use std::fs;

use anyhow::Context;
use xsa_packs::{Id, PackStack};

use super::Shader;

const BASE_NAMESPACE: &str = "base";
const SHADERS: &str = "shaders";
const SHADER_EXTENSION: &str = "spv";
const TONEMAP_SHADER_PATH: &str = "post/tonemap";
const HISTOGRAM_SHADER_PATH: &str = "post/exposure/histogram";
const BLOOM_DOWNSAMPLE_SHADER_PATH: &str = "post/bloom/downsample";
const BLOOM_UPSAMPLE_SHADER_PATH: &str = "post/bloom/upsample";
const TAA_SHADER_PATH: &str = "post/taa";
const LENS_LIGHT_SHADER_PATH: &str = "post/lens/light";
const LENS_GHOSTS_SHADER_PATH: &str = "post/lens/ghosts";

pub struct ShaderBinaries {
    // Each terrain program, by the shader it compiles.
    pub(in crate::renderer) forward: HashMap<Shader, Vec<u8>>,
    pub skybox: Vec<u8>,
    pub tonemap: Vec<u8>,
    pub histogram: Vec<u8>,
    pub bloom_downsample: Vec<u8>,
    pub bloom_upsample: Vec<u8>,
    pub taa: Vec<u8>,
    pub lens_light: Vec<u8>,
    pub lens_ghosts: Vec<u8>,
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
            // The skybox is drawn by its own pass.
            forward: Shader::ALL
                .iter()
                .filter(|shader| **shader != Shader::Skybox)
                .map(|shader| Ok((*shader, read(shader.path())?)))
                .collect::<anyhow::Result<_>>()?,
            skybox: read(Shader::Skybox.path())?,
            tonemap: read(TONEMAP_SHADER_PATH)?,
            histogram: read(HISTOGRAM_SHADER_PATH)?,
            bloom_downsample: read(BLOOM_DOWNSAMPLE_SHADER_PATH)?,
            bloom_upsample: read(BLOOM_UPSAMPLE_SHADER_PATH)?,
            taa: read(TAA_SHADER_PATH)?,
            lens_light: read(LENS_LIGHT_SHADER_PATH)?,
            lens_ghosts: read(LENS_GHOSTS_SHADER_PATH)?,
        })
    }
}
