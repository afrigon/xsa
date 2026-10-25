import struct

import numpy as np
import zstandard

from operations.compressor import Compressor
from operations.ktx2_writer import IDENTIFIER, Ktx2Writer
from operations.texture_format import TextureFormat


def key_values(data: bytes) -> dict[str, bytes]:
    _, _, offset, length = struct.unpack_from("<4I", data, 48)
    entries = {}
    position = offset

    while position < offset + length:
        (size,) = struct.unpack_from("<I", data, position)
        key, value = data[position + 4 : position + 4 + size].split(b"\0", 1)
        entries[key.decode()] = value
        position += 4 + size + (-size % 4)

    return entries


def test_an_array_stores_each_level_layer_after_layer(tmp_path):
    path = tmp_path / "array.ktx2"
    texture_format = TextureFormat.BC4_UNORM
    compressor = Compressor(texture_format)
    sizes = [(8, 8), (4, 4), (2, 2), (1, 1)]
    levels = [
        [compressor.compress(np.full((height, width, 4), layer * 80, np.uint8)) for layer in range(3)]
        for width, height in sizes
    ]
    Ktx2Writer(texture_format).write(path, 8, 8, levels, {})
    data = path.read_bytes()

    assert data[:12] == IDENTIFIER
    vk_format, _, width, height, depth, layers, faces, level_count, _ = struct.unpack_from("<9I", data, 12)
    assert (vk_format, width, height, depth, layers, faces, level_count) == (139, 8, 8, 0, 3, 1, 4)

    for level, (level_width, level_height) in enumerate(sizes):
        offset, length, uncompressed = struct.unpack_from("<3Q", data, 80 + 24 * level)
        raw = zstandard.ZstdDecompressor().decompress(data[offset : offset + length])
        assert len(raw) == uncompressed == 3 * texture_format.level_size(level_width, level_height)
        assert raw == b"".join(levels[level])


def test_metadata_is_text_ending_with_a_nul_sorted_by_key(tmp_path):
    path = tmp_path / "metadata.ktx2"
    block = Compressor(TextureFormat.BC7_SRGB).compress(np.zeros((1, 1, 4), np.uint8))
    Ktx2Writer(TextureFormat.BC7_SRGB).write(path, 1, 1, [[block]], {"xsa.slopeSpread": "0.5 0.25"})
    entries = key_values(path.read_bytes())

    assert list(entries) == ["KTXwriter", "xsa.slopeSpread"]
    assert entries["xsa.slopeSpread"] == b"0.5 0.25\0"
