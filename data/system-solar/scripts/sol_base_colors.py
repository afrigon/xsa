# /// script
# requires-python = ">=3.12"
# dependencies = ["numpy", "texture2ddecoder"]
# ///
"""Write each planet material's base color: the area-weighted average of its Sol color map, in linear light."""

import re
import struct
from pathlib import Path

import numpy as np
import texture2ddecoder

PACK = Path(__file__).resolve().parent.parent
SOL_PACK = PACK.parent / "system-solar-sol"
MATERIALS = PACK / "system-solar" / "resources" / "materials"
SOL_MATERIALS = SOL_PACK / "system-solar" / "resources" / "materials"
SOL_TEXTURES = SOL_PACK / "system-solar" / "resources" / "textures" / "bodies"

DDS_MAGIC = b"DDS "
DDS_HEADER_SIZE = 128
DX10_HEADER_SIZE = 20
HEIGHT_OFFSET = 12
WIDTH_OFFSET = 16
FOURCC_OFFSET = 84
DXGI_FORMAT_OFFSET = DDS_HEADER_SIZE
DX10_FOURCC = b"DX10"
BC7_FORMATS = {98, 99}
BC7_BLOCK_BYTES = 16
BC7_BLOCK_SIZE = 4

BASE_COLOR = re.compile(r"^base-color .*\n", re.MULTILINE)
SHADER = re.compile(r"^shader .*\n", re.MULTILINE)


def read_bc7(path: Path) -> np.ndarray:
    data = path.read_bytes()

    if data[:4] != DDS_MAGIC or data[FOURCC_OFFSET : FOURCC_OFFSET + 4] != DX10_FOURCC:
        raise ValueError(f"{path}: not a DDS file with a DX10 header")

    (dxgi_format,) = struct.unpack_from("<I", data, DXGI_FORMAT_OFFSET)

    if dxgi_format not in BC7_FORMATS:
        raise ValueError(f"{path}: DXGI format {dxgi_format} is not BC7")

    height, width = struct.unpack_from("<II", data, HEIGHT_OFFSET)
    start = DDS_HEADER_SIZE + DX10_HEADER_SIZE
    size = -(-width // BC7_BLOCK_SIZE) * -(-height // BC7_BLOCK_SIZE) * BC7_BLOCK_BYTES
    bgra = texture2ddecoder.decode_bc7(data[start : start + size], width, height)
    return np.frombuffer(bgra, np.uint8).reshape(height, width, 4)[..., 2::-1]


def linear(srgb: np.ndarray) -> np.ndarray:
    return np.where(srgb <= 0.04045, srgb / 12.92, ((srgb + 0.055) / 1.055) ** 2.4)


# Equirectangular rows cover less area toward the poles, in proportion to the cosine of their latitude.
def area_weighted_mean(rgb: np.ndarray) -> np.ndarray:
    height = rgb.shape[0]
    latitudes = (np.arange(height) + 0.5) / height * np.pi - np.pi / 2
    weights = np.cos(latitudes)
    rows = linear(rgb.astype(np.float64) / 255.0).mean(axis=1)
    return (rows * weights[:, None]).sum(axis=0) / weights.sum()


def with_base_color(material: str, color: np.ndarray) -> str:
    line = "base-color " + " ".join(f"{component:.4f}" for component in color) + "\n"
    material = BASE_COLOR.sub("", material)
    shader = SHADER.search(material)

    if shader is None:
        raise ValueError("a material without a shader")

    return material[: shader.end()] + line + material[shader.end() :]


def main() -> None:
    for sol_material in sorted(SOL_MATERIALS.glob("*.kdl")):
        body = sol_material.stem
        color = area_weighted_mean(read_bc7(SOL_TEXTURES / body / "color.dds"))
        sol_material.write_text(with_base_color(sol_material.read_text(), color))
        (MATERIALS / sol_material.name).write_text(with_base_color('shader "base:planet"\n', color))
        print(f"{body}: {' '.join(f'{component:.4f}' for component in color)}")


if __name__ == "__main__":
    main()
