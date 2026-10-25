import numpy as np

from operations.array_writer import ArrayWriter


def test_a_flat_slope_encodes_an_upright_normal():
    rgba = ArrayWriter._normal(np.zeros((1, 1, 2)))

    assert list(rgba[0, 0, :2]) == [128, 128]


def test_slopes_tilt_the_normal_against_them():
    x, y = ArrayWriter._normal(np.array([[[1.0, 0.0]]]))[0, 0, :2] / 255 * 2 - 1

    assert np.isclose(x, -np.sqrt(0.5), atol=0.01) and np.isclose(y, 0.0, atol=0.01)
