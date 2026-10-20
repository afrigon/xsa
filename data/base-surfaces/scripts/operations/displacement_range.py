from dataclasses import dataclass


# The heights in meters that a displacement of 0 and of 1 stand for.
@dataclass(frozen=True)
class DisplacementRange:
    low: float
    high: float
