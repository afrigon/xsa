from dataclasses import dataclass


# Hapke 2012's parameters other than the albedo: those that vary over the ground as texture ids, the scattering map
# holding the phase function (b, c) and the surge map the opposition effects (B_S0, h_S, B_C0, h_C) in that channel
# order; those that hold over the whole body as values, the macroscopic roughness θ̄ in degrees and the porosity K.
@dataclass(frozen=True)
class HapkeMaterial:
    scattering: str
    surge: str
    macroscopic_roughness: float
    porosity: float
