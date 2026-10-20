from operations.displacement_range import DisplacementRange
from operations.polyhaven_set import PolyhavenSet
from operations.surface_recipe import SurfaceRecipe


# Fine lunar soil, nearly self-similar across scales.
class Regolith(SurfaceRecipe):
    name = "regolith"
    sets = [PolyhavenSet("moon_01"), PolyhavenSet("moon_02")]
    displacement = DisplacementRange(low=0.0, high=0.05)
    largest_magnification = 8.0
