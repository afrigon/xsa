import logging
from pathlib import Path

import numpy as np
import OpenEXR
import rasterio

from operations.cube_projection import CubeProjection
from operations.raster import Raster, SourceLayout

CUBE_FACE_COUNT = 6
ROWS_PER_READ = 64
# Each source pixel is split this many times per axis, so a pixel straddling texels is shared between them.
SUBDIVISIONS = 2

logger = logging.getLogger(__name__)


class CubeMap:
    def __init__(self, faces: list[Raster], projection: CubeProjection):
        self.faces = faces
        self.projection = projection

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
        one = np.ones_like(s)
        x, y, z = [
            (one, -t, -s),
            (-one, -t, s),
            (s, one, t),
            (s, -one, -t),
            (s, -t, one),
            (-s, -t, -one),
        ][face]
        direction = np.stack([x, y, z], axis=-1)
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
        s = projection.from_plane(np.choose(face, [-z, z, x, x, x, -x]) / major)
        t = projection.from_plane(np.choose(face, [-y, -y, z, -z, -y, -y]) / major)
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
