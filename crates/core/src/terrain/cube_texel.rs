use glam::IVec3;
use resource_loader::utils::CubeFace;

use super::TexelSource;

// A texel of a face of a cube map `extent` texels wide, by column and row. Either may lie a few texels beyond the
// face, reaching into the adjacent faces, as filters sampling near an edge do.
//
// Texels are placed on the cube by exact integer positions: a face spans −extent to extent along its axes, and its
// texel centres sit at the odd coordinates in between. Mirrored by `terrain/cube.slang`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CubeTexel {
    pub face: CubeFace,
    pub column: i32,
    pub row: i32,
}

impl CubeTexel {
    // The stored texels this one reads: itself on its face, or folded over the edge onto the adjacent face.
    pub fn source(self, extent: i32) -> TexelSource {
        let inside = |coordinate: i32| (0..extent).contains(&coordinate);

        if inside(self.column) && inside(self.row) {
            return TexelSource::Stored(self);
        }

        let face = self.face;
        let point = self.point(extent);
        let column_edge = face.column_axis() * point.dot(face.column_axis()).signum();
        let row_edge = face.row_axis() * point.dot(face.row_axis()).signum();

        if inside(self.row) {
            return TexelSource::Stored(CubeTexel::folded(face, point, column_edge, extent));
        }

        if inside(self.column) {
            return TexelSource::Stored(CubeTexel::folded(face, point, row_edge, extent));
        }

        // The three texels as far from the corner on each face: this face's, reflected through the corner, and the
        // two reached by folding over both edges in either order.
        let column_excess = point.dot(column_edge) - extent;
        let row_excess = point.dot(row_edge) - extent;
        let reflected = point - column_edge * 2 * column_excess - row_edge * 2 * row_excess;
        let over_column = CubeTexel::folded(face, point, column_edge, extent);
        let over_row = CubeTexel::folded(face, point, row_edge, extent);

        TexelSource::Corner([
            CubeTexel::at(face, reflected, extent),
            CubeTexel::folded(over_column.face, over_column.point(extent), row_edge, extent),
            CubeTexel::folded(over_row.face, over_row.point(extent), column_edge, extent),
        ])
    }

    // Its centre on its face's plane, which lies beyond the face for a texel beyond its edges.
    fn point(self, extent: i32) -> IVec3 {
        self.face.normal() * extent
            + self.face.column_axis() * (2 * self.column + 1 - extent)
            + self.face.row_axis() * (2 * self.row + 1 - extent)
    }

    // A point beyond a face's edge, folded over that edge onto the adjacent face, whose normal is the edge's direction.
    fn folded(face: CubeFace, point: IVec3, edge: IVec3, extent: i32) -> CubeTexel {
        let excess = point.dot(edge) - extent;
        let adjacent = CubeFace::from_normal(edge).expect("an edge's direction is a face's normal");

        CubeTexel::at(adjacent, point - edge * excess - face.normal() * excess, extent)
    }

    fn at(face: CubeFace, point: IVec3, extent: i32) -> CubeTexel {
        let coordinate = |axis: IVec3| (point.dot(axis) + extent - 1).div_euclid(2);

        CubeTexel {
            face,
            column: coordinate(face.column_axis()),
            row: coordinate(face.row_axis()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXTENT: i32 = 8;

    fn stored(source: TexelSource) -> CubeTexel {
        match source {
            TexelSource::Stored(texel) => texel,
            TexelSource::Corner(_) => panic!("{source:?} is a corner"),
        }
    }

    #[test]
    fn a_texel_on_its_face_is_itself() {
        let texel = CubeTexel {
            face: CubeFace::PositiveZ,
            column: 3,
            row: 5,
        };

        assert_eq!(texel.source(EXTENT), TexelSource::Stored(texel));
    }

    // One step past an edge lands on the adjacent face's edge, and one step back past that edge returns.
    #[test]
    fn folding_over_an_edge_and_back_returns() {
        let outward = |texel: CubeTexel| -> CubeTexel {
            let step = |coordinate: i32| match coordinate {
                0 => -1,
                coordinate if coordinate == EXTENT - 1 => EXTENT,
                coordinate => coordinate,
            };

            CubeTexel {
                column: step(texel.column),
                row: step(texel.row),
                ..texel
            }
        };

        for face in CubeFace::ALL {
            for along in 1..EXTENT - 1 {
                for (column, row) in [(-1, along), (EXTENT, along), (along, -1), (along, EXTENT)] {
                    let beyond = CubeTexel { face, column, row };
                    let neighbour = stored(beyond.source(EXTENT));
                    let edge = CubeTexel {
                        face,
                        column: column.clamp(0, EXTENT - 1),
                        row: row.clamp(0, EXTENT - 1),
                    };

                    assert_ne!(neighbour.face, face, "{beyond:?}");
                    assert_eq!(
                        stored(outward(neighbour).source(EXTENT)),
                        edge,
                        "{beyond:?} through {neighbour:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_corner_averages_three_faces() {
        let beyond = CubeTexel {
            face: CubeFace::PositiveX,
            column: -1,
            row: -1,
        };
        let TexelSource::Corner(texels) = beyond.source(EXTENT) else {
            panic!("a texel beyond both edges is a corner");
        };
        let faces: Vec<_> = texels.iter().map(|texel| texel.face).collect();

        assert_eq!(
            texels[0],
            CubeTexel {
                face: CubeFace::PositiveX,
                column: 0,
                row: 0
            }
        );
        assert!(faces.contains(&CubeFace::PositiveZ) && faces.contains(&CubeFace::PositiveY));
        assert!(
            texels
                .iter()
                .all(|texel| (0..EXTENT).contains(&texel.column) && (0..EXTENT).contains(&texel.row))
        );
    }
}
