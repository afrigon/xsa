import numpy as np
import rasterio

from operations.cube_map import CUBE_FACE_COUNT, CubeMap
from operations.raster import SourceLayout

FACE_SIZE = 4
FRONT, BACK, RIGHT, LEFT, TOP, BOTTOM = range(CUBE_FACE_COUNT)


def write_source(path, pixels):
    with rasterio.open(path, "w", driver="GTiff", width=pixels.shape[1], height=pixels.shape[0], count=1,
                       dtype="float32") as file:
        file.write(pixels.astype(np.float32), 1)


# Longitude of each column's center, from -180 at the left edge, growing to the right.
def longitudes(width):
    return -180 + (np.arange(width) + 0.5) * 360 / width


def test_texel_inverts_direction_on_every_face():
    centers = (np.arange(FACE_SIZE) + 0.5) / FACE_SIZE * 2 - 1

    for face in range(CUBE_FACE_COUNT):
        direction = CubeMap._direction(face, centers[None, :], centers[:, None])
        faces, columns, rows = CubeMap._texel(direction, FACE_SIZE)

        assert (faces == face).all()
        assert (columns == np.arange(FACE_SIZE)[None, :]).all()
        assert (rows == np.arange(FACE_SIZE)[:, None]).all()


def test_a_constant_source_gives_constant_faces(tmp_path):
    write_source(tmp_path / "flat.tif", np.full((180, 360), 2.5))
    cube = CubeMap.from_equirectangular(tmp_path / "flat.tif", FACE_SIZE, SourceLayout(), scale=1000)

    assert all(np.allclose(face.pixels, 2500) for face in cube.faces)


def test_faces_sit_on_the_body_frame_axes(tmp_path):
    width = 720
    latitude = 90 - (np.arange(width // 2) + 0.5) * 180 / (width // 2)
    write_source(tmp_path / "latitude.tif", np.repeat(latitude[:, None], width, axis=1))
    write_source(tmp_path / "longitude.tif", np.repeat(longitudes(width)[None, :], width // 2, axis=0))
    by_latitude = CubeMap.from_equirectangular(tmp_path / "latitude.tif", FACE_SIZE, SourceLayout())
    by_longitude = CubeMap.from_equirectangular(tmp_path / "longitude.tif", FACE_SIZE, SourceLayout())
    center = slice(FACE_SIZE // 2 - 1, FACE_SIZE // 2 + 1)

    assert abs(by_longitude.faces[FRONT].pixels[center, center].mean()) < 1
    assert abs(by_longitude.faces[RIGHT].pixels[center, center].mean() - 90) < 1
    assert abs(by_longitude.faces[LEFT].pixels[center, center].mean() + 90) < 1
    assert by_latitude.faces[TOP].pixels[center, center].mean() > 60
    assert by_latitude.faces[BOTTOM].pixels[center, center].mean() < -60
