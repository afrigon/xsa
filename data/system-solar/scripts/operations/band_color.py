import warnings

import numpy as np

# colour-science warns at import about each optional dependency it lacks; only its tables are used here.
with warnings.catch_warnings(action="ignore"):
    import colour

OBSERVER = "CIE 1931 2 Degree Standard Observer"
ILLUMINANT = "D65"


# Reflectance measured in a few narrow bands, as the linear sRGB color of the surface under daylight: the spectrum
# is interpolated linearly between the band centers and held flat beyond the outer ones.
class BandColor:
    def __init__(self, wavelengths: list[float]):
        cmfs = colour.MSDS_CMFS[OBSERVER]
        illuminant = colour.SDS_ILLUMINANTS[ILLUMINANT].copy().align(cmfs.shape)
        lambdas = cmfs.wavelengths
        weighted = cmfs.values * illuminant.values[:, None]
        white_luminance = weighted[:, 1].sum()
        hats = np.stack([np.interp(lambdas, wavelengths, np.eye(len(wavelengths))[band]) for band in range(len(wavelengths))])
        band_to_xyz = (hats @ weighted / white_luminance).T
        xyz_to_srgb = colour.RGB_COLOURSPACES["sRGB"].matrix_XYZ_to_RGB
        self.matrix = xyz_to_srgb @ band_to_xyz

    # bands: … × band count reflectances, in the order of the wavelengths.
    def linear_srgb(self, bands: np.ndarray) -> np.ndarray:
        return bands @ self.matrix.T
