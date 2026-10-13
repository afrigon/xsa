from dataclasses import dataclass
from pathlib import Path

from operations.material_writer import MaterialWriter
from operations.raster import Raster


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

    def texture_path(self, texture: str) -> Path:
        return self.resources / "textures" / f"{texture}.ktx2"

    # The tier's width, or the source's when it is narrower: a map is never upscaled.
    def output_width(self, source: Path) -> int:
        width = min(self.width, Raster.native_width(source))
        return 1 << (width.bit_length() - 1)
