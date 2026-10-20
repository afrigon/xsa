import numpy as np


# A full mip chain, each level halving the one above rounded down, as Vulkan sizes them, each texel the exact area
# average of the texels it covers: means, and the slope statistics computed from them, carry through every level.
class MipChain:
    def __init__(self, pixels: np.ndarray):
        self.levels = [pixels]

        while self.levels[-1].shape[0] > 1 or self.levels[-1].shape[1] > 1:
            level = self.levels[-1]
            level = MipChain._area_resample(level, 0, max(level.shape[0] // 2, 1))
            self.levels.append(MipChain._area_resample(level, 1, max(level.shape[1] // 2, 1)))

    @staticmethod
    def _area_resample(values: np.ndarray, axis: int, size: int) -> np.ndarray:
        count = values.shape[axis]

        if size == count:
            return values

        values = np.moveaxis(values, axis, 0).astype(np.float64)
        prefix = np.concatenate([np.zeros((1,) + values.shape[1:]), np.cumsum(values, axis=0)])
        positions = np.arange(size + 1) * (count / size)
        whole = np.minimum(np.floor(positions).astype(int), count - 1)
        fraction = (positions - whole).reshape((-1,) + (1,) * (values.ndim - 1))
        integral = prefix[whole] + fraction * values[whole]
        return np.moveaxis((integral[1:] - integral[:-1]) * (size / count), 0, axis).astype(np.float32)
