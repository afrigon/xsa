import logging
import warnings
from dataclasses import dataclass
from pathlib import Path

import numpy as np
import rasterio
import rasterio.warp
from rasterio.crs import CRS
from rasterio.transform import from_origin

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
        with Raster._open(path) as source:
            return source.width

    # Global maps often ship without georeferencing; the recipe's layout describes them instead.
    @staticmethod
    def _open(path: Path):
        with warnings.catch_warnings(category=rasterio.errors.NotGeoreferencedWarning, action="ignore"):
            return rasterio.open(path)

    # Reads a global map, area-averaged down to `width` in linear values, then put in the game's layout.
    @staticmethod
    def read(path: Path, width: int, layout: SourceLayout, encoded_srgb: bool = False, scale: float = 1.0) -> "Raster":
        with Raster._open(path) as source:
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

    # Georeferenced tiles of one global map, in meters on a sphere, placed on a global grid at their own resolution;
    # their no-data becomes NaN, which resampling leaves out.
    @staticmethod
    def read_tiles(paths: list[Path], width: int, bands: list[int] | None = None) -> "Raster":
        grid = None

        for path in paths:
            with rasterio.open(path) as source:
                meters_per_degree = Raster._radius(source) * np.pi / 180
                degrees_per_pixel = source.transform.a / meters_per_degree
                columns = round(DEGREES_PER_TURN / degrees_per_pixel)
                left = round(source.transform.c / meters_per_degree / degrees_per_pixel) % columns
                top = round((90 - source.transform.f / meters_per_degree) / degrees_per_pixel)
                indexes = bands or list(range(1, source.count + 1))
                scales = np.array([source.scales[index - 1] for index in indexes], np.float32)
                data = np.moveaxis(source.read(indexes, masked=True).astype(np.float32).filled(np.nan), 0, -1) * scales

                if grid is None:
                    grid = np.full((columns // 2, columns, data.shape[-1]), np.nan, np.float32)

                rows = slice(max(top, 0), min(top + data.shape[0], grid.shape[0]))
                target = grid[rows, left : left + data.shape[1]]
                np.copyto(target, data[rows.start - top : rows.stop - top], where=np.isnan(target))
                logger.debug("%s placed at row %d, column %d", path.name, top, left)

        return Raster(Raster._game_layout(Raster._resampled(grid, width), SourceLayout(left_edge_longitude=0.0)))

    # A map in any projection rasterio reads (polar stereographic, …), interpolated onto the global grid: for maps
    # whose pixels are no larger than the grid's where they have data, such as polar maps.
    @staticmethod
    def read_reprojected(path: Path, width: int) -> "Raster":
        with rasterio.open(path) as source:
            height = width // 2
            target_crs = CRS.from_proj4(f"+proj=longlat +R={Raster._radius(source)} +no_defs")
            pixels = np.full((height, width), np.nan, np.float32)
            rasterio.warp.reproject(
                source.read(1, masked=True).astype(np.float32).filled(np.nan) * source.scales[0],
                pixels,
                src_transform=source.transform,
                src_crs=source.crs,
                src_nodata=np.nan,
                dst_transform=from_origin(0.0, 90.0, DEGREES_PER_TURN / width, 180.0 / height),
                dst_crs=target_crs,
                dst_nodata=np.nan,
                resampling=rasterio.warp.Resampling.bilinear,
            )

        return Raster(Raster._game_layout(pixels[..., None], SourceLayout(left_edge_longitude=0.0)))

    @property
    def latitudes(self) -> np.ndarray:
        return np.degrees(self._latitudes())

    # Rows without data take the nearest row that has some, column by column.
    def extended_toward_poles(self) -> "Raster":
        valid_rows = np.flatnonzero(~np.isnan(self.pixels).all(axis=(1, 2)))
        nearest = valid_rows[np.abs(np.arange(self.height)[:, None] - valid_rows[None, :]).argmin(axis=1)]
        return Raster(self.pixels[nearest])

    # Bilinear interpolation onto a finer grid, for smooth fields such as fitted parameters, whose values between
    # samples are estimates either way: the area average would turn each sample into a visible square.
    def interpolated(self, width: int) -> "Raster":
        height = width // 2
        x = (np.arange(width) + 0.5) * self.width / width - 0.5
        y = np.clip((np.arange(height) + 0.5) * self.height / height - 0.5, 0, self.height - 1)
        x0, y0 = np.floor(x).astype(int), np.floor(y).astype(int)
        fx, fy = (x - x0)[None, :, None], (y - y0)[:, None, None]
        x1, y1 = (x0 + 1) % self.width, np.minimum(y0 + 1, self.height - 1)
        x0 %= self.width
        top = self.pixels[y0][:, x0] * (1 - fx) + self.pixels[y0][:, x1] * fx
        bottom = self.pixels[y1][:, x0] * (1 - fx) + self.pixels[y1][:, x1] * fx
        return Raster((top * (1 - fy) + bottom * fy).astype(np.float32))

    def resampled(self, width: int) -> "Raster":
        return Raster(Raster._resampled(self.pixels, width))

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

    # From a height map in meters: the material shaders' tangent-space normal, whose x is the normal's westward
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

    @staticmethod
    def _radius(source) -> float:
        return source.crs.to_dict().get("R") or source.crs.ellipsoid_major_axis

    # Resamples a whole map to `width`, columns first in bands of rows to bound the memory used.
    @staticmethod
    def _resampled(pixels: np.ndarray, width: int) -> np.ndarray:
        rows = [
            Raster._area_resample(pixels[top : top + ROWS_PER_READ], 1, width).astype(np.float32)
            for top in range(0, pixels.shape[0], ROWS_PER_READ)
        ]
        return Raster._area_resample(np.concatenate(rows), 0, width // 2).astype(np.float32)

    # Box filter of exact area: each output sample averages the span of input it covers, fractions included. NaN
    # marks missing data: an output averages only the valid input it covers, and stays NaN when there is none.
    @staticmethod
    def _area_resample(values: np.ndarray, axis: int, size: int) -> np.ndarray:
        if size == values.shape[axis]:
            return values

        valid = ~np.isnan(values)

        if valid.all():
            return Raster._box(values, axis, size)

        sums = Raster._box(np.where(valid, values, 0), axis, size)
        weights = Raster._box(valid.astype(np.float64), axis, size)

        with np.errstate(invalid="ignore", divide="ignore"):
            return np.where(weights > 0, sums / weights, np.nan)

    @staticmethod
    def _box(values: np.ndarray, axis: int, size: int) -> np.ndarray:
        count = values.shape[axis]

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
