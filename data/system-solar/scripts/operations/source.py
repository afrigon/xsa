import hashlib
import logging
import urllib.request
from dataclasses import dataclass
from pathlib import Path

CHUNK_BYTES = 1 << 24
LOG_EVERY_BYTES = 1 << 28

logger = logging.getLogger(__name__)


@dataclass(frozen=True)
class Source:
    url: str
    sha256: str

    @property
    def file_name(self) -> str:
        return self.url.rsplit("/", 1)[-1]

    # Downloads into the cache once; a stamp next to the file records that its hash was checked.
    def fetch(self, cache: Path) -> Path:
        path = cache / self.file_name
        stamp = path.with_name(path.name + ".sha256")

        if path.exists() and stamp.exists() and stamp.read_text() == self.sha256:
            return path

        if not path.exists():
            self._download(path)

        digest = self._digest(path)

        if digest != self.sha256:
            raise ValueError(f"{path.name}: SHA-256 {digest}, expected {self.sha256}")

        stamp.write_text(self.sha256)
        return path

    def _download(self, path: Path) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        partial = path.with_name(path.name + ".part")
        logger.info("downloading %s", self.url)

        with urllib.request.urlopen(self.url) as response, partial.open("wb") as file:
            received = 0

            while chunk := response.read(CHUNK_BYTES):
                file.write(chunk)
                received += len(chunk)

                if received % LOG_EVERY_BYTES < len(chunk):
                    logger.info("%s: %d MB", path.name, received >> 20)

        partial.rename(path)

    @staticmethod
    def _digest(path: Path) -> str:
        digest = hashlib.sha256()

        with path.open("rb") as file:
            while chunk := file.read(CHUNK_BYTES):
                digest.update(chunk)

        return digest.hexdigest()
