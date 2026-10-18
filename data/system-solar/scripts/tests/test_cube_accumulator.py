import numpy as np
import rasterio
from rasterio.crs import CRS
from rasterio.transform import from_origin

from operations.cube_accumulator import CubeAccumulator
from operations.cube_map import CUBE_FACE_COUNT
from operations.cube_projection import CubeProjection
from operations.equirectangular_source import EquirectangularSource
from operations.polar_stereographic_source import PolarStereographicSource
from operations.raster import Raster, SourceLayout

FACE_SIZE = 4
FRONT, BACK, RIGHT, LEFT, TOP, BOTTOM = range(CUBE_FACE_COUNT)
RADIUS = 1737400.0
NORTH_POLAR = CRS.from_proj4(f"+proj=stere +lat_0=90 +lat_ts=90 +lon_0=0 +x_0=0 +y_0=0 +R={RADIUS} +units=m +no_defs")


def write_source(path, pixels):
    with rasterio.open(path, "w", driver="GTiff", width=pixels.shape[1], height=pixels.shape[0], count=1,
                       dtype="float32") as file:
        file.write(pixels.astype(np.float32), 1)


def write_polar(path, pixels, pixel_meters):
    half = pixels.shape[0] * pixel_meters / 2
    with rasterio.open(path, "w", driver="GTiff", width=pixels.shape[1], height=pixels.shape[0], count=1,
                       dtype="float32", crs=NORTH_POLAR, transform=from_origin(-half, half, pixel_meters, pixel_meters),
                       nodata=np.nan) as file:
        file.write(pixels.astype(np.float32), 1)


# Longitude of each column's center, from -180 at the left edge, growing to the right.
def longitudes(width):
    return -180 + (np.arange(width) + 0.5) * 360 / width


def accumulated(source, face_size=FACE_SIZE):
    accumulator = CubeAccumulator(face_size, CubeProjection.EQUAL_ANGLE, 1)
    accumulator.add(source)
    return accumulator


def test_a_constant_source_gives_constant_faces(tmp_path):
    write_source(tmp_path / "flat.tif", np.full((180, 360), 2.5))
    cube = accumulated(EquirectangularSource.global_map(tmp_path / "flat.tif", SourceLayout(), scale=1000)).cube_map()

    assert all(np.allclose(face.pixels, 2500) for face in cube.faces)


def test_faces_sit_on_the_body_frame_axes(tmp_path):
    width = 720
    latitude = 90 - (np.arange(width // 2) + 0.5) * 180 / (width // 2)
    write_source(tmp_path / "latitude.tif", np.repeat(latitude[:, None], width, axis=1))
    write_source(tmp_path / "longitude.tif", np.repeat(longitudes(width)[None, :], width // 2, axis=0))
    by_latitude = accumulated(EquirectangularSource.global_map(tmp_path / "latitude.tif", SourceLayout())).cube_map()
    by_longitude = accumulated(EquirectangularSource.global_map(tmp_path / "longitude.tif", SourceLayout())).cube_map()
    center = slice(FACE_SIZE // 2 - 1, FACE_SIZE // 2 + 1)

    assert abs(by_longitude.faces[FRONT].pixels[center, center].mean()) < 1
    assert abs(by_longitude.faces[RIGHT].pixels[center, center].mean() - 90) < 1
    assert abs(by_longitude.faces[LEFT].pixels[center, center].mean() + 90) < 1
    assert by_latitude.faces[TOP].pixels[center, center].mean() > 60
    assert by_latitude.faces[BOTTOM].pixels[center, center].mean() < -60


# A source coarser than the texels still reaches every texel: its pixels are split finer.
def test_a_coarse_source_reaches_every_texel():
    raster = Raster(np.ones((18, 36, 1), np.float32))
    cube = accumulated(EquirectangularSource.from_raster(raster), face_size=128).cube_map()

    assert all(np.isfinite(face.pixels).all() for face in cube.faces)


# The map in the game's layout puts longitude 0 a quarter of the way along each row, longitudes growing westward.
def test_a_raster_in_the_game_layout_sits_on_the_body_frame_axes():
    width = 360
    longitudes = np.radians((0.25 - (np.arange(width) + 0.5) / width) * 360)
    raster = Raster(np.repeat(np.sin(longitudes)[None, :, None], width // 2, axis=0).astype(np.float32))
    cube = accumulated(EquirectangularSource.from_raster(raster)).cube_map()
    center = slice(FACE_SIZE // 2 - 1, FACE_SIZE // 2 + 1)

    assert abs(cube.faces[FRONT].pixels[center, center].mean()) < 0.1
    assert cube.faces[RIGHT].pixels[center, center].mean() > 0.9
    assert cube.faces[LEFT].pixels[center, center].mean() < -0.9


def test_the_polar_projection_matches_rasterio(tmp_path):
    write_polar(tmp_path / "polar.tif", np.zeros((8, 8)), 50000.0)
    source = PolarStereographicSource(tmp_path / "polar.tif")
    x = np.array([0.0, 100000.0, -70000.0, 30000.0])
    y = np.array([50000.0, 0.0, -20000.0, 90000.0])
    latitudes, longitudes = source._geographic(x, y)
    expected_longitudes, expected_latitudes = rasterio.warp.transform(
        NORTH_POLAR, CRS.from_proj4(f"+proj=longlat +R={RADIUS} +no_defs"), x, y
    )

    assert np.allclose(np.degrees(latitudes), expected_latitudes)
    assert np.allclose(np.degrees(longitudes), expected_longitudes)


def test_a_polar_source_covers_only_the_texels_inside_it(tmp_path):
    write_polar(tmp_path / "cap.tif", np.full((100, 100), 7.0), 4000.0)
    accumulator = accumulated(PolarStereographicSource(tmp_path / "cap.tif"), face_size=128)
    covered = accumulator.covered(PolarStereographicSource(tmp_path / "cap.tif"))
    cube = accumulator.cube_map()
    center = 64

    assert covered[TOP, center, center]
    assert not covered[TOP, 0, 0]
    assert not covered[FRONT].any()
    assert np.allclose(cube.faces[TOP].pixels[covered[TOP]], 7.0)


def test_a_texel_with_missing_data_is_not_covered(tmp_path):
    pixels = np.full((100, 100), 7.0)
    pixels[50, 50] = np.nan
    write_polar(tmp_path / "hole.tif", pixels, 4000.0)
    source = PolarStereographicSource(tmp_path / "hole.tif")

    assert not accumulated(source, face_size=128).covered(source)[TOP, 63:65, 63:65].all()
