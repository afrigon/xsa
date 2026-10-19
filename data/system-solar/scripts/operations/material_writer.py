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
        color_texture: str,
        normal_texture: str,
        hapke: HapkeMaterial | None = None,
    ) -> Path:
        path = self.materials / f"{body}.kdl"
        shader = "lambert" if hapke is None else "hapke"
        text = self.header(shader, base_color)
        text += f'color-texture "{color_texture}"\nnormal-texture "{normal_texture}"\n'

        if hapke is not None:
            text += f'medium-texture "{hapke.medium_texture}"\nopposition-texture "{hapke.opposition_texture}"\n'

        path.write_text(text)
        return path

    @staticmethod
    def header(shader: str, base_color: Sequence[float]) -> str:
        color = " ".join(f"{component:.4f}" for component in base_color)
        return f'shader "base:{shader}"\nbase-color {color}\n'
