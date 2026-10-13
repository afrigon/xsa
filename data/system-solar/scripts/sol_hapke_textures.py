# Converts the system-solar-sol overlay's Hapke maps and materials to the planet shader's layout: a medium texture
# (b, c, θ̄ in degrees, porosity K) and an opposition texture (B_S0, h_S, B_C0, h_C), from Sol's 8-bit scatter and
# surge maps and its per-body roughness and porosity.

import re
import struct
from pathlib import Path

import numpy as np
import texture2ddecoder

from operations.raster import Raster
from operations.texture_writer import TextureWriter

PACK = Path(__file__).resolve().parent.parent
SOL = PACK.parent / "system-solar-sol" / "system-solar" / "resources"
DDS_HEADER_SIZE = 128
DX10_HEADER_SIZE = 20
HEIGHT_OFFSET = 12
UNORM8_MAX = 255.0
HAPKE_NODE = re.compile(r"^hapke .*$", re.MULTILINE)


def read_bc7(path: Path) -> np.ndarray:
    data = path.read_bytes()
    height, width = struct.unpack_from("<II", data, HEIGHT_OFFSET)
    start = DDS_HEADER_SIZE + DX10_HEADER_SIZE
    bgra = texture2ddecoder.decode_bc7(data[start : start + width * height], width, height)
    return np.frombuffer(bgra, np.uint8).reshape(height, width, 4)[..., [2, 1, 0, 3]] / UNORM8_MAX


def number(node: str, key: str) -> float:
    return float(re.search(rf"\b{key}=([-0-9.]+)", node).group(1))


def texture(node: str, key: str) -> str:
    return re.search(rf'\b{key}="([^"]+)"', node).group(1)


def convert(material: Path) -> None:
    text = material.read_text()
    node = HAPKE_NODE.search(text)

    if node is None or "scatter-texture" not in node.group(0):
        return

    scatter_path = SOL / "textures" / f"{texture(node.group(0), 'scatter-texture')}.dds"
    surge_path = SOL / "textures" / f"{texture(node.group(0), 'surge-texture')}.dds"
    # Sol encodes c as (c + 1) / 2 and both opposition amplitudes halved, to fit 8-bit channels.
    scatter = read_bc7(scatter_path)
    surge = read_bc7(surge_path)
    roughness = np.full(scatter.shape[:2], number(node.group(0), "roughness"))
    porosity = np.full(scatter.shape[:2], number(node.group(0), "porosity"))
    medium = np.stack([scatter[..., 1], 2 * scatter[..., 2] - 1, roughness, porosity], axis=-1)
    opposition = np.stack([2 * surge[..., 0], surge[..., 1], 2 * surge[..., 2], surge[..., 3]], axis=-1)

    body = scatter_path.parent
    TextureWriter.values().write(body / "hapke-medium.ktx2", [Raster(medium.astype(np.float32))])
    TextureWriter.values().write(body / "hapke-opposition.ktx2", [Raster(opposition.astype(np.float32))])
    textures = body.relative_to(SOL / "textures").as_posix()
    replacement = f'hapke medium-texture="{textures}/hapke-medium" opposition-texture="{textures}/hapke-opposition"'
    material.write_text(text[: node.start()] + replacement + text[node.end() :])
    scatter_path.unlink()
    surge_path.unlink()
    print(f"{material.stem}: {scatter.shape[1]}×{scatter.shape[0]}")


def main() -> None:
    for material in sorted((SOL / "materials").glob("*.kdl")):
        convert(material)


if __name__ == "__main__":
    main()
