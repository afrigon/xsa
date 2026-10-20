import struct
from pathlib import Path

import zstandard

from operations.texture_format import TextureFormat

IDENTIFIER = b"\xabKTX 20\xbb\r\n\x1a\n"
HEADER_SIZE = 80
LEVEL_INDEX_ENTRY_SIZE = 24
DESCRIPTOR_BLOCK_HEADER_SIZE = 24
SAMPLE_SIZE = 16
DESCRIPTOR_VERSION = 2
COLOR_PRIMARIES_BT709 = 1
SUPERCOMPRESSION_ZSTANDARD = 2
ZSTANDARD_LEVEL = 19
BLOCK_TYPE_SIZE = 1
WORD_BYTES = 4
WRITER_KEY = "KTXwriter"
WRITER = "base-surfaces scripts"


# Writes texture arrays: every level holds each layer in turn.
class Ktx2Writer:
    def __init__(self, texture_format: TextureFormat):
        self.format = texture_format

    # levels[level][layer] holds the encoded bytes of one layer of one mip level, largest level first. Metadata values
    # are text, written ending with a NUL.
    def write(self, path: Path, width: int, height: int, levels: list[list[bytes]], metadata: dict[str, str]) -> None:
        layer_count = len(levels[0])
        compressor = zstandard.ZstdCompressor(level=ZSTANDARD_LEVEL, threads=-1)
        raw_levels = []

        for level, layers in enumerate(levels):
            expected = self.format.level_size(max(width >> level, 1), max(height >> level, 1))

            if any(len(layer) != expected for layer in layers):
                raise ValueError(f"level {level}: expected {expected} bytes per layer")

            raw_levels.append(b"".join(layers))

        compressed = [compressor.compress(level) for level in raw_levels]
        descriptor = self._descriptor()
        key_values = self._key_values({WRITER_KEY: WRITER} | metadata)
        descriptor_offset = HEADER_SIZE + LEVEL_INDEX_ENTRY_SIZE * len(levels)
        key_values_offset = descriptor_offset + len(descriptor)
        data_offset = key_values_offset + len(key_values)

        # Levels are stored smallest first, as the specification recommends.
        offsets = [0] * len(levels)
        position = data_offset

        for level in reversed(range(len(levels))):
            offsets[level] = position
            position += len(compressed[level])

        header = IDENTIFIER + struct.pack(
            "<9I4I2Q",
            self.format.value.vk_format,
            BLOCK_TYPE_SIZE,
            width,
            height,
            0,
            layer_count,
            1,
            len(levels),
            SUPERCOMPRESSION_ZSTANDARD,
            descriptor_offset,
            len(descriptor),
            key_values_offset,
            len(key_values),
            0,
            0,
        )
        index = b"".join(
            struct.pack("<3Q", offsets[level], len(compressed[level]), len(raw_levels[level]))
            for level in range(len(levels))
        )

        path.parent.mkdir(parents=True, exist_ok=True)

        with path.open("wb") as file:
            file.write(header + index + descriptor + key_values)

            for level in reversed(range(len(levels))):
                file.write(compressed[level])

    def _descriptor(self) -> bytes:
        description = self.format.value
        block_size = DESCRIPTOR_BLOCK_HEADER_SIZE + SAMPLE_SIZE * len(description.samples)
        dimension = description.block_size - 1
        block = struct.pack(
            "<IHH4B4B8B",
            0,
            DESCRIPTOR_VERSION,
            block_size,
            description.model,
            COLOR_PRIMARIES_BT709,
            description.transfer,
            0,
            dimension,
            dimension,
            0,
            0,
            description.block_bytes,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        )

        for sample in description.samples:
            block += struct.pack(
                "<HBB4BII", sample.bit_offset, sample.bit_length - 1, sample.channel, 0, 0, 0, 0, sample.lower, sample.upper
            )

        return struct.pack("<I", 4 + len(block)) + block

    # The specification asks for entries sorted by key, each padded to a multiple of 4 bytes.
    @staticmethod
    def _key_values(metadata: dict[str, str]) -> bytes:
        data = b""

        for key in sorted(metadata):
            entry = key.encode() + b"\0" + metadata[key].encode() + b"\0"
            data += struct.pack("<I", len(entry)) + entry + b"\0" * (-len(entry) % WORD_BYTES)

        return data
