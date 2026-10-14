import struct

import numpy as np
import zstandard

from operations.raster import Raster
from operations.terrain_writer import TerrainWriter

R16_UNORM = 70


def test_the_range_is_the_maps_own_rounded_outward(tmp_path):
    heights = Raster(np.array([[[-9128.4], [10786.2]], [[0.0], [512.5]]], np.float32))
    TerrainWriter(tmp_path).write("luna", heights)

    assert (tmp_path / "luna.kdl").read_text() == 'height-map "luna"\nminimum -9129\nmaximum 10787\n'


def test_heights_read_back_within_half_a_step(tmp_path):
    heights = Raster(np.linspace(-9000, 11000, 8 * 4, dtype=np.float32).reshape(4, 8, 1))
    TerrainWriter(tmp_path).write("luna", heights)
    data = (tmp_path / "luna.ktx2").read_bytes()

    vk_format, type_size, width, height = struct.unpack_from("<4I", data, 12)
    level_count = struct.unpack_from("<I", data, 40)[0]
    assert (vk_format, type_size, width, height, level_count) == (R16_UNORM, 2, 8, 4, 4)

    offset, length, _ = struct.unpack_from("<3Q", data, 80)
    stored = np.frombuffer(zstandard.ZstdDecompressor().decompress(data[offset : offset + length]), np.uint16)
    meters = -9000 + stored.reshape(4, 8) / 65535 * 20000
    step = 20000 / 65535
    assert np.abs(meters - heights.pixels[..., 0]).max() <= step / 2 + 1e-3
