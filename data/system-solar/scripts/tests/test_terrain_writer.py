import struct

import numpy as np
import zstandard

from operations.cube_map import CUBE_FACE_COUNT, CubeMap
from operations.cube_projection import CubeProjection
from operations.raster import Raster
from operations.terrain_writer import TerrainWriter

R32_SFLOAT = 100


def test_the_range_is_the_maps_own_rounded_outward(tmp_path):
    faces = [Raster(np.zeros((2, 2, 1), np.float32)) for _ in range(CUBE_FACE_COUNT)]
    faces[0].pixels[0, 0] = -9128.4
    faces[5].pixels[1, 1] = 10786.2
    TerrainWriter(tmp_path).write("luna", CubeMap(faces, CubeProjection.EQUAL_ANGLE))

    assert (tmp_path / "luna.kdl").read_text() == 'height-map "luna"\nminimum -9129\nmaximum 10787\n'


def test_heights_read_back_as_written(tmp_path):
    heights = np.linspace(-9000, 11000, CUBE_FACE_COUNT * 4 * 4, dtype=np.float32).reshape(CUBE_FACE_COUNT, 4, 4, 1)
    TerrainWriter(tmp_path).write("luna", CubeMap([Raster(face) for face in heights], CubeProjection.EQUAL_ANGLE))
    data = (tmp_path / "luna.ktx2").read_bytes()

    vk_format, type_size, width, height = struct.unpack_from("<4I", data, 12)
    face_count, level_count = struct.unpack_from("<2I", data, 36)
    assert (vk_format, type_size, width, height, face_count, level_count) == (R32_SFLOAT, 4, 4, 4, CUBE_FACE_COUNT, 3)

    offset, length, _ = struct.unpack_from("<3Q", data, 80)
    stored = np.frombuffer(zstandard.ZstdDecompressor().decompress(data[offset : offset + length]), np.float32)
    assert np.array_equal(stored.reshape(heights.shape), heights)
