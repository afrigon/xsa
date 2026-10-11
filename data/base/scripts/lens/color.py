# CIE 1931 colour matching (Wyman, Sloan and Shirley's multi-lobe fit) and conversion to linear sRGB.
import numpy as np

def _lobe(x, mean, below, above):
    t = (x - mean) * np.where(x < mean, below, above)
    return np.exp(-0.5 * t * t)

def xyz(wavelength):
    x = 1.056 * _lobe(wavelength, 599.8, 0.0264, 0.0323) + 0.362 * _lobe(wavelength, 442.0, 0.0624, 0.0374) \
        - 0.065 * _lobe(wavelength, 501.1, 0.0490, 0.0382)
    y = 0.821 * _lobe(wavelength, 568.8, 0.0213, 0.0247) + 0.286 * _lobe(wavelength, 530.9, 0.0613, 0.0322)
    z = 1.217 * _lobe(wavelength, 437.0, 0.0845, 0.0278) + 0.681 * _lobe(wavelength, 459.0, 0.0385, 0.0725)
    return np.array([x, y, z])

XYZ_TO_SRGB = np.array([[3.2406, -1.5372, -0.4986], [-0.9689, 1.8758, 0.0415], [0.0557, -0.2040, 1.0570]])

def rgb_weights(wavelengths):
    """Per-wavelength linear sRGB weights for a flat spectrum, normalized so white sums to (1, 1, 1)."""
    weights = np.array([XYZ_TO_SRGB @ xyz(l) for l in wavelengths])
    return weights / weights.sum(0)
