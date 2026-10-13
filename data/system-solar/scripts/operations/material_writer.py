from pathlib import Path

import numpy as np


class MaterialWriter:
    def __init__(self, materials: Path):
        self.materials = materials

    def write_planet(self, body: str, base_color: np.ndarray, color_texture: str, normal_texture: str) -> Path:
        path = self.materials / f"{body}.kdl"
        color = " ".join(f"{component:.4f}" for component in base_color)
        path.write_text(
            'shader "base:planet"\n'
            f"base-color {color}\n"
            f'color-texture "{color_texture}"\n'
            f'normal-texture "{normal_texture}"\n'
        )
        return path
