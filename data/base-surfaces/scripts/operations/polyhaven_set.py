import hashlib
import json
import logging
import urllib.request
from dataclasses import dataclass
from pathlib import Path

from operations.credit import Credit

API = "https://api.polyhaven.com"
USER_AGENT = "xsa base-surfaces scripts"
CHUNK_BYTES = 1 << 24
MILLIMETERS_PER_METER = 1000.0
PIXELS_PER_RESOLUTION_STEP = 1024
FORMAT = "exr"

logger = logging.getLogger(__name__)


# A texture set published by Poly Haven: its maps, physical size and authors, from the Poly Haven API. Every response
# and download is kept in the cache, so a build after the first works from the files it already checked.
@dataclass(frozen=True)
class PolyhavenSet:
    id: str

    # The side of the square the set covers on the ground.
    def size_meters(self, cache: Path) -> float:
        width, height = self._info(cache)["dimensions"]

        if width != height:
            raise ValueError(f"{self.id}: {width} × {height} mm, expected a square")

        return width / MILLIMETERS_PER_METER

    # The map at the smallest resolution at least `pixels` wide, in OpenEXR.
    def fetch_map(self, map_name: str, pixels: int, cache: Path) -> Path:
        resolutions = self._files(cache)[map_name]
        sizes = {int(name.removesuffix("k")) * PIXELS_PER_RESOLUTION_STEP: name for name in resolutions}
        fitting = [size for size in sizes if size >= pixels]

        if not fitting:
            raise ValueError(f"{self.id}: no {map_name} map is {pixels} pixels wide")

        file = resolutions[sizes[min(fitting)]][FORMAT]
        path = self._directory(cache) / file["url"].rsplit("/", 1)[-1]
        stamp = path.with_name(path.name + ".md5")

        if path.exists() and stamp.exists() and stamp.read_text() == file["md5"]:
            return path

        self._download(file["url"], path)
        digest = hashlib.md5(path.read_bytes()).hexdigest()

        if digest != file["md5"]:
            raise ValueError(f"{path.name}: MD5 {digest}, expected {file['md5']}")

        stamp.write_text(file["md5"])
        return path

    def credit(self, cache: Path, location: str) -> Credit:
        info = self._info(cache)
        authors = ", ".join(f"{name} ({role.lower()})" for name, role in info["authors"].items())
        return Credit(
            f"{info['name']} ({location})",
            f"From Poly Haven (https://polyhaven.com/a/{self.id}), by {authors}. Licensed CC0 1.0 Universal, dedicated "
            "to the public domain (https://creativecommons.org/publicdomain/zero/1.0/). Resampled, with the colour, "
            "OpenGL normal and displacement maps compressed into texture arrays.",
        )

    def _info(self, cache: Path) -> dict:
        return self._api(f"info/{self.id}", self._directory(cache) / "info.json")

    def _files(self, cache: Path) -> dict:
        return self._api(f"files/{self.id}", self._directory(cache) / "files.json")

    def _directory(self, cache: Path) -> Path:
        return cache / "polyhaven" / self.id

    def _api(self, endpoint: str, path: Path) -> dict:
        if not path.exists():
            self._download(f"{API}/{endpoint}", path)

        return json.loads(path.read_text())

    @staticmethod
    def _download(url: str, path: Path) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        partial = path.with_name(path.name + ".part")
        logger.info("downloading %s", url)
        request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})

        with urllib.request.urlopen(request) as response, partial.open("wb") as file:
            while chunk := response.read(CHUNK_BYTES):
                file.write(chunk)

        partial.rename(path)
