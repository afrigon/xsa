from operations.displacement_range import DisplacementRange
from operations.polyhaven_set import PolyhavenSet
from operations.surface_recipe import SurfaceRecipe


# Regolith strewn with rocks.
class RegolithRocky(SurfaceRecipe):
    name = "regolith-rocky"
    sets = [PolyhavenSet("moon_meteor_02")]
    displacement = DisplacementRange(low=0.0, high=0.25)
