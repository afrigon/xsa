from dataclasses import dataclass, replace

import numpy as np


# Consecutive rows of a source map, from row `top`: each pixel's values (rows × columns × channels, NaN without data)
# and the latitude and longitude of its center in radians, both broadcastable to the pixels.
@dataclass(frozen=True)
class SourceBand:
    values: np.ndarray
    top: int
    latitudes: np.ndarray
    longitudes: np.ndarray

    def with_values(self, values: np.ndarray) -> "SourceBand":
        return replace(self, values=values)
