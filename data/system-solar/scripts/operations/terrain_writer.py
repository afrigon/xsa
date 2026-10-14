import math
from pathlib import Path

from operations.raster import Raster
from operations.texture_writer import TextureWriter


class TerrainWriter:
    def __init__(self, terrains: Path):
        self.terrains = terrains

    # A body's height map in meters relative to its radius, and the file describing it; the range is the map's own,
    # rounded outward to whole meters.
    def write(self, body: str, heights: Raster) -> Path:
        minimum = math.floor(float(heights.pixels.min()))
        maximum = math.ceil(float(heights.pixels.max()))
        TextureWriter.height(minimum, maximum).write(self.terrains / f"{body}.ktx2", [heights])
        path = self.terrains / f"{body}.kdl"
        path.write_text(f'height-map "{body}"\nminimum {minimum}\nmaximum {maximum}\n')
        return path
