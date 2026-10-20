import numpy as np

from operations.mip_chain import MipChain


def test_levels_halve_rounding_down_to_one_texel():
    chain = MipChain(np.zeros((1588, 1588, 1), np.float32))

    assert [level.shape[0] for level in chain.levels] == [1588, 794, 397, 198, 99, 49, 24, 12, 6, 3, 1]


def test_every_level_keeps_the_mean():
    pixels = np.random.default_rng(3).random((397, 397, 2)).astype(np.float32)
    chain = MipChain(pixels)

    for level in chain.levels:
        assert np.allclose(level.mean(axis=(0, 1)), pixels.mean(axis=(0, 1)), atol=1e-5)
