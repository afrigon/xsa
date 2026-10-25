from pathlib import Path

from operations.displacement_range import DisplacementRange


# Writes a surface's definition, `resources/surfaces/<name>.kdl`.
class SurfaceWriter:
    def __init__(self, directory: Path):
        self.directory = directory

    def write(
        self,
        name: str,
        textures: dict[str, str],
        size: float,
        displacement: DisplacementRange,
    ) -> None:
        lines = [f'{key} "{texture}"' for key, texture in textures.items()]
        lines.append(f"size {size!r}")
        lines.append(f"displacement low={displacement.low!r} high={displacement.high!r}")
        self.directory.mkdir(parents=True, exist_ok=True)
        (self.directory / f"{name}.kdl").write_text("\n".join(lines) + "\n")
