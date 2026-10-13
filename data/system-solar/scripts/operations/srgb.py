import numpy as np

LINEAR_THRESHOLD = 0.0031308
ENCODED_THRESHOLD = 0.04045
LINEAR_SCALE = 12.92
GAMMA = 2.4
SCALE = 1.055
OFFSET = 0.055


def to_linear(encoded: np.ndarray) -> np.ndarray:
    return np.where(encoded <= ENCODED_THRESHOLD, encoded / LINEAR_SCALE, ((encoded + OFFSET) / SCALE) ** GAMMA)


def to_srgb(linear: np.ndarray) -> np.ndarray:
    linear = np.clip(linear, 0.0, 1.0)
    return np.where(linear <= LINEAR_THRESHOLD, linear * LINEAR_SCALE, SCALE * linear ** (1.0 / GAMMA) - OFFSET)
