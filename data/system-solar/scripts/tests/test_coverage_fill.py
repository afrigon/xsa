import numpy as np

from operations.coverage_fill import CoverageFill
from operations.raster import Raster


def test_the_primary_is_kept_inside_its_coverage_and_the_fitted_secondary_fills_beyond():
    height, width = 36, 72
    latitudes = Raster(np.zeros((height, width, 1))).latitudes[:, None, None]
    secondary = np.broadcast_to(np.random.default_rng(3).random((1, width, 1)) + 1, (height, width, 1)).copy()
    truth = 0.5 * secondary + 0.1
    primary = np.where(np.abs(latitudes) < 70, truth, np.nan)
    filled = CoverageFill(60, 70).fill(Raster(primary), Raster(secondary)).pixels

    assert not np.isnan(filled).any()
    assert np.allclose(filled, truth, atol=1e-5)
