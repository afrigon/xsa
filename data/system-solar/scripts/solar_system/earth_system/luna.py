from operations.build_context import BuildContext
from operations.credit import Credit
from operations.raster import Raster, SourceLayout
from operations.recipe import Recipe
from operations.source import Source
from operations.texture_writer import TextureWriter

# The reference sphere of every LRO data product.
RADIUS = 1737400.0
METERS_PER_KILOMETER = 1000.0
CGI_MOON_KIT = SourceLayout(left_edge_longitude=-180.0, north_up=True)


class Luna(Recipe):
    name = "luna"
    color = Source(
        "https://svs.gsfc.nasa.gov/vis/a000000/a004700/a004720/lroc_color_16bit_srgb.tif",
        "4be1ed002f72c2a96ab245af64cf8a84c560d5498a2a7e7e3b568d38e2833752",
    )
    elevation = Source(
        "https://svs.gsfc.nasa.gov/vis/a000000/a004700/a004720/ldem_64.tif",
        "af55ea2ed9173d1744b98ac4205501bd0bb0dcfc6ac49a11473a411dc64afd90",
    )
    credits = [
        Credit(
            "Moon textures (system-solar/resources/textures/bodies/luna/)",
            "CGI Moon Kit, NASA's Scientific Visualization Studio (https://svs.gsfc.nasa.gov/4720): color from the "
            "Lunar Reconnaissance Orbiter Camera Hapke-normalized Wide Angle Camera mosaic, polar latitudes from the "
            "Lunar Orbiter Laser Altimeter albedo map; elevation from the Lunar Orbiter Laser Altimeter gridded data. "
            "Used under the NASA media usage guidelines; resampled, and the normal map derived from the elevation.",
        )
    ]

    def build(self, context: BuildContext) -> None:
        color_path = self.color.fetch(context.cache)
        color = Raster.read(color_path, context.output_width(color_path), CGI_MOON_KIT, encoded_srgb=True)
        TextureWriter.color().write(context.texture_path("bodies/luna/color"), [color])

        elevation_path = self.elevation.fetch(context.cache)
        width = context.output_width(elevation_path)
        heights = Raster.read(elevation_path, width, CGI_MOON_KIT, scale=METERS_PER_KILOMETER)
        TextureWriter.normal().write(context.texture_path("bodies/luna/normal"), [heights.normals(RADIUS)])

        context.materials.write_planet("luna", color.area_mean(), "bodies/luna/color", "bodies/luna/normal")
