import numpy as np

from operations.coverage_fill import CoverageFill
from operations.cube_map import CUBE_FACE_COUNT, CubeMap
from operations.cube_projection import CubeProjection
from operations.raster import Raster

FACE_SIZE = 32


def test_the_primary_is_kept_inside_its_coverage_and_the_fitted_secondary_fills_beyond():
    projection = CubeProjection.EQUAL_ANGLE
    random = np.random.default_rng(3)
    secondaries, primaries, truths = [], [], []

    for face in range(CUBE_FACE_COUNT):
        latitudes = np.degrees(np.arcsin(CubeMap.texel_directions(face, FACE_SIZE, projection)[..., 2]))[..., None]
        secondary = random.random((FACE_SIZE, FACE_SIZE, 1)) + 1
        truth = 0.5 * secondary + 0.1
        secondaries.append(Raster(secondary))
        truths.append(truth)
        primaries.append(Raster(np.where(np.abs(latitudes) < 70, truth, np.nan)))

    filled = CoverageFill(60, 70).fill(CubeMap(primaries, projection), CubeMap(secondaries, projection))

    for face, truth in zip(filled.faces, truths):
        assert not np.isnan(face.pixels).any()
        assert np.allclose(face.pixels, truth, atol=1e-5)
