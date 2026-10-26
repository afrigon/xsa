import logging
import math
from typing import Protocol

import numpy as np

from operations.cube_map import CUBE_FACE_COUNT, CubeMap
from operations.cube_projection import CubeProjection
from operations.raster import Raster
from operations.source_band import SourceBand

logger = logging.getLogger(__name__)


class Source(Protocol):
    @property
    def pixel_angle(self) -> float: ...

    def bands(self): ...

    def subcells(self, band: SourceBand, subdivisions: int) -> tuple[np.ndarray, np.ndarray]: ...

    def contains(self, directions: np.ndarray) -> np.ndarray: ...


# Each texel is the area-weighted average of the source pixels on the ground it covers, read in the source's own
# projection. Ground without data, NaN in the source, is counted apart, so a texel knows whether a source covers it.
# Source pixels are split until every texel receives at least `cells_per_texel` cells along each axis, so a pixel
# straddling texels is shared between them in proportion to its ground in each; at 1, a pixel finer than a texel goes
# whole to the texel under its center.
class CubeAccumulator:
    def __init__(self, face_size: int, projection: CubeProjection, channels: int, cells_per_texel: int):
        self.face_size = face_size
        self.projection = projection
        self.cells_per_texel = cells_per_texel
        texel_count = CUBE_FACE_COUNT * face_size * face_size
        self.sums = np.zeros((texel_count, channels))
        self.weights = np.zeros(texel_count)
        self.missing = np.zeros(texel_count)

    def subdivisions(self, source: Source) -> int:
        width = CubeMap.smallest_texel_width(self.face_size, self.projection)
        return math.ceil(self.cells_per_texel * source.pixel_angle / width)

    def add(self, source: Source) -> None:
        subdivisions = self.subdivisions(source)

        for band in source.bands():
            self.add_band(source, band, subdivisions)

    # `band` may carry values computed from the source's own, one per source pixel.
    def add_band(self, source: Source, band: SourceBand, subdivisions: int) -> None:
        directions, areas = source.subcells(band, subdivisions)
        face, column, row = CubeMap._texel(directions, self.face_size, self.projection)
        texels = ((face * self.face_size + row) * self.face_size + column).ravel()
        values = band.values.repeat(subdivisions, axis=0).repeat(subdivisions, axis=1)
        values = values.reshape(-1, values.shape[-1])
        areas = np.ascontiguousarray(areas).ravel()
        valid = np.isfinite(values).all(axis=-1)
        # A band touches a sliver of the cube: sums run over the texels it reaches, never over the whole cube.
        touched, slots = np.unique(texels, return_inverse=True)
        count = len(touched)

        for channel in range(values.shape[-1]):
            self.sums[touched, channel] += np.bincount(slots[valid], values[valid, channel] * areas[valid], count)

        self.weights[touched] += np.bincount(slots[valid], areas[valid], count)
        self.missing[touched] += np.bincount(slots[~valid], areas[~valid], count)
        logger.debug("added rows %d to %d", band.top, band.top + band.values.shape[0])

    # NaN where no source pixel with data fell.
    def cube_map(self) -> CubeMap:
        with np.errstate(invalid="ignore", divide="ignore"):
            pixels = np.where(self.weights[:, None] > 0, self.sums / self.weights[:, None], np.nan)

        faces = pixels.astype(np.float32).reshape(CUBE_FACE_COUNT, self.face_size, self.face_size, -1)
        return CubeMap([Raster(face) for face in faces], self.projection)

    # Whether `source`, accumulated alone, covers each texel entirely: the texel lies inside the source's extent and
    # none of its ground lacks data.
    def covered(self, source: Source) -> np.ndarray:
        size = self.face_size
        received = (self.weights > 0) & (self.missing == 0)
        received = received.reshape(CUBE_FACE_COUNT, size, size)
        edges = np.linspace(-1, 1, size + 1)
        inside = np.empty_like(received)

        for face in range(CUBE_FACE_COUNT):
            corners = source.contains(CubeMap._direction(face, edges[None, :], edges[:, None], self.projection))
            inside[face] = corners[:-1, :-1] & corners[:-1, 1:] & corners[1:, :-1] & corners[1:, 1:]

        return received & inside
