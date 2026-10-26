from collections.abc import Iterator
from pathlib import Path

import numpy as np
import rasterio

from operations.raster import Raster
from operations.source_band import SourceBand

ROWS_PER_READ = 64


# A map in the spherical polar stereographic projection, centered on a pole with true scale there, read in bands of
# rows. LOLA's labels give the reference radius as the offset: values are heights above it, so only the scale applies.
class PolarStereographicSource:
    def __init__(self, path: Path):
        self.path = path

        with rasterio.open(path) as source:
            crs = source.crs
            parameters = crs.to_dict()
            self.radius = Raster._radius(source)
            self.transform = source.transform
            self.width, self.height = source.width, source.height
            self.scale = source.scales[0]

        pole = parameters.get("lat_0")
        expected = {"proj": "stere", "lat_ts": pole, "lon_0": 0, "x_0": 0, "y_0": 0}

        if pole not in (90, -90) or any(parameters.get(key) != value for key, value in expected.items()):
            raise ValueError(f"{path.name}: {crs} is not a spherical polar stereographic projection")

        if self.transform.b or self.transform.d:
            raise ValueError(f"{path.name}: rotated pixels are not supported")

        self.hemisphere = np.sign(pole)

    # The widest angle a pixel spans, at the pole where the scale is true.
    @property
    def pixel_angle(self) -> float:
        return max(abs(self.transform.a), abs(self.transform.e)) / self.radius

    def bands(self) -> Iterator[SourceBand]:
        columns = np.arange(self.width) + 0.5

        for top in range(0, self.height, ROWS_PER_READ):
            count = min(ROWS_PER_READ, self.height - top)

            with rasterio.open(self.path) as source:
                window = rasterio.windows.Window(0, top, self.width, count)
                values = source.read(1, window=window, masked=True).astype(np.float64).filled(np.nan) * self.scale

            latitudes, longitudes = self._geographic(*self._projected(columns[None, :], top + np.arange(count)[:, None] + 0.5))
            yield SourceBand(values[..., None], top, latitudes, longitudes)

    # Each pixel of the band split into `subdivisions` × `subdivisions` cells: their centers' directions and the
    # solid angles they cover, rows of cells in the pixels' order.
    def subcells(self, band: SourceBand, subdivisions: int) -> tuple[np.ndarray, np.ndarray]:
        rows, columns = band.values.shape[:2]
        x, y = self._projected(
            (np.arange(columns * subdivisions) + 0.5)[None, :] / subdivisions,
            band.top + (np.arange(rows * subdivisions) + 0.5)[:, None] / subdivisions,
        )
        latitudes, longitudes = self._geographic(x, y)
        directions = np.stack(
            [np.cos(latitudes) * np.cos(longitudes), np.cos(latitudes) * np.sin(longitudes), np.sin(latitudes)],
            axis=-1,
        )
        # The projection stretches lengths by 2 / (1 + sin |latitude|), so areas by its square.
        stretch = 2 / (1 + np.sin(np.abs(latitudes)))
        cell_area = abs(self.transform.a * self.transform.e) / subdivisions**2 / self.radius**2
        return directions, cell_area / stretch**2

    # Whether each direction falls inside the map's square.
    def contains(self, directions: np.ndarray) -> np.ndarray:
        latitudes = np.arcsin(np.clip(directions[..., 2], -1, 1))
        longitudes = np.arctan2(directions[..., 1], directions[..., 0])
        distance = 2 * self.radius * np.tan((np.pi / 2 - self.hemisphere * latitudes) / 2)
        x = distance * np.sin(longitudes)
        y = -self.hemisphere * distance * np.cos(longitudes)
        column = (x - self.transform.c) / self.transform.a
        row = (y - self.transform.f) / self.transform.e
        return (column >= 0) & (column <= self.width) & (row >= 0) & (row <= self.height)

    # Pixel coordinates, from the top left corner of the map, to meters in the projection's plane.
    def _projected(self, column: np.ndarray, row: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
        return self.transform.c + column * self.transform.a, self.transform.f + row * self.transform.e

    def _geographic(self, x: np.ndarray, y: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
        from_pole = 2 * np.arctan(np.hypot(x, y) / (2 * self.radius))
        latitudes = self.hemisphere * (np.pi / 2 - from_pole)
        longitudes = np.arctan2(x, -self.hemisphere * y)
        return latitudes, longitudes
