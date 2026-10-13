# Converts the system-solar-sol overlay's Hapke maps into the half-float layout the planet shader reads.

import struct
from pathlib import Path

import numpy as np
import texture2ddecoder

from operations.raster import Raster
from operations.texture_writer import TextureWriter

PACK = Path(__file__).resolve().parent.parent
SOL_TEXTURES = PACK.parent / "system-solar-sol" / "system-solar" / "resources" / "textures" / "bodies"
DDS_HEADER_SIZE = 128
DX10_HEADER_SIZE = 20
HEIGHT_OFFSET = 12
UNORM8_MAX = 255.0


def read_bc7(path: Path) -> np.ndarray:
    data = path.read_bytes()
    height, width = struct.unpack_from("<II", data, HEIGHT_OFFSET)
    start = DDS_HEADER_SIZE + DX10_HEADER_SIZE
    bgra = texture2ddecoder.decode_bc7(data[start : start + width * height], width, height)
    return np.frombuffer(bgra, np.uint8).reshape(height, width, 4)[..., [2, 1, 0, 3]] / UNORM8_MAX


# Sol encodes c as (c + 1) / 2 and both opposition amplitudes halved, to fit 8-bit channels.
def decode(name: str, encoded: np.ndarray) -> np.ndarray:
    if name == "scatter":
        return np.stack([encoded[..., 0], encoded[..., 1], 2 * encoded[..., 2] - 1], axis=-1)

    return np.stack([2 * encoded[..., 0], encoded[..., 1], 2 * encoded[..., 2], encoded[..., 3]], axis=-1)


def main() -> None:
    for name in ["scatter", "surge"]:
        for dds in sorted(SOL_TEXTURES.glob(f"*/{name}.dds")):
            values = decode(name, read_bc7(dds)).astype(np.float32)
            TextureWriter.values().write(dds.with_suffix(".ktx2"), [Raster(values)])
            dds.unlink()
            print(f"{dds.parent.name}/{name}: {values.shape[1]}×{values.shape[0]}")


if __name__ == "__main__":
    main()
