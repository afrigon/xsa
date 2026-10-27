use anyhow::bail;
use glam::Vec3;

use super::material_document::MaterialDocument;
use crate::{Id, PackStack};

pub enum MeshMaterialDefinition {
    // Shaded with Hapke 2012, every parameter constant over the mesh: w from the base colour, the phase function's b
    // and c, the macroscopic roughness θ̄ (degrees), the porosity coefficient K and the opposition effects' B_S0, h_S,
    // B_C0 and h_C.
    Hapke {
        base_color: Vec3,
        lobe_width: f32,
        lobe_balance: f32,
        macroscopic_roughness_degrees: f32,
        porosity: f32,
        shadow_hiding_amplitude: f32,
        shadow_hiding_width: f32,
        backscatter_amplitude: f32,
        backscatter_width: f32,
    },
}

impl MeshMaterialDefinition {
    pub fn load(stack: &PackStack, material: &Id) -> anyhow::Result<MeshMaterialDefinition> {
        let document = MaterialDocument::read(stack, material)?;

        document.with_path(MeshMaterialDefinition::parse(&document))
    }

    fn parse(document: &MaterialDocument) -> anyhow::Result<MeshMaterialDefinition> {
        let shader = document.shader()?;

        match shader.as_str() {
            "base:hapke" => Ok(MeshMaterialDefinition::Hapke {
                base_color: document.color("base-color")?,
                lobe_width: document.number("lobe-width")?,
                lobe_balance: document.number("lobe-balance")?,
                macroscopic_roughness_degrees: document.number("macroscopic-roughness")?,
                porosity: document.number("porosity")?,
                shadow_hiding_amplitude: document.number_property("shadow-hiding", "amplitude")?,
                shadow_hiding_width: document.number_property("shadow-hiding", "width")?,
                backscatter_amplitude: document.number_property("backscatter", "amplitude")?,
                backscatter_width: document.number_property("backscatter", "width")?,
            }),
            _ => bail!("shader {shader} does not shade meshes: use base:hapke"),
        }
    }
}
