import math
from pathlib import Path

from operations.cube_map import CubeMap
from operations.texture_writer import TextureWriter


class TerrainWriter:
    def __init__(self, terrains: Path):
        self.terrains = terrains

    # A body's height map in meters relative to its radius, and the file describing it with the range the map's
    # heights span, rounded outward to whole meters.
    def write(self, body: str, heights: CubeMap) -> Path:
        minimum = math.floor(min(float(face.pixels.min()) for face in heights.faces))
        maximum = math.ceil(max(float(face.pixels.max()) for face in heights.faces))
        TextureWriter.height().write(self.terrains / f"{body}.ktx2", heights.faces)
        path = self.terrains / f"{body}.kdl"
        path.write_text(f'height-map "{body}"\nminimum {minimum}\nmaximum {maximum}\n')
        return path
