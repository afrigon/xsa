from enum import Enum

import numpy as np


# How a face coordinate s in -1..1 maps to the gnomonic plane coordinate the face selection table works in. Gnomonic
# is what the hardware samples; equal-angle spaces texels by equal angles, so their ground size varies far less.
class CubeProjection(Enum):
    GNOMONIC = "gnomonic"
    EQUAL_ANGLE = "equal-angle"

    def to_plane(self, face_coordinate: np.ndarray) -> np.ndarray:
        if self is CubeProjection.EQUAL_ANGLE:
            return np.tan(np.pi / 4 * face_coordinate)

        return face_coordinate

    def from_plane(self, plane_coordinate: np.ndarray) -> np.ndarray:
        if self is CubeProjection.EQUAL_ANGLE:
            return np.arctan(plane_coordinate) * 4 / np.pi

        return plane_coordinate
