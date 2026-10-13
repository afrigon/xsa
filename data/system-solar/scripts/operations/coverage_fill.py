import logging

import numpy as np

from operations.raster import Raster

logger = logging.getLogger(__name__)


# Fills where a primary map has no data from a secondary measurement of a related quantity, related to the primary by
# a linear fit per channel over the band of latitudes both cover; across that band the primary hands over to the
# fitted secondary gradually, so no seam remains where the primary's coverage ends.
class CoverageFill:
    def __init__(self, overlap_start: float, overlap_end: float):
        self.overlap_start = overlap_start
        self.overlap_end = overlap_end

    def fill(self, primary: Raster, secondary: Raster) -> Raster:
        latitude = np.abs(primary.latitudes)[:, None]
        overlap = (latitude >= self.overlap_start) & (latitude <= self.overlap_end)
        reference = secondary.pixels[..., 0]
        fitted = np.empty_like(primary.pixels)

        for channel in range(primary.pixels.shape[-1]):
            target = primary.pixels[..., channel]
            sample = overlap & ~np.isnan(target) & ~np.isnan(reference)
            slope, intercept = np.polyfit(reference[sample].astype(np.float64), target[sample].astype(np.float64), 1)
            residual = np.std(target[sample] - (slope * reference[sample] + intercept))
            logger.info("channel %d: %.4f × secondary + %.4f, residual %.4f", channel, slope, intercept, residual)
            fitted[..., channel] = slope * reference + intercept

        handover = np.clip((latitude - self.overlap_start) / (self.overlap_end - self.overlap_start), 0, 1)[..., None]
        blended = np.where(np.isnan(fitted), primary.pixels, (1 - handover) * primary.pixels + handover * fitted)
        return Raster(np.where(np.isnan(primary.pixels), fitted, blended).astype(np.float32))
