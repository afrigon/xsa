use std::collections::HashSet;

use anyhow::{Context, ensure};
use glam::{Mat3, Mat4, Vec3, Vec4};
use gltf::buffer::Source;
use gltf::mesh::Mode;

use super::{BoundingBox, MeshData, MeshVertex, Submesh, TangentFrame};
use crate::utils::HalfFloat;

const GLB_MAGIC: &[u8; 4] = b"glTF";
const UV_SET: u32 = 0;
// The rotation from glTF's frame to the model frame: (x, y, z) → (x, −z, y).
const TO_MODEL_FRAME: Mat3 = Mat3::from_cols(Vec3::X, Vec3::Z, Vec3::NEG_Y);

/// Reads binary glTF 2.0 files whose every mesh node draws one triangle list, with normals, tangents and one set of
/// texture coordinates. Converts glTF's frame (+Y up, +Z forward) to the model frame, Blender's: +Z up, −Y forward.
pub struct GlbLoader;

impl GlbLoader {
    pub const EXTENSION: &'static str = "glb";

    pub fn load(bytes: &[u8]) -> anyhow::Result<MeshData> {
        ensure!(bytes.starts_with(GLB_MAGIC), "not a binary glTF file");

        let gltf = gltf::Gltf::from_slice(bytes)?;
        let blob = gltf.blob.as_deref();
        let document = &gltf.document;
        let scene = document
            .default_scene()
            .or_else(|| document.scenes().next())
            .context("the file has no scene")?;
        let mut builder = MeshBuilder {
            blob,
            vertices: Vec::new(),
            indices: Vec::new(),
            submeshes: Vec::new(),
            nodes: HashSet::new(),
        };

        for node in scene.nodes() {
            builder.add_node(&node, Mat4::IDENTITY)?;
        }

        ensure!(!builder.submeshes.is_empty(), "the file has no mesh");

        Ok(builder.finish())
    }
}

struct MeshBuilder<'a> {
    blob: Option<&'a [u8]>,
    vertices: Vec<MeshVertex>,
    indices: Vec<u32>,
    submeshes: Vec<Submesh>,
    nodes: HashSet<String>,
}

impl MeshBuilder<'_> {
    // Every index a submesh draws, placed by its node.
    fn finish(self) -> MeshData {
        let placed = self.submeshes.iter().flat_map(|submesh| {
            let start = submesh.first_index as usize;
            self.indices[start..start + submesh.index_count as usize]
                .iter()
                .map(|&index| {
                    submesh
                        .transform
                        .transform_point3(Vec3::from_array(self.vertices[index as usize].position))
                })
        });
        let bounding_box = BoundingBox::around(placed);

        MeshData {
            vertices: self.vertices,
            indices: self.indices,
            submeshes: self.submeshes,
            bounding_box,
        }
    }

    fn add_node(&mut self, node: &gltf::Node, parent: Mat4) -> anyhow::Result<()> {
        let transform = parent * Mat4::from_cols_array_2d(&node.transform().matrix());

        if let Some(mesh) = node.mesh() {
            let name = node.name().context("a mesh node has no name")?;
            self.add_mesh(name, &mesh, transform, node.skin().is_some())
                .with_context(|| format!("node {name}"))?;
        }

        for child in node.children() {
            self.add_node(&child, transform)?;
        }

        Ok(())
    }

    fn add_mesh(&mut self, name: &str, mesh: &gltf::Mesh, transform: Mat4, skinned: bool) -> anyhow::Result<()> {
        ensure!(!skinned, "skinned meshes are not supported");
        ensure!(
            self.nodes.insert(name.to_string()),
            "another mesh node has the same name"
        );

        let mut primitives = mesh.primitives();
        let primitive = primitives.next().context("its mesh has no primitive")?;
        ensure!(
            primitives.next().is_none(),
            "its mesh has several primitives: export with Materials set to No export"
        );
        ensure!(primitive.mode() == Mode::Triangles, "its mesh is not a triangle list");
        ensure!(
            primitive.morph_targets().next().is_none(),
            "morph targets are not supported"
        );

        let blob = self.blob;
        let reader = primitive.reader(|buffer| match buffer.source() {
            Source::Bin => blob,
            Source::Uri(_) => None,
        });
        let positions: Vec<[f32; 3]> = reader.read_positions().context("its mesh has no positions")?.collect();
        let normals: Vec<[f32; 3]> = reader.read_normals().context("its mesh has no normals")?.collect();
        let tangents: Vec<[f32; 4]> = reader
            .read_tangents()
            .context("its mesh has no tangents: export with Tangents on")?
            .collect();
        let uvs: Vec<[f32; 2]> = reader
            .read_tex_coords(UV_SET)
            .context("its mesh has no texture coordinates")?
            .into_f32()
            .collect();
        ensure!(
            normals.len() == positions.len() && tangents.len() == positions.len() && uvs.len() == positions.len(),
            "its mesh's attributes have different vertex counts"
        );

        let base_vertex = u32::try_from(self.vertices.len()).context("the file has too many vertices")?;

        for (((position, normal), tangent), uv) in positions.iter().zip(&normals).zip(&tangents).zip(&uvs) {
            let tangent = Vec4::from(*tangent);
            let tangent = (TO_MODEL_FRAME * tangent.truncate()).extend(tangent.w);

            self.vertices.push(MeshVertex {
                position: (TO_MODEL_FRAME * Vec3::from(*position)).to_array(),
                tangent_frame: TangentFrame::new(TO_MODEL_FRAME * Vec3::from(*normal), tangent)?,
                uv: uv.map(HalfFloat::from_f32),
            });
        }

        let first_index = u32::try_from(self.indices.len()).context("the file has too many indices")?;

        match reader.read_indices() {
            Some(indices) => {
                for index in indices.into_u32() {
                    ensure!(
                        (index as usize) < positions.len(),
                        "its mesh has an index past its vertices"
                    );
                    self.indices.push(base_vertex + index);
                }
            }
            None => self
                .indices
                .extend((0..positions.len() as u32).map(|index| base_vertex + index)),
        }

        let index_count = self.indices.len() as u32 - first_index;
        ensure!(
            index_count.is_multiple_of(3),
            "its mesh's index count is not a multiple of three"
        );

        let frame = Mat4::from_mat3(TO_MODEL_FRAME);
        self.submeshes.push(Submesh {
            node: name.to_string(),
            first_index,
            index_count,
            transform: frame * transform * frame.inverse(),
        });

        Ok(())
    }
}
