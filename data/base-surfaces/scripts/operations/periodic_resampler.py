import numpy as np

LANCZOS_LOBES = 3


# Resamples a tiling image to another size with a Lanczos-3 filter whose taps wrap around the edges, so the result
# tiles as seamlessly as the source.
class PeriodicResampler:
    def __init__(self, width: int, height: int):
        self.width = width
        self.height = height

    # pixels: height × width × channels.
    def resample(self, pixels: np.ndarray) -> np.ndarray:
        rows = self._weights(pixels.shape[0], self.height)
        columns = self._weights(pixels.shape[1], self.width)
        resampled = np.tensordot(rows, pixels.astype(np.float32), axes=(1, 0))
        return np.ascontiguousarray(np.moveaxis(np.tensordot(resampled, columns, axes=(1, 1)), -1, 1))

    # The matrix taking `source` samples to `target`; when shrinking, the kernel widens to the source spacing.
    @staticmethod
    def _weights(source: int, target: int) -> np.ndarray:
        scale = source / target
        support = LANCZOS_LOBES * max(scale, 1.0)
        centers = (np.arange(target) + 0.5) * scale - 0.5
        offsets = np.arange(-int(np.ceil(support)), int(np.ceil(support)) + 1)
        taps = np.floor(centers)[:, None] + offsets[None, :]
        weights = PeriodicResampler._lanczos((taps - centers[:, None]) / max(scale, 1.0))
        weights /= weights.sum(axis=1, keepdims=True)
        matrix = np.zeros((target, source), np.float32)
        np.add.at(matrix, (np.arange(target)[:, None], taps.astype(int) % source), weights)
        return matrix

    @staticmethod
    def _lanczos(x: np.ndarray) -> np.ndarray:
        return np.where(np.abs(x) < LANCZOS_LOBES, np.sinc(x) * np.sinc(x / LANCZOS_LOBES), 0.0)
