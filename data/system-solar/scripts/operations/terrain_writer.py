import math
from pathlib import Path

import ckdl

from operations.cube_map import CubeMap
from operations.texture_writer import TextureWriter

KDL_VERSION = ckdl.KdlVersion.kdl_2


class TerrainWriter:
    def __init__(self, terrains: Path):
        self.terrains = terrains

    # A body's height map in meters relative to its radius, and in the body's terrain file the range its values span:
    # the map's own, rounded outward to whole meters. The rest of the terrain file is left as written.
    def write(self, body: str, heights: CubeMap) -> Path:
        minimum = math.floor(min(float(face.pixels.min()) for face in heights.faces))
        maximum = math.ceil(max(float(face.pixels.max()) for face in heights.faces))
        TextureWriter.height(minimum, maximum).write(self.terrains / f"{body}.ktx2", heights.faces)
        path = self.terrains / f"{body}.kdl"
        document = ckdl.parse(path.read_text(), version=KDL_VERSION)
        self._set(document, "minimum", minimum)
        self._set(document, "maximum", maximum)
        path.write_text(document.dump(ckdl.EmitterOptions(version=KDL_VERSION)))
        return path

    @staticmethod
    def _set(document: ckdl.Document, name: str, value: int) -> None:
        node = ckdl.Node(name, value)

        for index, existing in enumerate(document.nodes):
            if existing.name == name:
                document.nodes[index] = node
                return

        document.nodes.append(node)
