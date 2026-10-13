from operations.build_context import BuildContext
from operations.credit import Credit
from operations.cube_map import CubeMap
from operations.recipe import Recipe
from operations.source import Source
from operations.texture_writer import TextureWriter

FACE_SIZE = 4096
SAMPLES_PER_AXIS = 2


class Skybox(Recipe):
    name = "skybox"
    star_map = Source(
        "https://svs.gsfc.nasa.gov/vis/a000000/a004800/a004851/starmap_2020_16k.exr",
        "19a1351f00c386a6e5eec4d67af96d5fc71edf6a1189941579b9498b52e7589a",
    )
    credits = [
        Credit(
            "Skybox (system-solar/resources/textures/skyboxes/)",
            "Deep Star Maps 2020, NASA/Goddard Space Flight Center Scientific Visualization Studio "
            "(https://svs.gsfc.nasa.gov/4851). Gaia DR2: ESA/Gaia/DPAC. Hipparcos-2, Tycho-2: ESA. Used under the "
            "NASA media usage guidelines; the cube map is converted from the source EXR.",
        )
    ]

    def build(self, context: BuildContext) -> None:
        cube_map = CubeMap.from_sky_map(self.star_map.fetch(context.cache), FACE_SIZE, SAMPLES_PER_AXIS)
        TextureWriter.sky().write(context.texture_path("skyboxes/deep-star-maps"), cube_map.faces)
