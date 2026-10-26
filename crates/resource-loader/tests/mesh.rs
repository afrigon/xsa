use glam::{Mat4, Vec3, Vec4};
use resource_loader::mesh::{GlbLoader, MeshData, TangentFrame};

const GLB_MAGIC: &[u8; 4] = b"glTF";
const GLB_VERSION: u32 = 2;
const GLB_HEADER_BYTES: usize = 12;
const CHUNK_HEADER_BYTES: usize = 8;
const JSON_CHUNK: &[u8; 4] = b"JSON";
const BINARY_CHUNK: &[u8; 4] = b"BIN\0";
const CHUNK_ALIGNMENT: usize = 4;
const COMPONENT_FLOAT: u32 = 5126;
const COMPONENT_UNSIGNED_INT: u32 = 5125;
const MODE_LINES: u32 = 1;
const MODE_TRIANGLES: u32 = 4;
const TOLERANCE: f32 = 1e-3;

struct Triangle {
    positions: [[f32; 3]; 3],
    normal: [f32; 3],
    tangent: Option<[f32; 4]>,
    mode: u32,
}

impl Triangle {
    fn upright() -> Triangle {
        Triangle {
            positions: [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normal: [0.0, 0.0, 1.0],
            tangent: Some([1.0, 0.0, 0.0, 1.0]),
            mode: MODE_TRIANGLES,
        }
    }
}

// Writes binary glTF files: every accessor's data in one buffer, the file's binary chunk.
#[derive(Default)]
struct GlbBuilder {
    binary: Vec<u8>,
    buffer_views: Vec<String>,
    accessors: Vec<String>,
    meshes: Vec<String>,
    nodes: Vec<String>,
    scene_nodes: Vec<usize>,
}

impl GlbBuilder {
    fn floats(&mut self, values: &[f32], kind: &str, bounds: Option<(Vec3, Vec3)>) -> usize {
        let components = match kind {
            "VEC2" => 2,
            "VEC3" => 3,
            _ => 4,
        };
        let bounds = bounds
            .map(|(minimum, maximum)| {
                format!(
                    r#","min":[{},{},{}],"max":[{},{},{}]"#,
                    minimum.x, minimum.y, minimum.z, maximum.x, maximum.y, maximum.z
                )
            })
            .unwrap_or_default();
        let bytes: Vec<u8> = values.iter().flat_map(|value| value.to_le_bytes()).collect();

        self.accessor(&bytes, COMPONENT_FLOAT, values.len() / components, kind, &bounds)
    }

    fn indices(&mut self, values: &[u32]) -> usize {
        let bytes: Vec<u8> = values.iter().flat_map(|value| value.to_le_bytes()).collect();

        self.accessor(&bytes, COMPONENT_UNSIGNED_INT, values.len(), "SCALAR", "")
    }

    fn accessor(&mut self, bytes: &[u8], component: u32, count: usize, kind: &str, extra: &str) -> usize {
        let offset = self.binary.len();
        self.binary.extend_from_slice(bytes);
        self.buffer_views.push(format!(
            r#"{{"buffer":0,"byteOffset":{offset},"byteLength":{}}}"#,
            bytes.len()
        ));
        self.accessors.push(format!(
            r#"{{"bufferView":{},"componentType":{component},"count":{count},"type":"{kind}"{extra}}}"#,
            self.buffer_views.len() - 1
        ));

        self.accessors.len() - 1
    }

    fn primitive(&mut self, triangle: &Triangle, indexed: bool) -> String {
        let positions: Vec<f32> = triangle.positions.iter().flatten().copied().collect();
        let minimum = triangle
            .positions
            .iter()
            .map(|&p| Vec3::from(p))
            .fold(Vec3::MAX, Vec3::min);
        let maximum = triangle
            .positions
            .iter()
            .map(|&p| Vec3::from(p))
            .fold(Vec3::MIN, Vec3::max);
        let position = self.floats(&positions, "VEC3", Some((minimum, maximum)));
        let normal = self.floats(&triangle.normal.repeat(3), "VEC3", None);
        let uv = self.floats(&[0.0, 0.0, 1.0, 0.0, 0.0, 1.0], "VEC2", None);
        let tangent = triangle
            .tangent
            .map(|tangent| {
                let accessor = self.floats(&tangent.repeat(3), "VEC4", None);
                format!(r#","TANGENT":{accessor}"#)
            })
            .unwrap_or_default();
        let indices = if indexed {
            format!(r#","indices":{}"#, self.indices(&[0, 1, 2]))
        } else {
            String::new()
        };

        format!(
            r#"{{"attributes":{{"POSITION":{position},"NORMAL":{normal},"TEXCOORD_0":{uv}{tangent}}},"mode":{}{indices}}}"#,
            triangle.mode
        )
    }

    fn mesh(&mut self, primitives: &[String]) -> usize {
        self.meshes
            .push(format!(r#"{{"primitives":[{}]}}"#, primitives.join(",")));

        self.meshes.len() - 1
    }

    fn node(&mut self, fields: &str) -> usize {
        self.nodes.push(format!("{{{fields}}}"));

        self.nodes.len() - 1
    }

    fn mesh_node(&mut self, name: &str, triangle: &Triangle) -> usize {
        let primitive = self.primitive(triangle, true);
        let mesh = self.mesh(&[primitive]);

        self.node(&format!(r#""name":"{name}","mesh":{mesh}"#))
    }

    fn bytes(&self) -> Vec<u8> {
        let scene_nodes: Vec<String> = self.scene_nodes.iter().map(usize::to_string).collect();
        let json = format!(
            r#"{{"asset":{{"version":"2.0"}},"scene":0,"scenes":[{{"nodes":[{}]}}],"nodes":[{}],"meshes":[{}],"accessors":[{}],"bufferViews":[{}],"buffers":[{{"byteLength":{}}}]}}"#,
            scene_nodes.join(","),
            self.nodes.join(","),
            self.meshes.join(","),
            self.accessors.join(","),
            self.buffer_views.join(","),
            self.binary.len()
        );
        let json = GlbBuilder::padded(json.into_bytes(), b' ');
        let binary = GlbBuilder::padded(self.binary.clone(), 0);
        let length = GLB_HEADER_BYTES + 2 * CHUNK_HEADER_BYTES + json.len() + binary.len();
        let mut bytes = Vec::with_capacity(length);
        bytes.extend_from_slice(GLB_MAGIC);
        bytes.extend_from_slice(&GLB_VERSION.to_le_bytes());
        bytes.extend_from_slice(&(length as u32).to_le_bytes());

        for (kind, chunk) in [(JSON_CHUNK, &json), (BINARY_CHUNK, &binary)] {
            bytes.extend_from_slice(&(chunk.len() as u32).to_le_bytes());
            bytes.extend_from_slice(kind);
            bytes.extend_from_slice(chunk);
        }

        bytes
    }

    fn padded(mut bytes: Vec<u8>, padding: u8) -> Vec<u8> {
        bytes.resize(bytes.len().next_multiple_of(CHUNK_ALIGNMENT), padding);

        bytes
    }
}

fn single_node(name: &str, triangle: &Triangle) -> Vec<u8> {
    let mut builder = GlbBuilder::default();
    let node = builder.mesh_node(name, triangle);
    builder.scene_nodes.push(node);

    builder.bytes()
}

fn load_error(bytes: &[u8]) -> String {
    match GlbLoader::load(bytes) {
        Ok(_) => panic!("the file loaded"),
        Err(error) => format!("{error:#}"),
    }
}

fn assert_close(actual: Vec3, expected: Vec3) {
    assert!(actual.abs_diff_eq(expected, TOLERANCE), "{actual} is not {expected}");
}

#[test]
fn gltf_axes_become_the_model_frame() {
    let mesh = GlbLoader::load(&single_node("upright", &Triangle::upright())).unwrap();
    let vertex = mesh.vertices[2];

    assert_eq!(vertex.position, [0.0, 0.0, 1.0]);
    assert_close(vertex.tangent_frame.normal(), Vec3::NEG_Y);
    assert_close(vertex.tangent_frame.tangent(), Vec3::X);
    assert_eq!(mesh.submeshes[0].node, "upright");
    assert_eq!(mesh.submeshes[0].index_count, 3);
}

#[test]
fn node_transforms_compose_through_the_hierarchy_in_the_model_frame() {
    let mut builder = GlbBuilder::default();
    let child = builder.mesh_node("child", &Triangle::upright());
    let parent = builder.node(&format!(
        r#""name":"parent","translation":[0,2,0],"children":[{child}]"#
    ));
    builder.nodes[child] = builder.nodes[child].replacen('{', r#"{"translation":[1,0,0],"#, 1);
    builder.scene_nodes.push(parent);

    let mesh = GlbLoader::load(&builder.bytes()).unwrap();
    let transform = mesh.submeshes[0].transform;

    assert!(
        transform.abs_diff_eq(Mat4::from_translation(Vec3::new(1.0, 0.0, 2.0)), TOLERANCE),
        "{transform}"
    );
}

#[test]
fn the_bounding_box_encloses_the_vertices_where_their_nodes_place_them() {
    let mut builder = GlbBuilder::default();
    let node = builder.mesh_node("moved", &Triangle::upright());
    builder.nodes[node] = builder.nodes[node].replacen('{', r#"{"translation":[1,2,0],"#, 1);
    builder.scene_nodes.push(node);

    let mesh = GlbLoader::load(&builder.bytes()).unwrap();

    assert_close(mesh.bounding_box.minimum, Vec3::new(1.0, 0.0, 2.0));
    assert_close(mesh.bounding_box.maximum, Vec3::new(2.0, 0.0, 3.0));
    assert!((mesh.bounding_box.radius() - 0.5_f32.hypot(0.5)).abs() < TOLERANCE);
}

#[test]
fn submeshes_index_into_the_shared_vertices() {
    let mut builder = GlbBuilder::default();
    let first = builder.mesh_node("first", &Triangle::upright());
    let second = builder.mesh_node("second", &Triangle::upright());
    builder.scene_nodes.extend([first, second]);

    let mesh = GlbLoader::load(&builder.bytes()).unwrap();

    assert_eq!(mesh.vertices.len(), 6);
    assert_eq!(mesh.indices, [0, 1, 2, 3, 4, 5]);
    assert_eq!(mesh.submeshes[1].first_index, 3);
}

#[test]
fn unindexed_triangles_draw_their_vertices_in_order() {
    let mut builder = GlbBuilder::default();
    let primitive = builder.primitive(&Triangle::upright(), false);
    let mesh = builder.mesh(&[primitive]);
    let node = builder.node(&format!(r#""name":"unindexed","mesh":{mesh}"#));
    builder.scene_nodes.push(node);

    assert_eq!(GlbLoader::load(&builder.bytes()).unwrap().indices, [0, 1, 2]);
}

#[test]
fn tangent_frames_keep_their_normal_tangent_and_bitangent() {
    let normals = [Vec3::Z, Vec3::NEG_Z, Vec3::new(0.3, -0.5, 0.8), Vec3::X];
    let tangents = [Vec3::X, Vec3::Y, Vec3::new(1.0, 0.2, -0.25), Vec3::NEG_Z];

    for (&normal, &tangent) in normals.iter().zip(&tangents) {
        for sign in [1.0, -1.0] {
            let frame = TangentFrame::new(normal, tangent.extend(sign)).unwrap();
            let normal = normal.normalize();
            let tangent = tangent.reject_from_normalized(normal).normalize();

            assert_close(frame.normal(), normal);
            assert_close(frame.tangent(), tangent);
            assert_eq!(frame.bitangent_sign(), sign);
            assert_close(frame.bitangent(), normal.cross(tangent) * sign);
        }
    }
}

#[test]
fn tangent_frames_keep_their_sign_when_w_rounds_to_zero() {
    // A half turn about the tangent's axis has w = 0.
    let frame = TangentFrame::new(Vec3::NEG_Z, Vec4::new(1.0, 0.0, 0.0, -1.0)).unwrap();

    assert_eq!(frame.bitangent_sign(), -1.0);
    assert_close(frame.normal(), Vec3::NEG_Z);
}

#[test]
fn meshes_without_tangents_are_refused() {
    let triangle = Triangle {
        tangent: None,
        ..Triangle::upright()
    };

    assert!(load_error(&single_node("flat", &triangle)).contains("no tangents"));
}

#[test]
fn meshes_with_several_primitives_are_refused() {
    let mut builder = GlbBuilder::default();
    let first = builder.primitive(&Triangle::upright(), true);
    let second = builder.primitive(&Triangle::upright(), true);
    let mesh = builder.mesh(&[first, second]);
    let node = builder.node(&format!(r#""name":"split","mesh":{mesh}"#));
    builder.scene_nodes.push(node);

    assert!(load_error(&builder.bytes()).contains("several primitives"));
}

#[test]
fn primitives_other_than_triangle_lists_are_refused() {
    let triangle = Triangle {
        mode: MODE_LINES,
        ..Triangle::upright()
    };

    assert!(load_error(&single_node("lines", &triangle)).contains("not a triangle list"));
}

#[test]
fn mesh_nodes_sharing_a_name_are_refused() {
    let mut builder = GlbBuilder::default();
    let first = builder.mesh_node("twin", &Triangle::upright());
    let second = builder.mesh_node("twin", &Triangle::upright());
    builder.scene_nodes.extend([first, second]);

    assert!(load_error(&builder.bytes()).contains("same name"));
}

#[test]
fn text_gltf_files_are_refused() {
    assert!(load_error(br#"{"asset":{"version":"2.0"}}"#).contains("not a binary glTF file"));
}

#[test]
fn mesh_files_need_the_glb_extension() {
    let error = MeshData::read(std::path::Path::new("model.gltf")).err().unwrap();

    assert!(error.to_string().contains("binary glTF"));
}
