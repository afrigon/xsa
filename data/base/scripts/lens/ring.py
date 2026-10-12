# The ring halo, an artistic element shaped after Sol's Ghost1 but round to suit a spherical lens, stored as luminance
# per unit illuminance (1/sr) and fading to exactly zero inside the texture.
import numpy as np
from blur import gaussian_blur
import quality

PEAK = 0.0158
FADE_START = 0.85
FADE_END = 0.95

SIZE = 1024
y, x = (np.mgrid[0:SIZE, 0:SIZE] + 0.5) / SIZE * 2 - 1
radius = np.hypot(x, y)
angle = np.arctan2(y, x)
from_horizontal = np.minimum(np.abs(angle), np.pi - np.abs(angle))

# Fine, low-contrast streaks: smooth periodic noise with many cells around the circle, two octaves.
def periodic_noise(cells, seed):
    values = np.random.default_rng(seed).normal(0, 1, cells)
    position = (angle + np.pi) / (2 * np.pi) * cells
    cell = np.floor(position).astype(int) % cells
    fraction = position - np.floor(position)
    fraction = fraction * fraction * (3 - 2 * fraction)
    return values[cell] * (1 - fraction) + values[(cell + 1) % cells] * fraction
streaks = 1 + 0.25 * periodic_noise(720, 1) + 0.15 * periodic_noise(160, 2)

# Brightness around the ring: a faint band everywhere, swelling into the side highlights.
swell = 0.75 * np.exp(-(from_horizontal / np.radians(6)) ** 2) + 0.25 * np.exp(-from_horizontal / np.radians(18))
band_width = 0.17
# Red sits inside, blue outside.
channel_radii = np.array([0.685, 0.72, 0.75])
channel_gains = np.array([0.85, 1.0, 1.1])
image = np.zeros((SIZE, SIZE, 3))
# A soft band with long tails, plus faint streaks smeared across it, then the whole texture blurred.
streak_smear = 0.9 * (streaks - 1)
for channel in range(3):
    distance = (radius - channel_radii[channel]) / (band_width * 1.3)
    band = 1 / (1 + distance ** 2) ** 3
    highlight = np.exp(-((radius - channel_radii[channel]) / (0.07 * 1.3)) ** 2)
    image[..., channel] = channel_gains[channel] * (0.07 * band + 0.6 * swell * highlight) * (1 + streak_smear)
image = np.clip(gaussian_blur(image, SIZE * 0.003), 0, None)

window = np.clip((FADE_END - radius) / (FADE_END - FADE_START), 0, 1)
window = window * window * (3 - 2 * window)
ring = image / image.max() * PEAK * window[..., None]
np.save(quality.ring_file(), ring.astype(np.float32))
