from dataclasses import dataclass
from pathlib import Path

from operations.surface_writer import SurfaceWriter

NAMESPACE = "base-surfaces"
TEXTURE_EXTENSION = ".ktx2"


@dataclass(frozen=True)
class BuildContext:
    pack: Path
    # Texels per meter of ground.
    density: int

    @property
    def cache(self) -> Path:
        return self.pack / "cache"

    @property
    def resources(self) -> Path:
        return self.pack / NAMESPACE / "resources"

    @property
    def surfaces(self) -> SurfaceWriter:
        return SurfaceWriter(self.resources / "surfaces")

    def texture_path(self, texture: str) -> Path:
        return self.resources / "textures" / f"{texture}{TEXTURE_EXTENSION}"
