from dataclasses import dataclass


# Hapke 2012's parameters other than the albedo: the particulate medium (b, c, θ̄ in degrees, porosity K) and the
# opposition effects (B_S0, h_S, B_C0, h_C), in that channel order.
@dataclass(frozen=True)
class HapkeMaterial:
    medium_texture: str
    opposition_texture: str
