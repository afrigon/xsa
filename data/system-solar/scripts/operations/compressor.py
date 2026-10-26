import etcpak
import numpy as np

from operations.texture_format import TextureFormat

# The type each uncompressed format stores its texels as.
TEXEL_TYPES = {
    TextureFormat.RGBA8_SRGB: np.uint8,
    TextureFormat.RG16_SFLOAT: np.float16,
    TextureFormat.RGBA16_SFLOAT: np.float16,
    TextureFormat.R16_UNORM: np.uint16,
}


class Compressor:
    def __init__(self, texture_format: TextureFormat):
        self.format = texture_format

    # pixels: height × width × channels, already in the format's range: bytes for a block-compressed format.
    def compress(self, pixels: np.ndarray) -> bytes:
        if self.format in TEXEL_TYPES:
            return np.ascontiguousarray(pixels, TEXEL_TYPES[self.format]).tobytes()

        padded = self._padded(pixels)
        height, width = padded.shape[:2]
        data = padded.tobytes()

        match self.format:
            case TextureFormat.BC5_UNORM:
                return etcpak.compress_bc5(data, width, height)
            case TextureFormat.BC7_SRGB:
                return etcpak.compress_bc7(data, width, height, None)

        raise ValueError(f"no compressor for {self.format}")

    def _padded(self, pixels: np.ndarray) -> np.ndarray:
        size = self.format.value.block_size
        height, width = pixels.shape[:2]
        padding = ((0, -height % size), (0, -width % size), (0, 0))
        return np.ascontiguousarray(np.pad(pixels, padding, mode="edge"))
