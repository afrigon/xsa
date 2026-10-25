from abc import ABC, abstractmethod
from pathlib import Path

from operations.build_context import BuildContext
from operations.credit import Credit


class Recipe(ABC):
    name: str

    @abstractmethod
    def build(self, context: BuildContext) -> None: ...

    @abstractmethod
    def credits(self, cache: Path) -> list[Credit]: ...
