use std::ops::Range;

use super::GridVertex;

const TRIANGLE_CORNERS_PER_CELL: u32 = 6;

struct Grid {
    cells: u32,
    indices: Range<u32>,
}

// Square grids of several sizes sharing one vertex and index buffer. Every cell is split along the diagonal from
// its lowest corner to its highest, which vertex morphing relies on. Triangles wind clockwise in columns and rows:
// a cube face's columns and rows are seen from inside the cube, so they face outward.
pub struct GridMesh {
    pub vertices: Vec<GridVertex>,
    pub indices: Vec<u32>,
    grids: Vec<Grid>,
}

impl GridMesh {
    pub fn new(cell_counts: impl IntoIterator<Item = u32>) -> GridMesh {
        let mut mesh = GridMesh {
            vertices: Vec::new(),
            indices: Vec::new(),
            grids: Vec::new(),
        };

        for cells in cell_counts {
            mesh.add_grid(cells);
        }

        mesh
    }

    pub fn indices(&self, cells: u32) -> Range<u32> {
        self.grids
            .iter()
            .find(|grid| grid.cells == cells)
            .map(|grid| grid.indices.clone())
            .unwrap_or_else(|| panic!("no grid of {cells} cells"))
    }

    fn add_grid(&mut self, cells: u32) {
        let first_vertex = self.vertices.len() as u32;
        let first_index = self.indices.len() as u32;
        let points_per_side = cells + 1;

        for row in 0..points_per_side {
            for column in 0..points_per_side {
                self.vertices.push(GridVertex { cell: [column, row] });
            }
        }

        for row in 0..cells {
            for column in 0..cells {
                let corner = first_vertex + row * points_per_side + column;
                let next_column = corner + 1;
                let next_row = corner + points_per_side;
                let opposite = next_row + 1;
                self.indices
                    .extend_from_slice(&[corner, opposite, next_column, corner, next_row, opposite]);
            }
        }

        let index_count = cells * cells * TRIANGLE_CORNERS_PER_CELL;
        self.grids.push(Grid {
            cells,
            indices: first_index..first_index + index_count,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grids_index_only_their_own_vertices() {
        let mesh = GridMesh::new([2, 4]);
        let small = mesh.indices(2);
        let large = mesh.indices(4);
        let small_vertices = 3 * 3;

        assert_eq!(small.len(), 2 * 2 * 6);
        assert_eq!(large.len(), 4 * 4 * 6);
        assert!(
            mesh.indices[small.start as usize..small.end as usize]
                .iter()
                .all(|&index| index < small_vertices)
        );
        assert!(
            mesh.indices[large.start as usize..large.end as usize]
                .iter()
                .all(|&index| index >= small_vertices)
        );
    }
}
