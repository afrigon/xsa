import logging

import numpy as np

from operations.cube_map import CUBE_FACE_COUNT, CubeMap
from operations.raster import Raster

logger = logging.getLogger(__name__)


# Fills where a primary map has no data from a secondary measurement of a related quantity, related to the primary by
# a linear fit per channel over the band of latitudes both cover; across that band the primary hands over to the
# fitted secondary gradually, so no seam remains where the primary's coverage ends.
class CoverageFill:
    def __init__(self, overlap_start: float, overlap_end: float):
        self.overlap_start = overlap_start
        self.overlap_end = overlap_end

    def fill(self, primary: CubeMap, secondary: CubeMap) -> CubeMap:
        latitudes = [self._latitudes(primary, face) for face in range(CUBE_FACE_COUNT)]
        channels = primary.faces[0].pixels.shape[-1]
        fits = [self._fit(primary, secondary, latitudes, channel) for channel in range(channels)]
        faces = []

        for face in range(CUBE_FACE_COUNT):
            pixels = primary.faces[face].pixels
            reference = secondary.faces[face].pixels[..., 0]
            fitted = np.stack([slope * reference + intercept for slope, intercept in fits], axis=-1)
            span = self.overlap_end - self.overlap_start
            handover = np.clip((latitudes[face] - self.overlap_start) / span, 0, 1)[..., None]
            blended = np.where(np.isnan(fitted), pixels, (1 - handover) * pixels + handover * fitted)
            faces.append(Raster(np.where(np.isnan(pixels), fitted, blended).astype(np.float32)))

        return CubeMap(faces, primary.projection)

    def _fit(self, primary: CubeMap, secondary: CubeMap, latitudes: list[np.ndarray], channel: int) -> tuple:
        targets, references = [], []

        for face in range(CUBE_FACE_COUNT):
            target = primary.faces[face].pixels[..., channel]
            reference = secondary.faces[face].pixels[..., 0]
            overlap = (latitudes[face] >= self.overlap_start) & (latitudes[face] <= self.overlap_end)
            sample = overlap & ~np.isnan(target) & ~np.isnan(reference)
            targets.append(target[sample].astype(np.float64))
            references.append(reference[sample].astype(np.float64))

        target, reference = np.concatenate(targets), np.concatenate(references)
        slope, intercept = np.polyfit(reference, target, 1)
        residual = np.std(target - (slope * reference + intercept))
        logger.info("channel %d: %.4f × secondary + %.4f, residual %.4f", channel, slope, intercept, residual)
        return slope, intercept

    # Absolute latitude of each texel, in degrees.
    @staticmethod
    def _latitudes(cube: CubeMap, face: int) -> np.ndarray:
        directions = CubeMap.texel_directions(face, cube.face_size, cube.projection)
        return np.degrees(np.abs(np.arcsin(np.clip(directions[..., 2], -1, 1))))
