from dataclasses import dataclass
from pathlib import Path

import numpy as np

from operations.exr_image import ExrImage
from operations.periodic_resampler import PeriodicResampler
from operations.polyhaven_set import PolyhavenSet

COLOR_MAP = "Diffuse"
NORMAL_MAP = "nor_gl"
DISPLACEMENT_MAP = "Displacement"


# One texture set resampled to a surface's size: linear colour, the ground's slopes (height change per unit of
# distance along the image's x and y), and displacement as the source stores it.
@dataclass(frozen=True)
class SurfaceVariant:
    color: np.ndarray
    slopes: np.ndarray
    displacement: np.ndarray

    @staticmethod
    def read(polyhaven_set: PolyhavenSet, pixels: int, cache: Path) -> "SurfaceVariant":
        resampler = PeriodicResampler(pixels, pixels)

        def read_map(map_name: str) -> np.ndarray:
            return ExrImage.read(polyhaven_set.fetch_map(map_name, pixels, cache))

        # Slopes average where normals do not, so the normals are resampled as slopes.
        normals = read_map(NORMAL_MAP) * 2.0 - 1.0
        normals /= np.linalg.norm(normals, axis=-1, keepdims=True)
        slopes = -normals[..., :2] / normals[..., 2:]

        return SurfaceVariant(
            color=np.maximum(resampler.resample(read_map(COLOR_MAP)), 0.0),
            slopes=resampler.resample(slopes),
            displacement=resampler.resample(read_map(DISPLACEMENT_MAP)),
        )
