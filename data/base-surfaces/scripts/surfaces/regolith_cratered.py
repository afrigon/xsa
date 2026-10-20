from operations.displacement_range import DisplacementRange
from operations.polyhaven_set import PolyhavenSet
from operations.surface_recipe import SurfaceRecipe


# Regolith pitted by small impacts.
class RegolithCratered(SurfaceRecipe):
    name = "regolith-cratered"
    sets = [PolyhavenSet("moon_meteor_01")]
    displacement = DisplacementRange(low=0.0, high=0.15)
