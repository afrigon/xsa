import etcpak
import numpy as np

from operations.texture_format import TextureFormat

# bc7enc's most thorough search: every mode, all 64 partitions, least-squares endpoint refinement.
BC7_UBER_LEVEL = 4
BC7_MAX_PARTITIONS = 64


class Compressor:
    def __init__(self, texture_format: TextureFormat):
        self.format = texture_format

    # pixels: height × width × 4 bytes.
    def compress(self, pixels: np.ndarray) -> bytes:
        padded = self._padded(pixels)
        height, width = padded.shape[:2]
        data = padded.tobytes()

        match self.format:
            case TextureFormat.BC4_UNORM:
                return etcpak.compress_bc4(data, width, height)
            case TextureFormat.BC5_UNORM:
                return etcpak.compress_bc5(data, width, height)
            case TextureFormat.BC7_SRGB:
                return etcpak.compress_bc7(data, width, height, self._bc7_parameters())

        raise ValueError(f"no compressor for {self.format}")

    @staticmethod
    def _bc7_parameters() -> etcpak.BC7CompressBlockParams:
        parameters = etcpak.BC7CompressBlockParams()
        parameters.init_perceptual_weights()
        parameters.m_uber_level = BC7_UBER_LEVEL
        parameters.m_max_partitions = BC7_MAX_PARTITIONS
        parameters.m_try_least_squares = True
        return parameters

    def _padded(self, pixels: np.ndarray) -> np.ndarray:
        size = self.format.value.block_size
        height, width = pixels.shape[:2]
        padding = ((0, -height % size), (0, -width % size), (0, 0))
        return np.ascontiguousarray(np.pad(pixels, padding, mode="wrap"))
