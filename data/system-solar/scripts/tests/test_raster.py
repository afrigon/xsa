import numpy as np

from operations.raster import Raster, SourceLayout


def test_area_resample_keeps_the_mean_for_any_factor():
    values = np.random.default_rng(1).random((7, 27, 3))
    resampled = Raster._area_resample(values, 1, 5)

    assert resampled.shape == (7, 5, 3)
    assert np.allclose(resampled.mean(axis=1), values.mean(axis=1))


def test_area_resample_by_an_integer_factor_averages_blocks():
    values = np.arange(8.0)

    assert np.allclose(Raster._area_resample(values, 0, 4), [0.5, 2.5, 4.5, 6.5])


def test_game_layout_puts_longitude_zero_at_a_quarter_and_north_last():
    width = 8
    longitudes = -180 + (np.arange(width) + 0.5) * 360 / width
    source = np.zeros((4, width, 2))
    source[..., 0] = longitudes
    source[0, :, 1] = 1
    pixels = Raster._game_layout(source, SourceLayout())
    game_longitudes = 90 - (np.arange(width) + 0.5) * 360 / width

    assert np.allclose((pixels[0, :, 0] - game_longitudes) % 360, 0)
    assert np.all(pixels[-1, :, 1] == 1)


def test_mipmaps_reach_one_texel_and_keep_the_mean():
    raster = Raster(np.random.default_rng(2).random((4, 8, 1)))
    levels = raster.mipmaps()

    assert [level.shape[:2] for level in levels] == [(4, 8), (2, 4), (1, 2), (1, 1)]
    assert np.isclose(levels[-1][0, 0, 0], raster.pixels.mean())


# In the game's layout: longitude 0 a quarter of the way along each row, longitudes growing westward, rows from the
# south pole.
def test_interpolation_at_pixel_centers_gives_the_pixels_and_wraps_around():
    width, height = 8, 4
    pixels = np.arange(width * height, dtype=np.float32).reshape(height, width, 1)
    raster = Raster(pixels)
    columns = np.arange(width)
    longitudes = (0.25 - (columns + 0.5) / width) * 2 * np.pi
    latitude = ((1 + 0.5) / height - 0.5) * np.pi

    assert np.allclose(raster.interpolated_at(np.full(width, latitude), longitudes)[..., 0], pixels[1, :, 0])

    seam = (0.25 - 1.0) * 2 * np.pi
    assert np.isclose(raster.interpolated_at(np.array(latitude), np.array(seam))[0], (pixels[1, 0, 0] + pixels[1, -1, 0]) / 2)
