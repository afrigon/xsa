from abc import ABC, abstractmethod

from operations.build_context import BuildContext
from operations.credit import Credit


class Recipe(ABC):
    name: str
    credits: list[Credit]

    @abstractmethod
    def build(self, context: BuildContext) -> None: ...
