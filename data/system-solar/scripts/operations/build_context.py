from dataclasses import dataclass
from pathlib import Path

from operations.material_writer import MaterialWriter
from operations.terrain_writer import TerrainWriter


@dataclass(frozen=True)
class BuildContext:
    pack: Path
    width: int

    @property
    def cache(self) -> Path:
        return self.pack / "cache"

    @property
    def resources(self) -> Path:
        return self.pack / "system-solar" / "resources"

    @property
    def materials(self) -> MaterialWriter:
        return MaterialWriter(self.resources / "materials")

    @property
    def terrains(self) -> TerrainWriter:
        return TerrainWriter(self.pack / "system-solar" / "data" / "terrains")

    def texture_path(self, texture: str) -> Path:
        return self.resources / "textures" / f"{texture}.ktx2"

    # A surface layer's cube map, which packaging converts into the streamed file the game reads.
    def terrain_layer_path(self, layer: str) -> Path:
        return self.resources / "terrains" / f"{layer}.ktx2"

    # The tier's width, or the source's when it is narrower: a map is never upscaled.
    def output_width(self, native_width: int) -> int:
        width = min(self.width, native_width)
        return 1 << (width.bit_length() - 1)
