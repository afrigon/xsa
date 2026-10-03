mod vertex;

pub use vertex::Vertex;

use glam::Vec3;

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

struct CubeFace {
    normal: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
}

const CUBE_FACES: [CubeFace; 6] = [
    CubeFace {
        normal: Vec3::X,
        horizontal: Vec3::Y,
        vertical: Vec3::Z,
    },
    CubeFace {
        normal: Vec3::NEG_X,
        horizontal: Vec3::NEG_Y,
        vertical: Vec3::Z,
    },
    CubeFace {
        normal: Vec3::Y,
        horizontal: Vec3::NEG_X,
        vertical: Vec3::Z,
    },
    CubeFace {
        normal: Vec3::NEG_Y,
        horizontal: Vec3::X,
        vertical: Vec3::Z,
    },
    CubeFace {
        normal: Vec3::Z,
        horizontal: Vec3::X,
        vertical: Vec3::Y,
    },
    CubeFace {
        normal: Vec3::NEG_Z,
        horizontal: Vec3::X,
        vertical: Vec3::NEG_Y,
    },
];

impl Mesh {
    pub fn cube_sphere(radius: f32, subdivisions: u32) -> Mesh {
        let points_per_side = subdivisions + 1;
        let mut vertices = Vec::with_capacity((6 * points_per_side * points_per_side) as usize);
        let mut indices = Vec::with_capacity((6 * subdivisions * subdivisions * 6) as usize);

        for face in CUBE_FACES {
            let first_vertex = vertices.len() as u32;

            for row in 0..points_per_side {
                for column in 0..points_per_side {
                    let u = column as f32 / subdivisions as f32 * 2.0 - 1.0;
                    let v = row as f32 / subdivisions as f32 * 2.0 - 1.0;
                    let direction = (face.normal + u * face.horizontal + v * face.vertical).normalize();
                    vertices.push(Vertex {
                        position: (direction * radius).to_array(),
                        normal: direction.to_array(),
                    });
                }
            }

            for row in 0..subdivisions {
                for column in 0..subdivisions {
                    let corner = first_vertex + row * points_per_side + column;
                    let right = corner + 1;
                    let above = corner + points_per_side;
                    let above_right = above + 1;
                    indices.extend_from_slice(&[corner, right, above_right, corner, above_right, above]);
                }
            }
        }

        Mesh { vertices, indices }
    }
}
