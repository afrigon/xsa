from dataclasses import dataclass


@dataclass(frozen=True)
class HapkeMaterial:
    scatter_texture: str
    surge_texture: str
    porosity: float
    roughness_degrees: float
