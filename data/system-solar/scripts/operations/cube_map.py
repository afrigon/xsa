import logging
from pathlib import Path

import numpy as np
import OpenEXR
import rasterio

from operations.cube_projection import CubeProjection
from operations.raster import Raster, SourceLayout

CUBE_FACE_COUNT = 6
# Each face's outward normal, the axis its columns run along left to right and the one its rows run along top to
# bottom: Vulkan's face order and orientation in the body's frame.
FACE_BASES = np.array(
    [
        [[1, 0, 0], [0, 0, -1], [0, -1, 0]],
        [[-1, 0, 0], [0, 0, 1], [0, -1, 0]],
        [[0, 1, 0], [1, 0, 0], [0, 0, 1]],
        [[0, -1, 0], [1, 0, 0], [0, 0, -1]],
        [[0, 0, 1], [1, 0, 0], [0, -1, 0]],
        [[0, 0, -1], [-1, 0, 0], [0, -1, 0]],
    ]
)
NORMAL, COLUMN_AXIS, ROW_AXIS = range(3)
ROWS_PER_READ = 64
# Each source pixel is split this many times per axis, so a pixel straddling texels is shared between them.
SUBDIVISIONS = 2

logger = logging.getLogger(__name__)


class CubeMap:
    def __init__(self, faces: list[Raster], projection: CubeProjection):
        self.faces = faces
        self.projection = projection

    @property
    def face_size(self) -> int:
        return self.faces[0].width

    # The mean over the sphere, each texel weighted by the ground it covers.
    def area_mean(self) -> np.ndarray:
        angles = CubeMap.texel_solid_angles(self.face_size, self.projection)[..., None]
        total = sum((face.pixels.astype(np.float64) * angles).sum(axis=(0, 1)) for face in self.faces)
        return total / (4 * np.pi)

    # The faces with `border` texels around each, taken across face edges from the adjacent face. Three faces meet at
    # a cube corner and no diagonal neighbour exists: a texel there is the average of the three texels as far from
    # the corner on each face.
    def padded(self, border: int) -> np.ndarray:
        size = self.face_size
        faces = np.stack([face.pixels for face in self.faces])
        padded = np.empty((CUBE_FACE_COUNT, size + 2 * border, size + 2 * border, faces.shape[-1]), faces.dtype)
        padded[:, border:-border, border:-border] = faces
        coordinates = np.arange(-border, size + border)
        x, y = [axis.ravel() for axis in np.meshgrid(coordinates, coordinates)]
        beyond = (x < 0) | (x >= size) | (y < 0) | (y >= size)
        x, y = x[beyond], y[beyond]

        for face in range(CUBE_FACE_COUNT):
            normal, column_axis, row_axis = FACE_BASES[face]
            point = size * normal + (2 * x + 1 - size)[:, None] * column_axis + (2 * y + 1 - size)[:, None] * row_axis
            column_edge = column_axis * np.sign(2 * x + 1 - size)[:, None]
            row_edge = row_axis * np.sign(2 * y + 1 - size)[:, None]
            normals = np.broadcast_to(normal, point.shape)
            across_column = (x < 0) | (x >= size)
            across_row = (y < 0) | (y >= size)
            values = np.empty((len(x), faces.shape[-1]), faces.dtype)

            only_column = across_column & ~across_row
            folded, folded_normals = CubeMap._folded(point, normals, column_edge, size)
            values[only_column] = CubeMap._texels_at(faces, folded[only_column], folded_normals[only_column], size)

            only_row = across_row & ~across_column
            folded, folded_normals = CubeMap._folded(point, normals, row_edge, size)
            values[only_row] = CubeMap._texels_at(faces, folded[only_row], folded_normals[only_row], size)

            corner = across_column & across_row
            column_excess = (point * column_edge).sum(axis=-1, keepdims=True) - size
            row_excess = (point * row_edge).sum(axis=-1, keepdims=True) - size
            reflected = point - column_edge * 2 * column_excess - row_edge * 2 * row_excess
            by_column, by_column_normals = CubeMap._folded(point, normals, column_edge, size)
            by_column, by_column_normals = CubeMap._folded(by_column, by_column_normals, row_edge, size)
            by_row, by_row_normals = CubeMap._folded(point, normals, row_edge, size)
            by_row, by_row_normals = CubeMap._folded(by_row, by_row_normals, column_edge, size)
            values[corner] = (
                CubeMap._texels_at(faces, reflected[corner], normals[corner], size).astype(np.float64)
                + CubeMap._texels_at(faces, by_column[corner], by_column_normals[corner], size)
                + CubeMap._texels_at(faces, by_row[corner], by_row_normals[corner], size)
            ) / 3

            padded[face, y + border, x + border] = values

        return padded

    # Texel centers sit at odd integer positions on a cube spanning -size to size: a point beyond a face's edge,
    # folded over that edge onto the adjacent face, whose normal is the edge's direction.
    @staticmethod
    def _folded(
        point: np.ndarray, normals: np.ndarray, edge: np.ndarray, size: int
    ) -> tuple[np.ndarray, np.ndarray]:
        excess = (point * edge).sum(axis=-1, keepdims=True) - size
        return point - edge * excess - normals * excess, edge

    @staticmethod
    def _texels_at(faces: np.ndarray, point: np.ndarray, normals: np.ndarray, size: int) -> np.ndarray:
        face = CubeMap._face_of(normals)
        column = ((point * FACE_BASES[face, COLUMN_AXIS]).sum(axis=-1) + size - 1) // 2
        row = ((point * FACE_BASES[face, ROW_AXIS]).sum(axis=-1) + size - 1) // 2
        return faces[face, row, column]

    @staticmethod
    def _face_of(normals: np.ndarray) -> np.ndarray:
        axis = np.abs(normals).argmax(axis=-1)
        negative = np.take_along_axis(normals, axis[:, None], axis=-1)[:, 0] < 0
        return 2 * axis + negative

    # Each texel center's direction on one face, rows top to bottom.
    @staticmethod
    def texel_directions(face: int, face_size: int, projection: CubeProjection) -> np.ndarray:
        centers = (np.arange(face_size) + 0.5) / face_size * 2 - 1
        return CubeMap._direction(face, centers[None, :], centers[:, None], projection)

    # The solid angle each texel covers, the same on every face: the gnomonic plane's rectangle between its edges,
    # integrated in closed form.
    @staticmethod
    def texel_solid_angles(face_size: int, projection: CubeProjection) -> np.ndarray:
        edges = projection.to_plane(np.linspace(-1, 1, face_size + 1))
        corner = np.arctan2(edges[:, None] * edges[None, :], np.sqrt(1 + edges[:, None] ** 2 + edges[None, :] ** 2))
        return corner[1:, 1:] - corner[1:, :-1] - corner[:-1, 1:] + corner[:-1, :-1]

    # The shortest angle any texel edge spans. Texels shrink toward a face's border in both projections, so the
    # outermost row holds the shortest edge.
    @staticmethod
    def smallest_texel_width(face_size: int, projection: CubeProjection) -> float:
        edges = np.linspace(-1, 1, face_size + 1)
        outer = CubeMap._direction(0, edges, np.full_like(edges, -1.0), projection)
        inner = CubeMap._direction(0, edges, np.full_like(edges, edges[1]), projection)
        along_row = np.arccos(np.clip((outer[1:] * outer[:-1]).sum(axis=-1), -1, 1))
        along_column = np.arccos(np.clip((outer * inner).sum(axis=-1), -1, 1))
        return float(min(along_row.min(), along_column.min()))

    # Each texel is the area-weighted average of the equirectangular source's ground it covers, so no texel depends
    # on the source's rows shrinking toward the poles.
    @staticmethod
    def from_equirectangular(
        path: Path, face_size: int, projection: CubeProjection, layout: SourceLayout, scale: float = 1.0
    ) -> "CubeMap":
        texel_count = CUBE_FACE_COUNT * face_size * face_size
        sums = np.zeros(texel_count)
        weights = np.zeros(texel_count)

        with Raster._open(path) as source:
            columns = (np.arange(source.width * SUBDIVISIONS) + 0.5) / (source.width * SUBDIVISIONS)
            longitudes = np.radians(layout.left_edge_longitude) + columns * 2 * np.pi

            for top in range(0, source.height, ROWS_PER_READ):
                window = rasterio.windows.Window(0, top, source.width, min(ROWS_PER_READ, source.height - top))
                band = source.read(1, window=window).astype(np.float64) * scale
                rows = (top * SUBDIVISIONS + np.arange(band.shape[0] * SUBDIVISIONS) + 0.5) / (
                    source.height * SUBDIVISIONS
                )
                latitudes = (0.5 - rows if layout.north_up else rows - 0.5) * np.pi
                direction = np.stack(
                    np.broadcast_arrays(
                        np.cos(latitudes)[:, None] * np.cos(longitudes)[None, :],
                        np.cos(latitudes)[:, None] * np.sin(longitudes)[None, :],
                        np.sin(latitudes)[:, None],
                    ),
                    axis=-1,
                )
                face, column, row = CubeMap._texel(direction, face_size, projection)
                texels = ((face * face_size + row) * face_size + column).ravel()
                area = np.broadcast_to(np.cos(latitudes)[:, None], face.shape).ravel()
                values = band.repeat(SUBDIVISIONS, axis=0).repeat(SUBDIVISIONS, axis=1).ravel()
                sums += np.bincount(texels, values * area, texel_count)
                weights += np.bincount(texels, area, texel_count)
                logger.debug("%s: read rows %d to %d", path.name, top, top + band.shape[0])

        if not weights.all():
            raise ValueError(f"{path.name}: too coarse for {face_size}-texel faces, some texels cover no pixel")

        faces = (sums / weights).astype(np.float32).reshape(CUBE_FACE_COUNT, face_size, face_size, 1)
        return CubeMap([Raster(face) for face in faces], projection)

    # A sky map holds right ascension 0 at its center, growing to the left, and the celestial north pole on row 0.
    @staticmethod
    def from_sky_map(
        path: Path, face_size: int, projection: CubeProjection, samples_per_axis: int
    ) -> "CubeMap":
        with OpenEXR.File(str(path)) as file:
            sky = np.asarray(file.channels()["RGB"].pixels, np.float32)

        offsets = (np.arange(samples_per_axis) + 0.5) / samples_per_axis
        faces = []

        for face in range(CUBE_FACE_COUNT):
            total = np.zeros((face_size, face_size, 3), np.float32)

            for row_offset in offsets:
                for column_offset in offsets:
                    t = (np.arange(face_size)[:, None] + row_offset) / face_size * 2 - 1
                    s = (np.arange(face_size)[None, :] + column_offset) / face_size * 2 - 1
                    total += CubeMap._sample(sky, CubeMap._direction(face, s, t, projection))

            faces.append(Raster(total / samples_per_axis**2))

        return CubeMap(faces, projection)

    # Inverse of the cube map face selection table in the Vulkan specification.
    @staticmethod
    def _direction(face: int, s: np.ndarray, t: np.ndarray, projection: CubeProjection) -> np.ndarray:
        s, t = np.broadcast_arrays(projection.to_plane(s), projection.to_plane(t))
        normal, column_axis, row_axis = FACE_BASES[face]
        direction = normal + s[..., None] * column_axis + t[..., None] * row_axis
        return direction / np.linalg.norm(direction, axis=-1, keepdims=True)

    # The cube map face selection table in the Vulkan specification: the face and the texel a direction falls in.
    @staticmethod
    def _texel(
        direction: np.ndarray, face_size: int, projection: CubeProjection
    ) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
        x, y, z = direction[..., 0], direction[..., 1], direction[..., 2]
        ax, ay, az = np.abs(x), np.abs(y), np.abs(z)
        on_x = (ax >= ay) & (ax >= az)
        on_y = ~on_x & (ay >= az)
        face = np.where(on_x, np.where(x > 0, 0, 1), np.where(on_y, np.where(y > 0, 2, 3), np.where(z > 0, 4, 5)))
        major = np.where(on_x, ax, np.where(on_y, ay, az))
        s = projection.from_plane((direction * FACE_BASES[face, COLUMN_AXIS]).sum(axis=-1) / major)
        t = projection.from_plane((direction * FACE_BASES[face, ROW_AXIS]).sum(axis=-1) / major)
        column = np.minimum(((s + 1) / 2 * face_size).astype(np.int64), face_size - 1)
        row = np.minimum(((t + 1) / 2 * face_size).astype(np.int64), face_size - 1)
        return face, column, row

    @staticmethod
    def _sample(sky: np.ndarray, direction: np.ndarray) -> np.ndarray:
        height, width = sky.shape[:2]
        right_ascension = np.arctan2(direction[..., 1], direction[..., 0])
        declination = np.arcsin(np.clip(direction[..., 2], -1, 1))
        u = np.mod(0.5 - right_ascension / (2 * np.pi), 1.0)
        v = 0.5 - declination / np.pi
        x = u * width - 0.5
        y = np.clip(v * height - 0.5, 0, height - 1)
        x0 = np.floor(x)
        y0 = np.floor(y)
        fraction_x = (x - x0)[..., None]
        fraction_y = (y - y0)[..., None]
        columns = [(x0 + offset).astype(int) % width for offset in (0, 1)]
        rows = [np.minimum((y0 + offset).astype(int), height - 1) for offset in (0, 1)]
        top = sky[rows[0], columns[0]] * (1 - fraction_x) + sky[rows[0], columns[1]] * fraction_x
        bottom = sky[rows[1], columns[0]] * (1 - fraction_x) + sky[rows[1], columns[1]] * fraction_x
        return top * (1 - fraction_y) + bottom * fraction_y
