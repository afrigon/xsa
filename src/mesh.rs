use glam::Vec3;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

const CUBE_FACES: [(Vec3, Vec3, Vec3); 6] = [
    (Vec3::X, Vec3::Y, Vec3::Z),
    (Vec3::NEG_X, Vec3::NEG_Y, Vec3::Z),
    (Vec3::Y, Vec3::NEG_X, Vec3::Z),
    (Vec3::NEG_Y, Vec3::X, Vec3::Z),
    (Vec3::Z, Vec3::X, Vec3::Y),
    (Vec3::NEG_Z, Vec3::X, Vec3::NEG_Y),
];

pub fn cube_sphere(radius: f32, subdivisions: u32) -> Mesh {
    let points_per_side = subdivisions + 1;
    let mut vertices = Vec::with_capacity((6 * points_per_side * points_per_side) as usize);
    let mut indices = Vec::with_capacity((6 * subdivisions * subdivisions * 6) as usize);

    for (normal, horizontal, vertical) in CUBE_FACES {
        let first_vertex = vertices.len() as u32;
        for row in 0..points_per_side {
            for column in 0..points_per_side {
                let u = column as f32 / subdivisions as f32 * 2.0 - 1.0;
                let v = row as f32 / subdivisions as f32 * 2.0 - 1.0;
                let direction = (normal + u * horizontal + v * vertical).normalize();
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
