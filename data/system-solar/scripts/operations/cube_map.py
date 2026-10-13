from pathlib import Path

import numpy as np
import OpenEXR

from operations.raster import Raster

CUBE_FACE_COUNT = 6


class CubeMap:
    def __init__(self, faces: list[Raster]):
        self.faces = faces

    # A sky map holds right ascension 0 at its center, growing to the left, and the celestial north pole on row 0.
    @staticmethod
    def from_sky_map(path: Path, face_size: int, samples_per_axis: int) -> "CubeMap":
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
                    total += CubeMap._sample(sky, CubeMap._direction(face, s, t))

            faces.append(Raster(total / samples_per_axis**2))

        return CubeMap(faces)

    # Inverse of the cube map face selection table in the Vulkan specification.
    @staticmethod
    def _direction(face: int, s: np.ndarray, t: np.ndarray) -> np.ndarray:
        s, t = np.broadcast_arrays(s, t)
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
