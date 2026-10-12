# Sequential ray tracing through a spherical lens prescription, with glass dispersion.
import numpy as np

# Abbe numbers are not in the prescription; these are typical of each index's glass family.
ABBE = {1.54: 59.5, 1.772: 49.6, 1.617: 54.9, 1.713: 53.8, 1.805: 25.4}

class Surface:
    def __init__(self, radius, z, index_after, semi_aperture, is_stop):
        self.radius = radius
        self.z = z
        self.index_after = index_after
        self.semi_aperture = semi_aperture
        self.is_stop = is_stop

def glass_index(n_d, wavelength):
    """Two-term Cauchy fit through n_d at 587.6 nm and the Abbe number (n_F at 486.1, n_C at 656.3)."""
    if n_d == 1.0:
        return 1.0
    abbe = ABBE[n_d]
    b = (n_d - 1) / abbe / (1 / 486.1 ** 2 - 1 / 656.3 ** 2)
    a = n_d - b / 587.6 ** 2
    return a + b / wavelength ** 2

def load(path):
    surfaces = []
    z = 0.0
    for line in open(path):
        if line.startswith("#") or not line.strip():
            continue
        radius, thickness, index, aperture = map(float, line.split()[:4])
        is_stop = radius == 0 and index == 0
        surfaces.append(Surface(radius, z, 1.0 if is_stop else index, aperture / 2, is_stop))
        z += thickness
    return surfaces

def intersect(surface, origin, direction):
    """Returns (point, normal facing against +z) for each ray, NaN where the ray misses."""
    if surface.radius == 0:
        t = (surface.z - origin[:, 2]) / direction[:, 2]
        point = origin + direction * t[:, None]
        normal = np.zeros_like(point)
        normal[:, 2] = -1
        return point, normal
    center = np.array([0, 0, surface.z + surface.radius])
    oc = origin - center
    b = np.einsum("ij,ij->i", oc, direction)
    c = np.einsum("ij,ij->i", oc, oc) - surface.radius ** 2
    disc = b * b - c
    sq = np.sqrt(np.where(disc >= 0, disc, np.nan))
    # The cap facing the vertex: nearer root for a center behind the vertex, and vice versa, for rays going +z;
    # pick the root whose point lies on the vertex's side of the sphere.
    t1, t2 = -b - sq, -b + sq
    p1 = origin + direction * t1[:, None]
    use_first = np.sign(surface.radius) * (p1[:, 2] - center[2]) < 0
    t = np.where(use_first, t1, t2)
    point = origin + direction * t[:, None]
    normal = (point - center) / surface.radius
    normal = normal * -np.sign(normal[:, 2:3])
    return point, normal

def refract(direction, normal, n1, n2):
    cos_i = -np.einsum("ij,ij->i", direction, normal)
    flip = cos_i < 0
    normal = np.where(flip[:, None], -normal, normal)
    cos_i = np.abs(cos_i)
    eta = n1 / n2
    k = 1 - eta ** 2 * (1 - cos_i ** 2)
    k = np.where(k >= 0, k, np.nan)
    return eta * direction + (eta * cos_i - np.sqrt(k))[:, None] * normal, cos_i

def reflect(direction, normal):
    cos_i = -np.einsum("ij,ij->i", direction, normal)
    return direction + 2 * cos_i[:, None] * normal, np.abs(cos_i)

def index_before(surfaces, i, wavelength):
    return 1.0 if i == 0 else glass_index(surfaces[i - 1].index_after, wavelength)

