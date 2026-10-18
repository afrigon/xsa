import numpy as np
import pytest

from operations.cube_map import CUBE_FACE_COUNT, CubeMap
from operations.cube_projection import CubeProjection
from operations.raster import Raster

FACE_SIZE = 4
FRONT, BACK, RIGHT, LEFT, TOP, BOTTOM = range(CUBE_FACE_COUNT)


@pytest.mark.parametrize("projection", list(CubeProjection))
def test_texel_inverts_direction_on_every_face(projection):
    centers = (np.arange(FACE_SIZE) + 0.5) / FACE_SIZE * 2 - 1

    for face in range(CUBE_FACE_COUNT):
        direction = CubeMap._direction(face, centers[None, :], centers[:, None], projection)
        faces, columns, rows = CubeMap._texel(direction, FACE_SIZE, projection)

        assert (faces == face).all()
        assert (columns == np.arange(FACE_SIZE)[None, :]).all()
        assert (rows == np.arange(FACE_SIZE)[:, None]).all()


def test_equal_angle_texels_span_equal_angles_along_a_faces_center_row():
    edges = np.linspace(-1, 1, FACE_SIZE + 1)
    direction = CubeMap._direction(FRONT, edges, np.zeros_like(edges), CubeProjection.EQUAL_ANGLE)
    angles = np.arctan2(-direction[:, 2], direction[:, 0])

    assert np.allclose(np.diff(angles), np.pi / 2 / FACE_SIZE)


@pytest.mark.parametrize("projection", list(CubeProjection))
def test_texel_solid_angles_cover_the_sphere(projection):
    angles = CubeMap.texel_solid_angles(16, projection)

    assert (angles > 0).all()
    assert np.isclose(CUBE_FACE_COUNT * angles.sum(), 4 * np.pi)


@pytest.mark.parametrize("projection", list(CubeProjection))
def test_the_smallest_texel_width_is_the_shortest_edge_of_the_face(projection):
    size = 32
    edges = np.linspace(-1, 1, size + 1)
    corners = CubeMap._direction(FRONT, edges[None, :], edges[:, None], projection)
    along_rows = np.arccos(np.clip((corners[:, 1:] * corners[:, :-1]).sum(axis=-1), -1, 1))
    along_columns = np.arccos(np.clip((corners[1:] * corners[:-1]).sum(axis=-1), -1, 1))

    assert np.isclose(CubeMap.smallest_texel_width(size, projection), min(along_rows.min(), along_columns.min()))


def direction_cube(size, projection):
    faces = [Raster(CubeMap.texel_directions(face, size, projection)) for face in range(CUBE_FACE_COUNT)]
    return CubeMap(faces, projection)


# A border texel continues its face's grid past the edge: on the adjacent face it is the texel as far from the edge,
# so its direction stays within a texel of where the grid would put it.
@pytest.mark.parametrize("projection", list(CubeProjection))
def test_borders_continue_across_face_edges(projection):
    size, border = 16, 2
    padded = direction_cube(size, projection).padded(border)
    coordinates = (np.arange(-border, size + border) + 0.5) / size * 2 - 1
    width = CubeMap.smallest_texel_width(size, projection)

    for face in range(CUBE_FACE_COUNT):
        inside = np.abs(coordinates) < 1
        one_edge = inside[:, None] ^ inside[None, :]
        nominal = CubeMap._direction(face, coordinates[None, :], coordinates[:, None], projection)
        angles = np.arccos(np.clip((padded[face] * nominal).sum(axis=-1), -1, 1))

        assert (angles[one_edge] < 2 * width).all()


def test_a_corner_border_texel_averages_the_three_faces():
    size, border = 4, 1
    faces = [Raster(np.full((size, size, 1), float(face))) for face in range(CUBE_FACE_COUNT)]
    padded = CubeMap(faces, CubeProjection.EQUAL_ANGLE).padded(border)

    # The top left corner of +X meets +Y and +Z.
    assert np.isclose(padded[FRONT, 0, 0, 0], (FRONT + RIGHT + TOP) / 3)
    assert (padded[FRONT, border:-border, border:-border] == FRONT).all()


def test_the_area_mean_weights_texels_by_their_ground():
    size = 8
    faces = [Raster(np.full((size, size, 1), 1.0 if face == TOP else 0.0)) for face in range(CUBE_FACE_COUNT)]

    assert np.isclose(CubeMap(faces, CubeProjection.EQUAL_ANGLE).area_mean()[0], 1 / 6)
