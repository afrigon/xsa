import logging
import warnings
from dataclasses import dataclass
from pathlib import Path

import numpy as np
import rasterio

from operations import srgb

ROWS_PER_READ = 512
DEGREES_PER_TURN = 360.0
# The game's maps put longitude 0 at a quarter of the width, longitude growing to the left.
GAME_LEFT_EDGE_LONGITUDE = 90.0

logger = logging.getLogger(__name__)


@dataclass(frozen=True)
class SourceLayout:
    # Longitude of the left edge of the image, in degrees; longitude grows to the right.
    left_edge_longitude: float = -180.0
    north_up: bool = True


# An equirectangular map in the game's layout: longitude 0 at u = 0.25, u growing westward, south on row 0.
class Raster:
    def __init__(self, pixels: np.ndarray):
        self.pixels = pixels

    @property
    def width(self) -> int:
        return self.pixels.shape[1]

    @property
    def height(self) -> int:
        return self.pixels.shape[0]

    @staticmethod
    def native_width(path: Path) -> int:
        with rasterio.open(path) as source:
            return source.width

    # Reads a global map, area-averaged down to `width` in linear values, then put in the game's layout.
    @staticmethod
    def read(path: Path, width: int, layout: SourceLayout, encoded_srgb: bool = False, scale: float = 1.0) -> "Raster":
        # Global maps often ship without georeferencing; the recipe's layout describes them instead.
        with warnings.catch_warnings(category=rasterio.errors.NotGeoreferencedWarning, action="ignore"):
            source = rasterio.open(path)

        with source:
            if width > source.width:
                raise ValueError(f"{path.name}: {width} is wider than the source's {source.width}")

            maximum = np.iinfo(source.dtypes[0]).max if np.issubdtype(source.dtypes[0], np.integer) else 1.0
            rows = []

            for top in range(0, source.height, ROWS_PER_READ):
                window = rasterio.windows.Window(0, top, source.width, min(ROWS_PER_READ, source.height - top))
                band = np.moveaxis(source.read(window=window), 0, -1).astype(np.float64) / maximum

                if encoded_srgb:
                    band = srgb.to_linear(band)

                rows.append(Raster._area_resample(band * scale, 1, width).astype(np.float32))
                logger.debug("%s: read rows %d to %d", path.name, top, top + band.shape[0])

        pixels = Raster._area_resample(np.concatenate(rows), 0, width // 2)
        return Raster(Raster._game_layout(pixels, layout))

    def mipmaps(self) -> list[np.ndarray]:
        levels = [self.pixels]

        while levels[-1].shape[0] > 1 or levels[-1].shape[1] > 1:
            level = levels[-1]

            if level.shape[0] > 1:
                level = (level[0::2] + level[1::2]) / 2

            if level.shape[1] > 1:
                level = (level[:, 0::2] + level[:, 1::2]) / 2

            levels.append(level)

        return levels

    # Equirectangular rows cover less area toward the poles, in proportion to the cosine of their latitude.
    def area_mean(self) -> np.ndarray:
        weights = np.cos(self._latitudes())
        rows = self.pixels.astype(np.float64).mean(axis=1)
        return (rows * weights[:, None]).sum(axis=0) / weights.sum()

    # From a height map in meters: the planet shader's tangent-space normal, whose x is the normal's westward
    # component and y its northward one.
    def normals(self, radius: float) -> "Raster":
        heights = self.pixels[..., 0].astype(np.float64)
        meters_per_column = 2 * np.pi * radius / self.width * np.cos(self._latitudes())[:, None]
        meters_per_row = np.pi * radius / self.height
        # Columns grow westward and rows northward.
        east_slope = (np.roll(heights, 1, axis=1) - np.roll(heights, -1, axis=1)) / (2 * meters_per_column)
        padded = np.pad(heights, ((1, 1), (0, 0)), mode="edge")
        north_slope = (padded[2:] - padded[:-2]) / (2 * meters_per_row)
        length = np.sqrt(east_slope**2 + north_slope**2 + 1)
        return Raster(np.stack([east_slope / length, -north_slope / length], axis=-1).astype(np.float32))

    def _latitudes(self) -> np.ndarray:
        return (np.arange(self.height) + 0.5) / self.height * np.pi - np.pi / 2

    # Box filter of exact area: each output sample averages the span of input it covers, fractions included.
    @staticmethod
    def _area_resample(values: np.ndarray, axis: int, size: int) -> np.ndarray:
        count = values.shape[axis]

        if size == count:
            return values

        values = np.moveaxis(values, axis, 0).astype(np.float64)
        prefix = np.concatenate([np.zeros((1,) + values.shape[1:]), np.cumsum(values, axis=0)])
        positions = np.arange(size + 1) * (count / size)
        whole = np.minimum(np.floor(positions).astype(int), count - 1)
        fraction = (positions - whole).reshape((-1,) + (1,) * (values.ndim - 1))
        integral = prefix[whole] + fraction * values[whole]
        return np.moveaxis((integral[1:] - integral[:-1]) * (size / count), 0, axis)

    @staticmethod
    def _game_layout(pixels: np.ndarray, layout: SourceLayout) -> np.ndarray:
        width = pixels.shape[1]
        shift = (GAME_LEFT_EDGE_LONGITUDE - layout.left_edge_longitude) / DEGREES_PER_TURN * width

        if shift != round(shift):
            raise ValueError(f"a {width}-wide map cannot be shifted to the game's longitudes without resampling")

        columns = (round(shift) - 1 - np.arange(width)) % width
        rows = slice(None, None, -1) if layout.north_up else slice(None)
        return np.ascontiguousarray(pixels[rows][:, columns])
