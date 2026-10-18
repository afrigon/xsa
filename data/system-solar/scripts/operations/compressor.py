import numpy as np

from operations.texture_format import TextureFormat

# The type each format stores its texels as.
TEXEL_TYPES = {
    TextureFormat.RGBA8_SRGB: np.uint8,
    TextureFormat.RG8_UNORM: np.uint8,
    TextureFormat.RGBA16_SFLOAT: np.float16,
    TextureFormat.R32_SFLOAT: np.float32,
}


class Compressor:
    def __init__(self, texture_format: TextureFormat):
        self.format = texture_format

    # pixels: height × width × channels, already in the format's range.
    def compress(self, pixels: np.ndarray) -> bytes:
        return np.ascontiguousarray(pixels, TEXEL_TYPES[self.format]).tobytes()
