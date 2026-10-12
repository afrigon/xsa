# Far-field diffraction of the entrance pupil (blades, edge roughness, dust, scratches), averaged over texture pixels:
# the pupil is cut into patches with power-complementary windows whose intensities add, which leaves the pattern
# averaged over the patch's angular resolution, as a texture pixel integrates it.
import time
from functools import partial
from multiprocessing import Pool
import numpy as np
import aperture
import color
import quality
import design

PUPIL_RADIUS = design.FOCAL / design.F_NUMBER / 2 * 1e-3  # m, circumradius of the blade opening seen from the front
SAMPLE = 0.5e-6                              # m per pupil sample
PATCH = quality.STARBURST_PATCH                # samples per patch side
STEP = PATCH // 2
WAVELENGTHS = quality.STARBURST_WAVELENGTHS
HALF_EXTENT_DEGREES = 20.0
HALF_EXTENT = np.radians(HALF_EXTENT_DEGREES)
TEXTURE = 2048
FAR_PATCH = quality.STARBURST_FAR_PATCH
FAR_SAMPLE = 0.2e-6                         # m per pupil sample: fine enough to reach 90 degrees at 400 nm
FAR_START_DEGREES = 16.0
LOGPOLAR_WIDTH = 2048
LOGPOLAR_HEIGHT = 1024
LOGPOLAR_INNER_DEGREES = 0.01
LOGPOLAR_EXTENT_DEGREES = 90.0
rng = np.random.default_rng(3)

# Blade edge roughness at many scales, as on machined edges.
ROUGHNESS = 0.5e-6
ROUGHNESS_LONGEST = 1e-3
ROUGHNESS_SHORTEST = 1e-6
edge_length = 2 * PUPIL_RADIUS * np.sin(np.pi / aperture.BLADES) * 1.2
noise_samples = int(edge_length / SAMPLE)
roughness = []
for blade in range(aperture.BLADES):
    white = rng.normal(0, 1, noise_samples)
    frequencies = np.fft.rfftfreq(noise_samples, SAMPLE)
    band = (frequencies >= 1 / ROUGHNESS_LONGEST) & (frequencies <= 1 / ROUGHNESS_SHORTEST)
    # Power falling as 1/f², so amplitude as 1/f.
    shape = np.where(band, 1 / np.maximum(frequencies, 1e-12), 0.0)
    shaped = np.fft.irfft(np.fft.rfft(white) * shape, noise_samples)
    roughness.append(shaped / shaped.std() * ROUGHNESS)

# Each blade sits slightly off its ideal place, angle and curvature.
blade_offsets = rng.normal(0, 0.03, aperture.BLADES)
blade_tilts = rng.normal(0, np.radians(1.5), aperture.BLADES)
blade_curvatures = 1 + rng.normal(0, 0.1, aperture.BLADES)

SCRATCHES = 3
scratches = []
for _ in range(SCRATCHES):
    scratches.append((rng.uniform(-0.6, 0.6, 2) * PUPIL_RADIUS, rng.uniform(0, np.pi), rng.uniform(0.5e-3, 1.5e-3)))
SCRATCH_WIDTH = 1e-6
SCRATCH_TRANSMISSION = 0.5

DUST_DENSITY = 1.3e6                          # specks per m² of pupil
dust_radii = np.exp(rng.uniform(np.log(1.5e-6), np.log(12.5e-6), int(DUST_DENSITY * np.pi * PUPIL_RADIUS ** 2)))

def pupil(x, y):
    """Amplitude transmission of the pupil at points (m)."""
    open_ = np.ones(x.shape, dtype=bool)
    for blade in range(aperture.BLADES):
        apothem = PUPIL_RADIUS * np.cos(np.pi / aperture.BLADES) * (1 + blade_offsets[blade])
        arc_radius = aperture.BLADE_CURVATURE_RADIUS * PUPIL_RADIUS * blade_curvatures[blade]
        normal = aperture.ROTATION + np.pi / aperture.BLADES + blade * 2 * np.pi / aperture.BLADES + blade_tilts[blade]
        center = (apothem - arc_radius) * np.array([np.cos(normal), np.sin(normal)])
        dx, dy = x - center[0], y - center[1]
        distance = np.hypot(dx, dy)
        along = np.arctan2(dy, dx) - normal
        along = (along + np.pi) % (2 * np.pi) - np.pi
        index = np.clip((along * arc_radius / SAMPLE + noise_samples / 2).astype(int), 0, noise_samples - 1)
        open_ &= distance < arc_radius + roughness[blade][index]
    transmission = open_.astype(np.float64)
    for center, angle, length in scratches:
        across = (x - center[0]) * np.sin(angle) - (y - center[1]) * np.cos(angle)
        along = (x - center[0]) * np.cos(angle) + (y - center[1]) * np.sin(angle)
        transmission[(np.abs(across) < SCRATCH_WIDTH / 2) & (np.abs(along) < length / 2)] *= SCRATCH_TRANSMISSION
    return transmission

def patch_power(center, patch, sample):
    """Squared Fourier transform of one windowed pupil patch."""
    window_1d = np.sin(np.pi * (np.arange(patch) + 0.5) / patch)
    window = np.outer(window_1d, window_1d)
    offsets = (np.arange(patch) - patch / 2) * sample
    cx, cy = center
    x, y = np.meshgrid(cx + offsets, cy + offsets)
    spectrum = np.fft.fftshift(np.fft.fft2(pupil(x, y) * window)) * sample ** 2
    return np.abs(spectrum) ** 2

# The opening's actual extremes: blades displaced inward or outward, plus a margin for their edge roughness.
EDGE_MARGIN = 5 * ROUGHNESS
inner_radius = PUPIL_RADIUS * np.cos(np.pi / aperture.BLADES) * (1 + blade_offsets.min()) - EDGE_MARGIN
outer_radius = PUPIL_RADIUS * (1 + blade_offsets.max()) + EDGE_MARGIN

def patch_centers(patch, sample):
    """Patches straddling the blade edges or holding a scratch; fully open patches add nothing beyond the core."""
    half_patch = patch / 2 * sample
    extent = PUPIL_RADIUS + half_patch
    grid = np.arange(-extent, extent + 1e-12, patch / 2 * sample)

    def straddles_edge(cx, cy):
        nearest = np.hypot(max(abs(cx) - half_patch, 0), max(abs(cy) - half_patch, 0))
        farthest = np.hypot(abs(cx) + half_patch, abs(cy) + half_patch)
        return nearest < outer_radius and farthest > inner_radius

    def holds_scratch(cx, cy):
        for center, angle, length in scratches:
            ends = [center + side * length / 2 * np.array([np.cos(angle), np.sin(angle)]) for side in (-1, 1)]
            samples = [ends[0] + (ends[1] - ends[0]) * t for t in np.linspace(0, 1, 64)]
            if any(abs(p[0] - cx) < half_patch and abs(p[1] - cy) < half_patch for p in samples):
                return True
        return False

    return [(cx, cy) for cy in grid for cx in grid if straddles_edge(cx, cy) or holds_scratch(cx, cy)]

def frequency_psf(patch, sample, label):
    """The pattern per unit spatial frequency area, normalized to the transmitted power, and its frequency step."""
    centers = patch_centers(patch, sample)
    start = time.time()
    power = np.zeros((patch, patch))
    # Each worker holds a few full patches in memory, so fewer workers than cores keep it out of swap.
    with Pool(6) as pool:
        for result in pool.imap_unordered(partial(patch_power, patch=patch, sample=sample), centers):
            power += result
    print(f"{label}: {len(centers)} patches in {time.time() - start:.0f}s", flush=True)
    return power / open_area(), 1 / (patch * sample)

def open_area():
    coarse = np.linspace(-PUPIL_RADIUS, PUPIL_RADIUS, 4000)
    x, y = np.meshgrid(coarse, coarse)
    return pupil(x, y).mean() * (2 * PUPIL_RADIUS) ** 2

def bessel_j1(x):
    tau = (np.arange(256) + 0.5) / 256 * np.pi
    return np.mean(np.cos(tau[None, :] - x.ravel()[:, None] * np.sin(tau[None, :])), 1).reshape(x.shape)

def dust_profile(radial, wavelength, total_area):
    """Dust scatter along one radius (luminance per unit illuminance): by Babinet's principle each opaque speck
    scatters as a hole of its size, an Airy pattern."""
    k_sin = 2 * np.pi / wavelength * np.sin(radial)
    profile = np.zeros_like(radial)
    for radius in dust_radii:
        x = np.maximum(k_sin * radius, 1e-9)
        profile += (np.pi * radius ** 2) ** 2 * (2 * bessel_j1(x) / x) ** 2
    return profile / (wavelength ** 2 * total_area)

if __name__ == "__main__":
    psf_frequency, frequency_step = frequency_psf(PATCH, SAMPLE, "fine")
    total_area = open_area()

    def bilinear(image, rows, cols):
        rows = np.clip(rows, 0, image.shape[0] - 2)
        cols = np.clip(cols, 0, image.shape[1] - 2)
        r0, c0 = np.floor(rows).astype(int), np.floor(cols).astype(int)
        fr, fc = rows - r0, cols - c0
        return ((image[r0, c0] * (1 - fc) + image[r0, c0 + 1] * fc) * (1 - fr)
                + (image[r0 + 1, c0] * (1 - fc) + image[r0 + 1, c0 + 1] * fc) * fr)

    theta = (np.arange(TEXTURE) + 0.5 - TEXTURE / 2) / (TEXTURE / 2) * HALF_EXTENT
    tx, ty = np.meshgrid(theta, theta)
    weights = color.rgb_weights(WAVELENGTHS * 1e9)
    texture = np.zeros((TEXTURE, TEXTURE, 3))
    for wavelength, w in zip(WAVELENGTHS, weights):
        fx = np.sin(tx) / wavelength / frequency_step + PATCH / 2
        fy = np.sin(ty) / wavelength / frequency_step + PATCH / 2
        # Luminance per unit illuminance (1/sr): dΩ = λ² df.
        texture += bilinear(psf_frequency, fy, fx)[..., None] / wavelength ** 2 * w
        # The dust pattern is radial: evaluate it along one radius and interpolate.
        radial = np.linspace(0, HALF_EXTENT * np.sqrt(2), 4000)
        texture += np.interp(np.hypot(tx, ty), radial, dust_profile(radial, wavelength, total_area))[..., None] * w
    np.save(quality.starburst_file(), texture.astype(np.float32))

    # The far part, past FAR_START, sampled finely enough in the pupil to reach 90 degrees, straight onto the
    # log-polar grid the game samples: rows are angles from the light on a log scale, columns directions around it.
    far_psf, far_step = frequency_psf(FAR_PATCH, FAR_SAMPLE, "far")
    rows = (np.arange(LOGPOLAR_HEIGHT) + 0.5) / LOGPOLAR_HEIGHT
    angles = np.radians(LOGPOLAR_INNER_DEGREES) * (LOGPOLAR_EXTENT_DEGREES / LOGPOLAR_INNER_DEGREES) ** rows
    far_rows = angles >= np.radians(FAR_START_DEGREES)
    directions = (np.arange(LOGPOLAR_WIDTH) + 0.5) / LOGPOLAR_WIDTH * 2 * np.pi
    phi, theta = np.meshgrid(directions, angles[far_rows])
    far = np.zeros((LOGPOLAR_HEIGHT, LOGPOLAR_WIDTH, 3))
    for wavelength, w in zip(WAVELENGTHS, weights):
        fx = np.sin(theta) * np.cos(phi) / wavelength / far_step + FAR_PATCH / 2
        fy = np.sin(theta) * np.sin(phi) / wavelength / far_step + FAR_PATCH / 2
        values = bilinear(far_psf, fy, fx) / wavelength ** 2
        values += np.interp(theta, angles, dust_profile(angles, wavelength, total_area))
        far[far_rows] += values[..., None] * w
    np.save(quality.starburst_far_file(), far.astype(np.float32))
