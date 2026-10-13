from collections.abc import Callable
from pathlib import Path

import numpy as np

from operations import srgb
from operations.compressor import Compressor
from operations.ktx2_writer import Ktx2Writer
from operations.raster import Raster
from operations.texture_format import TextureFormat

UNORM8_MAX = 255


class TextureWriter:
    def __init__(self, texture_format: TextureFormat, encode: Callable[[np.ndarray], np.ndarray]):
        self.format = texture_format
        self.encode = encode

    @staticmethod
    def color() -> "TextureWriter":
        return TextureWriter(TextureFormat.BC7_SRGB, TextureWriter._srgb)

    @staticmethod
    def normal() -> "TextureWriter":
        return TextureWriter(TextureFormat.BC5_UNORM, TextureWriter._signed_pair)

    @staticmethod
    def sky() -> "TextureWriter":
        return TextureWriter(TextureFormat.RGBA8_SRGB, TextureWriter._srgb)

    # Physical values stored as they are, up to four per texel.
    @staticmethod
    def values() -> "TextureWriter":
        return TextureWriter(TextureFormat.RGBA16_SFLOAT, TextureWriter._four_channels)

    # One face for a 2D texture, six for a cube map.
    def write(self, path: Path, faces: list[Raster]) -> None:
        compressor = Compressor(self.format)
        chains = [face.mipmaps() for face in faces]
        levels = [[compressor.compress(self.encode(chain[level])) for chain in chains] for level in range(len(chains[0]))]
        Ktx2Writer(self.format).write(path, faces[0].width, faces[0].height, levels)

    @staticmethod
    def _srgb(linear: np.ndarray) -> np.ndarray:
        return TextureWriter._rgba(srgb.to_srgb(linear))

    @staticmethod
    def _signed_pair(values: np.ndarray) -> np.ndarray:
        return TextureWriter._rgba(values * 0.5 + 0.5)

    @staticmethod
    def _four_channels(values: np.ndarray) -> np.ndarray:
        height, width, channels = values.shape
        return np.concatenate([values, np.zeros((height, width, 4 - channels), values.dtype)], axis=-1)

    @staticmethod
    def _rgba(unit: np.ndarray) -> np.ndarray:
        height, width, channels = unit.shape
        rgba = np.zeros((height, width, 4), np.uint8)
        rgba[..., :channels] = np.round(np.clip(unit, 0.0, 1.0) * UNORM8_MAX)
        rgba[..., 3] = UNORM8_MAX
        return rgba
