from dataclasses import dataclass


@dataclass(frozen=True)
class Credit:
    heading: str
    text: str
