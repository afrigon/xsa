from pathlib import Path

import numpy as np

from operations.hapke_material import HapkeMaterial

# Sol's look trims, which only its own Hapke variant reads: 1 leaves physical data as it is.
NEUTRAL_TRIM = 1


class MaterialWriter:
    def __init__(self, materials: Path):
        self.materials = materials

    def write_planet(
        self,
        body: str,
        base_color: np.ndarray,
        color_texture: str,
        normal_texture: str,
        hapke: HapkeMaterial | None = None,
    ) -> Path:
        path = self.materials / f"{body}.kdl"
        color = " ".join(f"{component:.4f}" for component in base_color)
        text = (
            'shader "base:planet"\n'
            f"base-color {color}\n"
            f'color-texture "{color_texture}"\n'
            f'normal-texture "{normal_texture}"\n'
        )

        if hapke is not None:
            text += (
                f'hapke scatter-texture="{hapke.scatter_texture}" surge-texture="{hapke.surge_texture}" '
                f"porosity={hapke.porosity:g} roughness={hapke.roughness_degrees:g} "
                f"blend={NEUTRAL_TRIM} light-boost={NEUTRAL_TRIM} gamma-boost={NEUTRAL_TRIM}\n"
            )

        path.write_text(text)
        return path
