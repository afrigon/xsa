from collections.abc import Callable, Iterator
from pathlib import Path

import numpy as np
import rasterio

from operations.raster import Raster, SourceLayout
from operations.source_band import SourceBand

ROWS_PER_READ = 64


# A map whose rows are evenly spaced latitudes and whose columns are evenly spaced longitudes, read in bands of rows.
# Steps are signed radians per pixel: a column step below zero runs westward, a row step below zero southward.
class EquirectangularSource:
    def __init__(
        self,
        read: Callable[[int, int], np.ndarray],
        width: int,
        height: int,
        left_longitude: float,
        top_latitude: float,
        column_step: float,
        row_step: float,
    ):
        self.read = read
        self.width = width
        self.height = height
        self.left_longitude = left_longitude
        self.top_latitude = top_latitude
        self.column_step = column_step
        self.row_step = row_step

    # A whole-body map without georeferencing, laid out as `layout` describes.
    @staticmethod
    def global_map(path: Path, layout: SourceLayout, scale: float = 1.0) -> "EquirectangularSource":
        with Raster._open(path) as source:
            width, height = source.width, source.height

        def read(top: int, count: int) -> np.ndarray:
            with Raster._open(path) as source:
                window = rasterio.windows.Window(0, top, width, count)
                return source.read(1, window=window).astype(np.float64)[..., None] * scale

        top_latitude, row_step = (np.pi / 2, -np.pi / height) if layout.north_up else (-np.pi / 2, np.pi / height)
        return EquirectangularSource(
            read, width, height, np.radians(layout.left_edge_longitude), top_latitude, 2 * np.pi / width, row_step
        )

    # A georeferenced map or tile in meters on a sphere; its no-data becomes NaN.
    @staticmethod
    def georeferenced(path: Path, bands: list[int] | None = None) -> "EquirectangularSource":
        with rasterio.open(path) as source:
            radius = Raster._radius(source)
            transform = source.transform
            width, height = source.width, source.height
            indexes = bands or list(range(1, source.count + 1))
            scales = np.array([source.scales[index - 1] for index in indexes])

        def read(top: int, count: int) -> np.ndarray:
            with rasterio.open(path) as source:
                window = rasterio.windows.Window(0, top, width, count)
                values = source.read(indexes, window=window, masked=True).astype(np.float64).filled(np.nan)
                return np.moveaxis(values, 0, -1) * scales

        return EquirectangularSource(
            read,
            width,
            height,
            transform.c / radius,
            transform.f / radius,
            transform.a / radius,
            transform.e / radius,
        )

    # A whole-body map in the game's layout: longitude 0 a quarter of the way along each row, longitudes growing
    # westward, rows from the south pole.
    @staticmethod
    def from_raster(raster: Raster) -> "EquirectangularSource":
        pixels = raster.pixels.astype(np.float64)
        return EquirectangularSource(
            lambda top, count: pixels[top : top + count],
            raster.width,
            raster.height,
            np.pi / 2,
            -np.pi / 2,
            -2 * np.pi / raster.width,
            np.pi / raster.height,
        )

    # The widest angle a pixel spans, on the equator.
    @property
    def pixel_angle(self) -> float:
        return max(abs(self.column_step), abs(self.row_step))

    def bands(self) -> Iterator[SourceBand]:
        longitudes = self.left_longitude + (np.arange(self.width) + 0.5) * self.column_step

        for top in range(0, self.height, ROWS_PER_READ):
            count = min(ROWS_PER_READ, self.height - top)
            latitudes = self.top_latitude + (top + np.arange(count) + 0.5) * self.row_step
            yield SourceBand(self.read(top, count), top, latitudes[:, None], longitudes[None, :])

    # Each pixel of the band split into `subdivisions` × `subdivisions` cells: their centers' directions and the
    # solid angles they cover, rows of cells in the pixels' order.
    def subcells(self, band: SourceBand, subdivisions: int) -> tuple[np.ndarray, np.ndarray]:
        rows, columns = band.values.shape[:2]
        offsets = (np.arange(rows * subdivisions) + 0.5) / subdivisions
        latitudes = self.top_latitude + (band.top + offsets) * self.row_step
        longitudes = self.left_longitude + (np.arange(columns * subdivisions) + 0.5) / subdivisions * self.column_step
        directions = np.stack(
            np.broadcast_arrays(
                np.cos(latitudes)[:, None] * np.cos(longitudes)[None, :],
                np.cos(latitudes)[:, None] * np.sin(longitudes)[None, :],
                np.sin(latitudes)[:, None],
            ),
            axis=-1,
        )
        cell_area = abs(self.row_step * self.column_step) / subdivisions**2
        areas = np.broadcast_to((np.cos(latitudes) * cell_area)[:, None], directions.shape[:2])
        return directions, areas

    # Whether each direction falls inside the map's extent.
    def contains(self, directions: np.ndarray) -> np.ndarray:
        latitudes = np.arcsin(np.clip(directions[..., 2], -1, 1))
        longitudes = np.arctan2(directions[..., 1], directions[..., 0])
        bottom = self.top_latitude + self.height * self.row_step
        inside_rows = (latitudes >= min(self.top_latitude, bottom)) & (latitudes <= max(self.top_latitude, bottom))
        columns_per_turn = 2 * np.pi / abs(self.column_step)
        column = np.mod((longitudes - self.left_longitude) / self.column_step, columns_per_turn)
        return inside_rows & (column <= self.width)
