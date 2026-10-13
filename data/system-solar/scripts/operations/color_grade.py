import numpy as np

from operations.raster import Raster

# Relative luminance of linear sRGB.
LUMINANCE_WEIGHTS = np.array([0.2126, 0.7152, 0.0722])


# An artistic grade on a linear sRGB map: each pixel keeps its luminance, and its distance from the neutral gray of
# that luminance is scaled by the saturation, so the pattern and relative colors stay while the overall tint changes.
class ColorGrade:
    def __init__(self, saturation: float):
        self.saturation = saturation

    def apply(self, color: Raster) -> Raster:
        luminance = (color.pixels @ LUMINANCE_WEIGHTS)[..., None]
        return Raster((luminance + (color.pixels - luminance) * self.saturation).astype(np.float32))
