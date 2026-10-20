import logging
from collections.abc import Iterable
from pathlib import Path

import numpy as np

from operations.array_writer import ArrayWriter
from operations.build_context import BuildContext
from operations.credit import Credit
from operations.displacement_range import DisplacementRange
from operations.mip_chain import MipChain
from operations.polyhaven_set import PolyhavenSet
from operations.recipe import Recipe
from operations.surface_variant import SurfaceVariant

BLOCK_SIZE = 4
AVERAGE_COLOR_KEY = "xsa.averageColor"
SLOPE_SPREAD_KEY = "xsa.slopeSpread"

logger = logging.getLogger(__name__)


# A surface: the texture sets that are its interchangeable variants, written as colour, normal and displacement
# arrays at the build's density, with its definition.
class SurfaceRecipe(Recipe):
    sets: list[PolyhavenSet]
    displacement: DisplacementRange

    def build(self, context: BuildContext) -> None:
        pixels = self._pixels(context)
        variants = []

        for polyhaven_set in self.sets:
            logger.info("%s: resampling %s to %d pixels", self.name, polyhaven_set.id, pixels)
            variants.append(SurfaceVariant.read(polyhaven_set, pixels, context.cache))

        displacements = [variant.displacement for variant in variants]
        lowest = min(displacement.min() for displacement in displacements)
        highest = max(displacement.max() for displacement in displacements)
        color_chains = [MipChain(variant.color) for variant in variants]
        slope_chains = [MipChain(variant.slopes) for variant in variants]
        displacement_chains = [MipChain((displacement - lowest) / (highest - lowest)) for displacement in displacements]
        textures = {
            "color-texture": f"surfaces/{self.name}/color",
            "normal-texture": f"surfaces/{self.name}/normal",
            "displacement-texture": f"surfaces/{self.name}/displacement",
        }
        color_metadata = {AVERAGE_COLOR_KEY: self._numbers(self._average_color(color_chains))}
        normal_metadata = {SLOPE_SPREAD_KEY: self._numbers(self._slope_spread(slope_chains))}

        ArrayWriter.color().write(context.texture_path(textures["color-texture"]), color_chains, color_metadata)
        ArrayWriter.normal().write(context.texture_path(textures["normal-texture"]), slope_chains, normal_metadata)
        ArrayWriter.displacement().write(context.texture_path(textures["displacement-texture"]), displacement_chains, {})
        context.surfaces.write(self.name, textures, pixels / context.density, self.displacement)

    def credits(self, cache: Path) -> list[Credit]:
        location = f"base-surfaces/resources/textures/surfaces/{self.name}/"
        return [polyhaven_set.credit(cache, location) for polyhaven_set in self.sets]

    # The side in texels: the sets' size at the build's density, rounded to whole blocks. The definition's size is
    # this side over the density, so the density is exact and the ground size moves by under two texels.
    def _pixels(self, context: BuildContext) -> int:
        sizes = {polyhaven_set.size_meters(context.cache) for polyhaven_set in self.sets}

        if len(sizes) != 1:
            raise ValueError(f"{self.name}: its sets cover different sizes, {sorted(sizes)} m")

        return round(sizes.pop() * context.density / BLOCK_SIZE) * BLOCK_SIZE

    # Every layer covers the same area, so the array's average is the average of its layers' averages.
    @staticmethod
    def _average_color(chains: list[MipChain]) -> np.ndarray:
        return np.mean([chain.levels[0].mean(axis=(0, 1), dtype=np.float64) for chain in chains], axis=0)

    # The root mean square slope at every mip level: how steep the ground each level shows is on average.
    @staticmethod
    def _slope_spread(chains: list[MipChain]) -> list[float]:
        return [
            float(np.sqrt(np.mean([(chain.levels[level].astype(np.float64) ** 2).sum(axis=-1).mean() for chain in chains])))
            for level in range(len(chains[0].levels))
        ]

    @staticmethod
    def _numbers(values: Iterable[float]) -> str:
        return " ".join(repr(float(value)) for value in values)
