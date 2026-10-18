from collections.abc import Sequence
from pathlib import Path

import numpy as np

from operations.hapke_material import HapkeMaterial


class MaterialWriter:
    def __init__(self, materials: Path):
        self.materials = materials

    def write_base_color(self, body: str, base_color: Sequence[float]) -> Path:
        path = self.materials / f"{body}.kdl"
        path.write_text(self.planet(base_color))
        return path

    def write_planet(
        self,
        body: str,
        base_color: np.ndarray,
        color_terrain: str,
        normal_terrain: str,
        hapke: HapkeMaterial | None = None,
    ) -> Path:
        path = self.materials / f"{body}.kdl"
        text = self.planet(base_color) + f'color-terrain "{color_terrain}"\nnormal-terrain "{normal_terrain}"\n'

        if hapke is not None:
            text += f'hapke medium-terrain="{hapke.medium_terrain}" opposition-terrain="{hapke.opposition_terrain}"\n'

        path.write_text(text)
        return path

    @staticmethod
    def planet(base_color: Sequence[float]) -> str:
        color = " ".join(f"{component:.4f}" for component in base_color)
        return f'shader "base:planet"\nbase-color {color}\n'
