from pathlib import Path

import numpy as np
import OpenEXR


class ExrImage:
    # The image's single channel layer as height × width × channels floats.
    @staticmethod
    def read(path: Path) -> np.ndarray:
        with OpenEXR.File(str(path)) as file:
            channels = list(file.channels().values())

            if len(channels) != 1:
                raise ValueError(f"{path.name}: {len(channels)} channel layers, expected one")

            pixels = channels[0].pixels.astype(np.float32)

        return pixels if pixels.ndim == 3 else pixels[..., None]
