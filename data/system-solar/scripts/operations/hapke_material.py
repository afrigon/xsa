from dataclasses import dataclass


@dataclass(frozen=True)
class HapkeMaterial:
    scatter_texture: str
    surge_texture: str
    # Hapke 2012's parameters other than the albedo: the particulate medium (b, c, θ̄ in degrees, porosity K) and the
    # opposition effects (B_S0, h_S, B_C0, h_C), in that channel order.
    medium_texture: str
    opposition_texture: str
    porosity: float
    roughness_degrees: float
