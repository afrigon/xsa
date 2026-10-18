use glam::IVec3;
use resource_loader::terrain::{CubeFace, PAGE_BORDER, PAGE_EXTENT, PAGE_INTERIOR, PageKey};

use super::{CubeMapInput, LayerEncoding};

/// Cuts a level's pages out of a cube map: each page's own area and the border around it, taken from the neighbouring
/// pages and, across a face edge, from the adjacent face.
///
/// Texels are placed on the cube by exact integer positions: a face spans −extent to extent along its axes, and its
/// texel centres sit at the odd coordinates in between.
pub struct PageBuilder<'a> {
    faces: [&'a [u8]; 6],
    extent: i32,
    texel_size: usize,
    encoding: LayerEncoding,
}

impl<'a> PageBuilder<'a> {
    pub fn new(input: &'a CubeMapInput, level: u32, encoding: LayerEncoding) -> PageBuilder<'a> {
        PageBuilder {
            faces: CubeFace::ALL.map(|face| input.level_texels(face, level)),
            extent: input.level_extent(level) as i32,
            texel_size: input.format().block_size(),
            encoding,
        }
    }

    /// A page's texels in the input's format, rows top to bottom.
    pub fn page(&self, face: CubeFace, key: PageKey) -> Vec<u8> {
        let row_size = PAGE_EXTENT as usize * self.texel_size;
        let mut page = vec![0; PAGE_EXTENT as usize * row_size];
        let left = (key.x() * PAGE_INTERIOR) as i32 - PAGE_BORDER as i32;
        let top = (key.y() * PAGE_INTERIOR) as i32 - PAGE_BORDER as i32;
        let first = left.max(0);
        let last = (left + PAGE_EXTENT as i32).min(self.extent);

        for (row, page_row) in page.chunks_exact_mut(row_size).enumerate() {
            let y = top + row as i32;

            if self.inside(y) {
                let start = (first - left) as usize * self.texel_size;
                let end = (last - left) as usize * self.texel_size;
                page_row[start..end].copy_from_slice(self.span(face, first, last, y));
            }

            for (column, texel) in page_row.chunks_exact_mut(self.texel_size).enumerate() {
                let x = left + column as i32;

                if !self.inside(x) || !self.inside(y) {
                    self.fill_beyond(face, x, y, texel);
                }
            }
        }

        page
    }

    fn fill_beyond(&self, face: CubeFace, x: i32, y: i32, output: &mut [u8]) {
        let point = self.point(face, x, y);
        let column_edge = face.column_axis() * point.dot(face.column_axis()).signum();
        let row_edge = face.row_axis() * point.dot(face.row_axis()).signum();

        if self.inside(y) {
            let (face, point) = self.folded(face, point, column_edge);
            return output.copy_from_slice(self.texel_at(face, point));
        }

        if self.inside(x) {
            let (face, point) = self.folded(face, point, row_edge);
            return output.copy_from_slice(self.texel_at(face, point));
        }

        // Three faces meet at a cube corner, so no diagonal neighbour exists. The texel takes the average of the three
        // texels as far from the corner on each face: this face's, reflected through the corner, and the two reached
        // by folding over both edges in either order.
        let column_excess = point.dot(column_edge) - self.extent;
        let row_excess = point.dot(row_edge) - self.extent;
        let reflected = point - column_edge * 2 * column_excess - row_edge * 2 * row_excess;
        let (column_face, column_point) = self.folded(face, point, column_edge);
        let (column_face, column_point) = self.folded(column_face, column_point, row_edge);
        let (row_face, row_point) = self.folded(face, point, row_edge);
        let (row_face, row_point) = self.folded(row_face, row_point, column_edge);
        let texels = [
            self.texel_at(face, reflected),
            self.texel_at(column_face, column_point),
            self.texel_at(row_face, row_point),
        ];

        self.encoding.average(&texels, output);
    }

    fn inside(&self, coordinate: i32) -> bool {
        (0..self.extent).contains(&coordinate)
    }

    // The point of a texel on a face's plane, which lies beyond the face for a texel beyond its edges.
    fn point(&self, face: CubeFace, x: i32, y: i32) -> IVec3 {
        face.normal() * self.extent
            + face.column_axis() * (2 * x + 1 - self.extent)
            + face.row_axis() * (2 * y + 1 - self.extent)
    }

    // A point beyond a face's edge, folded over that edge onto the adjacent face, whose normal is the edge's direction.
    fn folded(&self, face: CubeFace, point: IVec3, edge: IVec3) -> (CubeFace, IVec3) {
        let excess = point.dot(edge) - self.extent;
        let adjacent = CubeFace::from_normal(edge).unwrap();

        (adjacent, point - edge * excess - face.normal() * excess)
    }

    fn texel_at(&self, face: CubeFace, point: IVec3) -> &[u8] {
        let coordinate = |axis: IVec3| (point.dot(axis) + self.extent - 1) / 2;
        let x = coordinate(face.column_axis());

        self.span(face, x, x + 1, coordinate(face.row_axis()))
    }

    // Texels `first` to `last`, excluded, of a row.
    fn span(&self, face: CubeFace, first: i32, last: i32, y: i32) -> &[u8] {
        let row = y as usize * self.extent as usize;
        let start = (row + first as usize) * self.texel_size;
        let end = (row + last as usize) * self.texel_size;

        &self.faces[face.index()][start..end]
    }
}
