from collections.abc import Sequence
from pathlib import Path

import numpy as np

from operations.hapke_material import HapkeMaterial


class MaterialWriter:
    def __init__(self, materials: Path):
        self.materials = materials

    def write_base_color(self, body: str, base_color: Sequence[float]) -> Path:
        path = self.materials / f"{body}.kdl"
        path.write_text(self.header("lambert", base_color))
        return path

    def write_textured(
        self,
        body: str,
        base_color: np.ndarray,
        color: str,
        normal: str,
        hapke: HapkeMaterial | None = None,
    ) -> Path:
        path = self.materials / f"{body}.kdl"
        shader = "lambert" if hapke is None else "hapke"
        text = self.header(shader, base_color)
        text += f'color "{color}"\nnormal "{normal}"\n'

        if hapke is not None:
            text += f'scattering "{hapke.scattering}"\nsurge "{hapke.surge}"\n'
            text += f"macroscopic-roughness {hapke.macroscopic_roughness:.4f}\nporosity {hapke.porosity:.4f}\n"

        path.write_text(text)
        return path

    @staticmethod
    def header(shader: str, base_color: Sequence[float]) -> str:
        color = " ".join(f"{component:.4f}" for component in base_color)
        return f'shader "base:terrain/{shader}"\nbase-color {color}\n'
