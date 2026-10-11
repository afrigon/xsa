# Exports the baked lens to the base pack: half-float DDS textures and the main lens profile that uses them. Every
# value in the profile comes from here or the bake, so a rebake reproduces it.
import glob
import os
import struct
import numpy as np
import design
import quality
from blur import disk_blur
from starburst import HALF_EXTENT_DEGREES, LOGPOLAR_INNER_DEGREES, LOGPOLAR_EXTENT_DEGREES

RESOURCES = os.path.join(quality.SCRIPTS, "../../base/resources")
TEXTURE_FOLDER = "lens"
OUTPUT = os.path.normpath(os.path.join(RESOURCES, "textures", TEXTURE_FOLDER))
PROFILE = os.path.normpath(os.path.join(RESOURCES, "lenses", "main.kdl"))
FOCAL = design.FOCAL
BIN = design.BIN
SENSOR_HALF = design.SENSOR_HALF
GHOST_TEXTURE = 512
# Ghosts are kept when their peak, for the Sun at 1 AU at the game's exposure there, reaches this exposed level.
SUN_ILLUMINANCE = 1.28e5
SUN_EXPOSURE = 1 / (1.2 * 2 ** 15)
MINIMUM_PEAK_EXPOSED = 0.01
MAXIMUM_GHOSTS = 8
# Relative level below which the FFT disk blur leaves only round-off, not light.
FFT_ROUND_OFF = 1e-6
# Stylized chromatic split written into the profile: for a light at the reference angle, each ghost's red moves
# outward and blue inward along the axis by this fraction of the ghost's half-brightness width; it grows with the angle.
CHROMATIC_SPLIT = 0.35
SPLIT_REFERENCE_DEGREES = 10.0
# Stylized tint written into the profile: the blue stripe holds its own against red and green.
GHOST_TINT = (1.0, 1.0, 1.3)
# Off-axis falloff written into the profile: cos^exponent, gentler than an ideal lens's 4, like a wide-angle design.
OFF_AXIS_EXPONENT = 2.0
# Light scattered by the lens's surfaces: SCATTER_TOTAL of the light spread within 60 degrees, falling off as
# 1 / (core^exponent + angle^exponent).
SCATTER_EXPONENT = 2.2
SCATTER_CORE = np.radians(0.3)
SCATTER_TOTAL = 0.0075
SCATTER_REACH = np.radians(60)
# The ring halo, an artistic ghost: its position on the axis and its size as a fraction of the screen's height.
RING_POSITION = 0.2
RING_SIZE = 0.4
# The game's screen spans 2 tan(vertical FOV / 2) in tangent units: 1080 px at 805.5 px per unit.
SCREEN_HEIGHT_TANGENT = 1080 / 805.5
DXGI_FORMAT_R16G16B16A16_FLOAT = 10

def scatter_constant():
    theta = np.linspace(0, SCATTER_REACH, 200000)
    shape = 1 / (SCATTER_CORE ** SCATTER_EXPONENT + theta ** SCATTER_EXPONENT)
    return SCATTER_TOTAL / np.trapezoid(shape * 2 * np.pi * np.sin(theta), theta)

def sensor_coordinates(bins):
    return (np.arange(bins) + 0.5) * BIN - SENSOR_HALF

def blurred_ghosts():
    """Every ghost blurred by its image of the Sun's disk, keyed by surface pair, and the light image's height."""
    npz = np.load(quality.ghosts_file())
    coords = sensor_coordinates(npz["direct"].shape[0])
    direct = npz["direct"].sum(2)
    star_y = (direct.sum(1) * coords).sum() / direct.sum()
    sun_on_sensor = FOCAL * np.tan(np.radians(design.SUN_ANGULAR_RADIUS_DEGREES)) / BIN
    ghosts = {}
    for key in npz.files:
        if key == "direct":
            continue
        image = npz[key]
        total = image.sum(2)
        if total.sum() <= 0:
            continue
        # A ghost moves |y_g / y_s| times as fast as the light's image, so it images the disk at that scale.
        ghost_y = (total.sum(1) * coords).sum() / total.sum()
        ghosts[key] = np.clip(disk_blur(image, abs(ghost_y / star_y) * sun_on_sensor), 0, None)
    return ghosts, star_y

def blurred_starburst():
    starburst = np.load(quality.starburst_file())
    texels_per_degree = starburst.shape[0] / 2 / HALF_EXTENT_DEGREES
    return np.clip(disk_blur(starburst, design.SUN_ANGULAR_RADIUS_DEGREES * texels_per_degree), 0, None)

def write_dds(path, rgb):
    """Writes a linear RGB image as R16G16B16A16_FLOAT with a full mip chain."""
    image = np.concatenate([rgb, np.ones(rgb.shape[:2] + (1,))], -1).astype(np.float32)
    levels = [image]
    while min(levels[-1].shape[:2]) > 1:
        current = levels[-1]
        h, w = current.shape[0] // 2 * 2, current.shape[1] // 2 * 2
        levels.append(current[:h, :w].reshape(h // 2, 2, w // 2, 2, 4).mean((1, 3)))
    height, width = image.shape[:2]
    header = struct.pack("<4s7I44x", b"DDS ", 124, 0x1 | 0x2 | 0x4 | 0x8 | 0x1000 | 0x20000, height, width, width * 8, 0, len(levels))
    pixel_format = struct.pack("<2I4s5I", 32, 0x4, b"DX10", 0, 0, 0, 0, 0)
    caps = struct.pack("<4I4x", 0x1000 | 0x400000 | 0x8, 0, 0, 0)
    dx10 = struct.pack("<5I", DXGI_FORMAT_R16G16B16A16_FLOAT, 3, 0, 1, 0)
    with open(path, "wb") as file:
        file.write(header + pixel_format + caps + dx10)
        for level in levels:
            file.write(np.clip(level, 0, 65000).astype(np.float16).tobytes())

def bilinear(image, rows, cols):
    rows = np.clip(rows, 0, image.shape[0] - 2)
    cols = np.clip(cols, 0, image.shape[1] - 2)
    r0, c0 = np.floor(rows).astype(int), np.floor(cols).astype(int)
    fr, fc = (rows - r0)[..., None], (cols - c0)[..., None]
    return ((image[r0, c0] * (1 - fc) + image[r0, c0 + 1] * fc) * (1 - fr)
            + (image[r0 + 1, c0] * (1 - fc) + image[r0 + 1, c0 + 1] * fc) * fr)

# The fine square texture covers the inner angles in detail and the far log-polar part the rest; between, both are
# valid and the two blend.
BLEND_START_DEGREES = 18.0
BLEND_END_DEGREES = 20.0

def log_polar_starburst():
    """The starburst on the game's log-polar grid: rows are angles from the light on a log scale, columns
    directions around it."""
    fine = blurred_starburst()
    far = np.load(quality.starburst_far_file())
    height, width = far.shape[:2]
    rows = (np.arange(height) + 0.5) / height
    angles = LOGPOLAR_INNER_DEGREES * (LOGPOLAR_EXTENT_DEGREES / LOGPOLAR_INNER_DEGREES) ** rows
    directions = (np.arange(width) + 0.5) / width * 2 * np.pi
    phi, degrees = np.meshgrid(directions, angles)
    size = fine.shape[0]
    radius = np.minimum(degrees, HALF_EXTENT_DEGREES) / HALF_EXTENT_DEGREES * size / 2
    sampled = bilinear(fine, size / 2 - 0.5 + radius * np.sin(phi), size / 2 - 0.5 + radius * np.cos(phi))
    blend = np.clip((degrees - BLEND_START_DEGREES) / (BLEND_END_DEGREES - BLEND_START_DEGREES), 0, 1)[..., None]
    return sampled * (1 - blend) + far * blend

def ghost_entries(ghosts, star_y):
    coords = sensor_coordinates(next(iter(ghosts.values())).shape[0])
    candidates = []
    for key, image in ghosts.items():
        peak = image.max() * SUN_ILLUMINANCE * SUN_EXPOSURE
        if peak >= MINIMUM_PEAK_EXPOSED:
            candidates.append((peak, key, image))
    entries = []
    for peak, key, image in sorted(candidates, reverse=True)[:MAXIMUM_GHOSTS]:
        profile = image.sum(2).sum(1)
        bright = coords[profile > profile.max() * 0.5]
        shift = CHROMATIC_SPLIT * (bright.max() - bright.min())
        total = image.sum(2)
        center_y = (total.sum(1) * coords).sum() / total.sum()
        # The crop holds the ghost's whole extent, so its texture is black at the edges: the light ends where only the
        # disk blur's numerical round-off remains.
        lit = total > total.max() * FFT_ROUND_OFF
        distance = np.maximum(np.abs(coords[:, None] - center_y), np.abs(coords[None, :]))
        # Two bins of margin, so even the edge texels blending with their neighbors read only black bins.
        half = distance[lit].max() + 2 * BIN
        # Texture u runs along the sensor's +y (toward the star), v across it.
        local = (np.arange(GHOST_TEXTURE) + 0.5) / GHOST_TEXTURE * 2 - 1
        lu, lv = np.meshgrid(local, local)
        rows = (center_y + lu * half + SENSOR_HALF) / BIN - 0.5
        cols = (lv * half + SENSOR_HALF) / BIN - 0.5
        texture = bilinear(image, rows, cols)
        name = f"ghost-{key}"
        # Stored at full brightness so it reads in an image viewer; the profile's intensity restores its level.
        intensity = texture.max()
        write_dds(os.path.join(OUTPUT, name + ".dds"), texture / intensity)
        position = 1 - center_y / star_y
        size = np.degrees(2 * np.arctan(half / FOCAL))
        dispersion_per_degree = shift / (2 * half) / SPLIT_REFERENCE_DEGREES
        tint = " ".join(f"{component:g}" for component in GHOST_TINT)
        entries.append(f'ghost position={position:.4f} size={size:.4f} dispersion-per-degree={dispersion_per_degree:.5f} intensity={intensity:.6g} {{\n'
                       f'    tint {tint}\n    texture "{TEXTURE_FOLDER}/{name}"\n}}')
    return entries

if __name__ == "__main__":
    os.makedirs(OUTPUT, exist_ok=True)
    for stale in glob.glob(os.path.join(OUTPUT, "*.dds")):
        os.remove(stale)
    ghosts, star_y = blurred_ghosts()
    entries = ghost_entries(ghosts, star_y)
    write_dds(os.path.join(OUTPUT, "starburst.dds"), log_polar_starburst())
    ring = np.load(quality.ring_file())
    ring_intensity = ring.max()
    write_dds(os.path.join(OUTPUT, "ring.dds"), ring / ring_intensity)
    ring_degrees = np.degrees(2 * np.arctan(RING_SIZE * SCREEN_HEIGHT_TANGENT / 2))
    entries.append(f'ghost position={RING_POSITION} size={ring_degrees:.4f} intensity={ring_intensity:.6g} {{\n    texture "{TEXTURE_FOLDER}/ring"\n}}')
    # The glare layer draws the scatter as intensity / angle^falloff with the angle in degrees.
    glare_intensity = scatter_constant() * np.degrees(1) ** SCATTER_EXPONENT
    with open(PROFILE, "w") as file:
        file.write(f"off-axis-falloff exponent={OFF_AXIS_EXPONENT:g}\n\n")
        file.write(f"glare intensity={glare_intensity:.4f} falloff={SCATTER_EXPONENT}\n\n")
        file.write(f'starburst texture="{TEXTURE_FOLDER}/starburst" inner={LOGPOLAR_INNER_DEGREES:g} extent={LOGPOLAR_EXTENT_DEGREES:g}\n\n')
        file.write("\n\n".join(entries) + "\n")
    print(f"wrote {PROFILE}")
