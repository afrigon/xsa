from collections.abc import Callable
from pathlib import Path

import numpy as np

from operations import srgb
from operations.compressor import Compressor
from operations.ktx2_writer import Ktx2Writer
from operations.mip_chain import MipChain
from operations.texture_format import TextureFormat

UNORM8_MAX = 255


# Writes a texture array from each layer's mip chain, each level encoded to bytes then compressed.
class ArrayWriter:
    def __init__(self, texture_format: TextureFormat, encode: Callable[[np.ndarray], np.ndarray]):
        self.format = texture_format
        self.encode = encode

    # Linear colour, stored sRGB.
    @staticmethod
    def color() -> "ArrayWriter":
        return ArrayWriter(TextureFormat.BC7_SRGB, lambda linear: ArrayWriter._rgba(srgb.to_srgb(linear)))

    # Slopes, stored as the x and y of the normal they tilt, z reconstructed.
    @staticmethod
    def normal() -> "ArrayWriter":
        return ArrayWriter(TextureFormat.BC5_UNORM, ArrayWriter._normal)

    # Displacement from 0 to 1.
    @staticmethod
    def displacement() -> "ArrayWriter":
        return ArrayWriter(TextureFormat.BC4_UNORM, ArrayWriter._rgba)

    def write(self, path: Path, chains: list[MipChain], metadata: dict[str, str]) -> None:
        compressor = Compressor(self.format)
        levels = [
            [compressor.compress(self.encode(chain.levels[level])) for chain in chains]
            for level in range(len(chains[0].levels))
        ]
        height, width = chains[0].levels[0].shape[:2]
        Ktx2Writer(self.format).write(path, width, height, levels, metadata)

    @staticmethod
    def _normal(slopes: np.ndarray) -> np.ndarray:
        length = np.sqrt(1.0 + (slopes**2).sum(axis=-1, keepdims=True))
        return ArrayWriter._rgba(-slopes / length * 0.5 + 0.5)

    @staticmethod
    def _rgba(unit: np.ndarray) -> np.ndarray:
        height, width, channels = unit.shape
        rgba = np.zeros((height, width, 4), np.uint8)
        rgba[..., :channels] = np.round(np.clip(unit, 0.0, 1.0) * UNORM8_MAX)
        rgba[..., 3] = UNORM8_MAX
        return rgba
