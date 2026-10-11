# The lens being baked: a 22 mm wide-angle design (Nakamura, Modern Lens Design p. 360) at f/5.6. Its focal length and
# image plane come from a ray traced close to the axis.
import hashlib
import os
import urllib.request
import numpy as np
import optics
import quality

# The prescription is downloaded when first needed rather than redistributed: pbrt-v4-scenes grants no license for it.
PRESCRIPTION_URL = ("https://raw.githubusercontent.com/mmp/pbrt-v4-scenes/304287dfcf964c1fd15eb0b887c31c437d0e5001/"
                    "lenses/wide.22mm.dat")
PRESCRIPTION_SHA256 = "b453825d9dd26c76831aa034a4f0893d9023b3bdae023a051b39f335957ae3bf"
PRESCRIPTION = os.path.join(quality.WORK, "wide.22mm.dat")
F_NUMBER = 5.6
# Ghosts are traced for a light this far off the axis; each keeps that shape everywhere on screen.
FIELD_DEGREES = 10.0
# The textures are blurred by the Sun's disk as seen from 1 AU.
SUN_ANGULAR_RADIUS_DEGREES = 0.266
NEAR_AXIS_HEIGHT = 1e-3
DESIGN_WAVELENGTH = 587.6
# The sensor is recorded in square bins of this angular size, out to about the design's 38 degree field.
BIN_TANGENT = 0.00099295
SENSOR_BINS = 1600

def download_prescription():
    if os.path.exists(PRESCRIPTION):
        return
    with urllib.request.urlopen(PRESCRIPTION_URL) as response:
        data = response.read()
    digest = hashlib.sha256(data).hexdigest()
    if digest != PRESCRIPTION_SHA256:
        raise RuntimeError(f"{PRESCRIPTION_URL} changed: sha256 {digest}, expected {PRESCRIPTION_SHA256}")
    with open(PRESCRIPTION, "wb") as file:
        file.write(data)

download_prescription()
surfaces = optics.load(PRESCRIPTION)

def paraxial_focus():
    origin = np.array([[0.0, NEAR_AXIS_HEIGHT, -10.0]])
    direction = np.array([[0.0, 0.0, 1.0]])
    for index, surface in enumerate(surfaces):
        point, normal = optics.intersect(surface, origin, direction)
        if not surface.is_stop:
            before = optics.index_before(surfaces, index, DESIGN_WAVELENGTH)
            after = optics.glass_index(surface.index_after, DESIGN_WAVELENGTH)
            direction, _ = optics.refract(direction, normal, before, after)
        origin = point
    focal = NEAR_AXIS_HEIGHT * direction[0, 2] / -direction[0, 1]
    image_z = origin[0, 2] - origin[0, 1] / direction[0, 1] * direction[0, 2]
    return focal, image_z

FOCAL, IMAGE_Z = paraxial_focus()
BIN = FOCAL * BIN_TANGENT
SENSOR_HALF = BIN * SENSOR_BINS / 2
