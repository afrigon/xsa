import numpy as np
import pytest

from operations.cube_map import CUBE_FACE_COUNT, FACE_BASES, CubeMap
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


def height_cube(size, height):
    projection = CubeProjection.EQUAL_ANGLE
    faces = [Raster(height(CubeMap.texel_directions(face, size, projection))[..., None]) for face in range(CUBE_FACE_COUNT)]
    return CubeMap(faces, projection)


def test_a_sphere_has_upright_normals():
    normals = height_cube(16, lambda directions: np.zeros(directions.shape[:-1])).normals(1000.0)

    assert all(np.allclose(face.pixels, 0, atol=1e-6) for face in normals.faces)


# Ground rising toward +Z: on +X, the face's rows run toward -Y and its columns toward -Z, so the normal leans along
# the columns, away from the rise.
def test_a_slope_tilts_the_normal_away_from_the_rise():
    radius = 1000.0
    normals = height_cube(64, lambda directions: 10.0 * directions[..., 2]).normals(radius)
    center = normals.faces[FRONT].pixels[31:33, 31:33]

    assert np.allclose(center[..., 0], 10.0 / radius, rtol=0.05)
    assert np.allclose(center[..., 1], 0, atol=1e-4)


# Ground 50 x z meters high: its exact normal tilts the sphere's by the surface gradient over the radius. The texels
# along +X's left column and the +Z texels beside them, which take their neighbours across the edge, both follow it.
def test_normals_stay_exact_along_face_edges():
    size, radius = 32, 1000.0
    normals = height_cube(size, lambda directions: 50.0 * directions[..., 0] * directions[..., 2]).normals(radius)

    for face, edge in ((FRONT, (slice(None), 0)), (TOP, (slice(None), size - 1))):
        directions = CubeMap.texel_directions(face, size, CubeProjection.EQUAL_ANGLE)[edge]
        tangent = FACE_BASES[face, 1] - (directions @ FACE_BASES[face, 1])[..., None] * directions
        tangent /= np.linalg.norm(tangent, axis=-1, keepdims=True)
        bitangent = np.cross(tangent, directions)
        x, y = normals.faces[face].pixels[edge][..., 0:1], normals.faces[face].pixels[edge][..., 1:2]
        world = x * tangent + y * bitangent + np.sqrt(1 - x**2 - y**2) * directions
        gradient = 50.0 * np.stack([directions[:, 2], np.zeros(size), directions[:, 0]], axis=-1)
        gradient -= (gradient * directions).sum(-1, keepdims=True) * directions
        exact = directions - gradient / radius
        exact /= np.linalg.norm(exact, axis=-1, keepdims=True)

        assert np.allclose(world, exact, atol=1e-3)
