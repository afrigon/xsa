import etcpak
import numpy as np

from operations.texture_format import TextureFormat


class Compressor:
    def __init__(self, texture_format: TextureFormat):
        self.format = texture_format

    # pixels: height × width × channels: bytes, 16-bit integers, or floats for a float format.
    def compress(self, pixels: np.ndarray) -> bytes:
        if self.format == TextureFormat.RGBA8_SRGB:
            return np.ascontiguousarray(pixels).tobytes()

        if self.format == TextureFormat.RGBA16_SFLOAT:
            return np.ascontiguousarray(pixels, np.float16).tobytes()

        if self.format == TextureFormat.R16_UNORM:
            return np.ascontiguousarray(pixels, np.uint16).tobytes()

        if self.format == TextureFormat.R32_SFLOAT:
            return np.ascontiguousarray(pixels, np.float32).tobytes()

        padded = self._padded(pixels)
        height, width = padded.shape[:2]
        data = padded.tobytes()

        match self.format:
            case TextureFormat.BC4_UNORM:
                return etcpak.compress_bc4(data, width, height)
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
