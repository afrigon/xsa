# The aperture stop: blades whose edges are arcs, so the opening is a slightly rounded regular polygon.
import numpy as np

BLADES = 9
ROTATION = np.radians(33)
# Radius of each blade edge's arc, in units of the opening's circumradius; infinite would be straight edges.
BLADE_CURVATURE_RADIUS = 3.0

def inside(x, y, circumradius):
    """Whether points of the stop plane pass through the opening."""
    apothem = circumradius * np.cos(np.pi / BLADES)
    arc_radius = BLADE_CURVATURE_RADIUS * circumradius
    passes = np.ones(np.shape(x), dtype=bool)
    for blade in range(BLADES):
        normal = ROTATION + np.pi / BLADES + blade * 2 * np.pi / BLADES
        # The arc bulges outward: its center lies inward, so the edge midpoint stays at the apothem.
        center = (apothem - arc_radius) * np.array([np.cos(normal), np.sin(normal)])
        passes &= (x - center[0]) ** 2 + (y - center[1]) ** 2 < arc_radius ** 2
    return passes
