# Traces every two-reflection ghost of the lens for a distant light at the design's field angle, per wavelength.
import time
from multiprocessing import Pool
import numpy as np
import optics
import aperture
import color
import quality
import design

RAYS_ACROSS = quality.GHOST_RAYS_ACROSS
WAVELENGTHS = quality.GHOST_WAVELENGTHS
SENSOR_HALF = design.SENSOR_HALF
BIN = design.BIN

surfaces = design.surfaces
optical = [i for i, s in enumerate(surfaces) if not s.is_stop]
stop_index = next(i for i, s in enumerate(surfaces) if s.is_stop)
FOCAL = design.FOCAL
IMAGE_Z = design.IMAGE_Z

def stop_height_ratio():
    o = np.array([[0, 0.1, -10.0]]); d = np.array([[0, 0, 1.0]])
    for i, s in enumerate(surfaces[:stop_index + 1]):
        p, n = optics.intersect(s, o, d)
        if i < stop_index:
            d, _ = optics.refract(d, n, optics.index_before(surfaces, i, design.DESIGN_WAVELENGTH),
                                  optics.glass_index(s.index_after, design.DESIGN_WAVELENGTH))
        o = p
    return o[0, 1] / 0.1

ENTRANCE_PUPIL = FOCAL / design.F_NUMBER
STOP_RADIUS = ENTRANCE_PUPIL / 2 * stop_height_ratio()

# An idealized broadband coating: the same small reflectance at every wavelength and angle, so ghosts stay neutral.
# Cemented glass-to-glass interfaces carry none and reflect by Fresnel's equations.
COATING_REFLECTANCE = 0.0075

def fresnel(cos_i, n1, n2):
    sin_t = n1 / n2 * np.sqrt(np.clip(1 - cos_i ** 2, 0, 1))
    cos_t = np.sqrt(np.clip(1 - sin_t ** 2, 0, 1))
    rs = ((n1 * cos_i - n2 * cos_t) / (n1 * cos_i + n2 * cos_t)) ** 2
    rp = ((n1 * cos_t - n2 * cos_i) / (n1 * cos_t + n2 * cos_i)) ** 2
    return (rs + rp) / 2

def interface_reflectance(cos_i, n1, n2):
    if n1 > 1.0 and n2 > 1.0:
        return fresnel(cos_i, n1, n2)
    return np.full_like(cos_i, COATING_REFLECTANCE)

def path(first, second):
    """Surface visits for a ghost reflecting at `second` then at `first` (first < second), or the direct image."""
    if first is None:
        return [(k, "forward") for k in range(len(surfaces))]
    visits = [(k, "forward") for k in range(second)] + [(second, "reflect-forward")]
    visits += [(k, "backward") for k in range(second - 1, first, -1)] + [(first, "reflect-backward")]
    visits += [(k, "forward") for k in range(first + 1, len(surfaces))]
    return visits

# Each cell's light is spread over sample points no farther apart than half a bin, so no bin it covers is skipped.
SAMPLES_PER_BIN = 2
MINIMUM_SUBDIVISIONS = 4
MAXIMUM_SUBDIVISIONS = 32

def trace(visits, wavelength, field):
    half = surfaces[0].semi_aperture
    grid = (np.arange(RAYS_ACROSS) + 0.5) / RAYS_ACROSS * 2 * half - half
    gx, gy = np.meshgrid(grid, grid)
    gx, gy = gx.ravel(), gy.ravel()
    count = gx.size
    direction = np.tile([0.0, np.sin(field), np.cos(field)], (count, 1))
    origin = np.stack([gx, gy - np.tan(field) * 10.0, np.full(count, -10.0)], 1)
    cell = (2 * half / RAYS_ACROSS) ** 2 * np.cos(field)
    weight = np.where(gx ** 2 + gy ** 2 < half ** 2, cell, 0.0)
    # Dead rays keep their place in the grid with zero weight, so neighbors still form cells.
    alive = np.nonzero(weight > 0)[0]
    o, d, w = origin[alive], direction[alive], weight[alive]
    for k, kind in visits:
        s = surfaces[k]
        point, normal = optics.intersect(s, o, d)
        blocked = np.hypot(point[:, 0], point[:, 1]) > s.semi_aperture
        if s.is_stop:
            blocked |= ~aperture.inside(point[:, 0], point[:, 1], STOP_RADIUS)
        w = np.where(blocked | np.isnan(point[:, 2]), 0.0, w)
        n_before = optics.index_before(surfaces, k, wavelength)
        n_after = optics.glass_index(s.index_after, wavelength)
        if s.is_stop:
            o = point
            continue
        if kind == "forward":
            d, cos_i = optics.refract(d, normal, n_before, n_after)
            w = w * (1 - interface_reflectance(cos_i, n_before, n_after))
        elif kind == "backward":
            d, cos_i = optics.refract(d, normal, n_after, n_before)
            w = w * (1 - interface_reflectance(cos_i, n_after, n_before))
        elif kind == "reflect-forward":
            d, cos_i = optics.reflect(d, normal)
            w = w * interface_reflectance(cos_i, n_before, n_after)
        else:
            d, cos_i = optics.reflect(d, normal)
            w = w * interface_reflectance(cos_i, n_after, n_before)
        o = point
        w = np.where(np.isfinite(w), w, 0.0)
    t = (IMAGE_Z - o[:, 2]) / d[:, 2]
    hit = o + d * t[:, None]
    hit_x = np.full(count, np.nan)
    hit_y = np.full(count, np.nan)
    flux = np.zeros(count)
    landed = np.isfinite(hit[:, 0]) & (t > 0) & (w > 0)
    hit_x[alive[landed]] = hit[landed, 0]
    hit_y[alive[landed]] = hit[landed, 1]
    flux[alive[landed]] = w[landed]
    hit_x = hit_x.reshape(RAYS_ACROSS, RAYS_ACROSS)
    hit_y = hit_y.reshape(RAYS_ACROSS, RAYS_ACROSS)
    flux = flux.reshape(RAYS_ACROSS, RAYS_ACROSS)
    # Each cell of four landed rays spreads its light over the quad they span, sampled subdivisions² times.
    corners = [(slice(None, -1), slice(None, -1)), (slice(None, -1), slice(1, None)),
               (slice(1, None), slice(None, -1)), (slice(1, None), slice(1, None))]
    cx = [hit_x[c] for c in corners]
    cy = [hit_y[c] for c in corners]
    cf = [flux[c] for c in corners]
    whole = np.all([np.isfinite(x) for x in cx], 0) & np.all([f > 0 for f in cf], 0)
    cx = [x[whole] for x in cx]
    cy = [y[whole] for y in cy]
    cell_flux = sum(f[whole] for f in cf) / 4
    edges = [np.hypot(cx[a] - cx[b], cy[a] - cy[b]) for a, b in [(0, 1), (0, 2), (1, 3), (2, 3)]]
    widest = np.percentile(np.max(edges, 0), 99) if cell_flux.size else 0.0
    subdivisions = int(np.clip(np.ceil(SAMPLES_PER_BIN * widest / BIN), MINIMUM_SUBDIVISIONS, MAXIMUM_SUBDIVISIONS))
    bins = design.SENSOR_BINS
    image = np.zeros((bins, bins))
    steps = (np.arange(subdivisions) + 0.5) / subdivisions
    for u in steps:
        for v in steps:
            px = (cx[0] * (1 - u) + cx[1] * u) * (1 - v) + (cx[2] * (1 - u) + cx[3] * u) * v
            py = (cy[0] * (1 - u) + cy[1] * u) * (1 - v) + (cy[2] * (1 - u) + cy[3] * u) * v
            part, _, _ = np.histogram2d(py, px, bins=bins, range=[[-SENSOR_HALF, SENSOR_HALF], [-SENSOR_HALF, SENSOR_HALF]],
                                        weights=cell_flux / subdivisions ** 2)
            image += part
    # Sensor irradiance per unit star illuminance, as equivalent luminance per unit illuminance (1/sr).
    pupil_area = np.pi * (ENTRANCE_PUPIL / 2) ** 2
    return image / BIN ** 2 * FOCAL ** 2 / pupil_area

def trace_ghost(first, second, field, weights):
    visits = path(first, second)
    rgb = 0
    for wavelength, w in zip(WAVELENGTHS, weights):
        rgb = rgb + trace(visits, wavelength, field)[..., None] * w
    return rgb.astype(np.float32)

if __name__ == "__main__":
    field = np.radians(design.FIELD_DEGREES)
    weights = color.rgb_weights(WAVELENGTHS)
    pairs = [(None, None)] + [(a, b) for ai, a in enumerate(optical) for b in optical[ai + 1:]]
    start = time.time()
    with Pool() as pool:
        traced = pool.starmap(trace_ghost, [(first, second, field, weights) for first, second in pairs])
    results = {}
    for (first, second), rgb in zip(pairs, traced):
        key = "direct" if first is None else f"{first}-{second}"
        results[key] = rgb
    print(f"traced in {time.time() - start:.0f}s", flush=True)
    np.savez_compressed(quality.ghosts_file(), **results)
