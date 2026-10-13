import struct

import numpy as np
import zstandard

from operations.compressor import Compressor
from operations.ktx2_writer import IDENTIFIER, Ktx2Writer
from operations.texture_format import TextureFormat


def test_header_and_levels_read_back(tmp_path):
    path = tmp_path / "texture.ktx2"
    texture_format = TextureFormat.BC7_SRGB
    compressor = Compressor(texture_format)
    sizes = [(8, 4), (4, 2), (2, 1), (1, 1)]
    levels = [[compressor.compress(np.full((height, width, 4), 128, np.uint8))] for width, height in sizes]
    Ktx2Writer(texture_format).write(path, 8, 4, levels)
    data = path.read_bytes()

    assert data[:12] == IDENTIFIER
    vk_format, _, width, height, depth, layers, faces, level_count, supercompression = struct.unpack_from("<9I", data, 12)
    assert (vk_format, width, height, depth, layers, faces, level_count, supercompression) == (146, 8, 4, 0, 0, 1, 4, 2)

    for level, (level_width, level_height) in enumerate(sizes):
        offset, length, uncompressed = struct.unpack_from("<3Q", data, 80 + 24 * level)
        raw = zstandard.ZstdDecompressor().decompress(data[offset : offset + length])
        assert len(raw) == uncompressed == texture_format.level_size(level_width, level_height)
        assert raw == levels[level][0]


def test_cube_map_levels_hold_every_face(tmp_path):
    path = tmp_path / "cube.ktx2"
    faces = [bytes([face]) * 4 * 4 * 4 for face in range(6)]
    small = [bytes([face]) * 4 for face in range(6)]
    Ktx2Writer(TextureFormat.RGBA8_SRGB).write(path, 4, 4, [faces, [bytes(16)] * 6, small])
    data = path.read_bytes()

    assert struct.unpack_from("<I", data, 36)[0] == 6
    offset, length, _ = struct.unpack_from("<3Q", data, 80)
    assert zstandard.ZstdDecompressor().decompress(data[offset : offset + length]) == b"".join(faces)


def test_half_float_textures_record_two_byte_channels(tmp_path):
    path = tmp_path / "values.ktx2"
    texture_format = TextureFormat.RGBA16_SFLOAT
    level = Compressor(texture_format).compress(np.array([[[1.5, -0.25, 2.0, 0.0]]], np.float32))
    Ktx2Writer(texture_format).write(path, 1, 1, [[level]])
    data = path.read_bytes()

    assert struct.unpack_from("<2I", data, 12) == (97, 2)
    offset, length, _ = struct.unpack_from("<3Q", data, 80)
    raw = zstandard.ZstdDecompressor().decompress(data[offset : offset + length])
    assert np.frombuffer(raw, np.float16).tolist() == [1.5, -0.25, 2.0, 0.0]
