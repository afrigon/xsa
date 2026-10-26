import logging

import numpy as np

from operations.band_color import BandColor
from operations.build_context import BuildContext
from operations.color_grade import ColorGrade
from operations.coverage_fill import CoverageFill
from operations.credit import Credit
from operations.cube_accumulator import CubeAccumulator
from operations.cube_map import CUBE_FACE_COUNT, CubeMap
from operations.cube_projection import CubeProjection
from operations.equirectangular_source import EquirectangularSource
from operations.hapke_material import HapkeMaterial
from operations.hapke_model import Geometry, HapkeModel
from operations.polar_stereographic_source import PolarStereographicSource
from operations.raster import Raster, SourceLayout
from operations.recipe import Recipe
from operations.source import Source
from operations.texture_writer import TextureWriter

logger = logging.getLogger(__name__)

# The reference sphere of every LRO data product.
RADIUS = 1737400.0
METERS_PER_KILOMETER = 1000.0
# Untiered: the server and every client read the same terrain.
TERRAIN_FACE_SIZE = 4096
# The mosaic's 76 pixels per degree resolve texels of this size.
COLOR_FACE_SIZE = 4096
# The Hapke parameter maps are 1° per pixel: about one pixel per texel.
HAPKE_FACE_SIZE = 128
# Heights split each source pixel so texels catch even shares of ground: uneven shares show as ridges in the normals.
# Colour takes each mosaic pixel whole, which costs a fraction of a level on average.
TERRAIN_CELLS_PER_TEXEL = 2
COLOR_CELLS_PER_TEXEL = 1
HAPKE_CELLS_PER_TEXEL = 2
CGI_MOON_KIT = SourceLayout(left_edge_longitude=-180.0, north_up=True)
MOSAIC = "https://pds.lroc.im-ldi.com/data/LRO-L-LROC-5-RDR-V1.0/LROLRC_2001/DATA/MDR/WAC_HAPKE"
PARAMETER_MAPS = "https://pds.lroc.im-ldi.com/data/LRO-L-LROC-5-RDR-V1.0/LROLRC_2001/DATA/SDP/WAC_HAPKEPARAMMAP"
ALBEDO = "https://pds-geosciences.wustl.edu/lro/lro-l-lola-3-rdr-v1/lrolol_1xxx/data/lola_gdr/polar/float_img"
POLAR_ELEVATION = "https://pds-geosciences.wustl.edu/lro/lro-l-lola-3-rdr-v1/lrolol_1xxx/data/lola_gdr/polar/img"
# The mosaic's photometric normalization: incidence = phase = 60°, emission = 0°.
STANDARD_GEOMETRY = Geometry(incidence=np.radians(60), emission=0.0, phase=np.radians(60))
# The parameter map holds 9 parameters for each of the camera's 7 bands, bands by increasing wavelength.
CAMERA_WAVELENGTHS = [321, 360, 415, 566, 604, 643, 689]
PARAMETERS = ["w", "b", "c", "Bc0", "hc", "Bs0", "hs", "theta", "phi"]
# The band LROC's photometry uses for brightness; the surface's Hapke maps hold its parameters.
HAPKE_WAVELENGTH = 566
# The look: the measured regolith is warm (red/blue 1.58 in sRGB); Sol's artists render it close to neutral (about
# 1.08), which people recognize as the Moon. This saturation brings the map's mean ratio to Sol's.
SATURATION = 0.18
# Beyond 70° of latitude only the laser altimeter's albedo exists; it takes over across the last 10° of the mosaic.
POLAR_OVERLAP = (60.0, 70.0)


class Luna(Recipe):
    name = "luna"
    mosaic = {
        415: [
            Source(f"{MOSAIC}/WAC_HAPKE_415NM_E350N0450.IMG", "ade024198db5f988f8fa3f072246a546d4c992326323ba950cc0c7c555a292ce"),
            Source(f"{MOSAIC}/WAC_HAPKE_415NM_E350S0450.IMG", "9ee6fc25d15be88e8bea417d9e6974c3d04a0e4d692cd1c17efc195d04e88fc4"),
            Source(f"{MOSAIC}/WAC_HAPKE_415NM_E350N1350.IMG", "b83cad5549f7e0e34cc48e6eda7c05e85c9db2d9ed3cfdabb791ad4dfd3cc160"),
            Source(f"{MOSAIC}/WAC_HAPKE_415NM_E350S1350.IMG", "033980d1f681563eb39106f148626c99ef1090a4445796271be8ba5c7349f471"),
            Source(f"{MOSAIC}/WAC_HAPKE_415NM_E350N2250.IMG", "84116993dc3f92edf2d8d1070371caea4558aa23178e85c28906b3163ae7e57a"),
            Source(f"{MOSAIC}/WAC_HAPKE_415NM_E350S2250.IMG", "336ab165abad52e15ad8e8d26c279be095271b9f1a408e99dbde3164d80f9467"),
            Source(f"{MOSAIC}/WAC_HAPKE_415NM_E350N3150.IMG", "407855d1b528409187ace2f0c7feb04d68da721ea67cc227249ca46ada21c364"),
            Source(f"{MOSAIC}/WAC_HAPKE_415NM_E350S3150.IMG", "78bbc3fdacd5f855d7004c66799f4f5a8db5a227a9c3af5e6cccd8094898e766"),
        ],
        566: [
            Source(f"{MOSAIC}/WAC_HAPKE_566NM_E350N0450.IMG", "84517fb17f897edfa2c587ad5eccbf9cd2a5181882ae244a2aae2c23aac225bd"),
            Source(f"{MOSAIC}/WAC_HAPKE_566NM_E350S0450.IMG", "524dec9b183d10b7a7c78965bab3194baf12e7372f79fb4cbecb182dccd518ee"),
            Source(f"{MOSAIC}/WAC_HAPKE_566NM_E350N1350.IMG", "751cd015f0724641c68b9cc630b53dad33d9226841471423440e97f8e13f1b59"),
            Source(f"{MOSAIC}/WAC_HAPKE_566NM_E350S1350.IMG", "e63b91e7d7575a59a928b64de9da9b7d6aab21a0bffb483e84d97611975967c5"),
            Source(f"{MOSAIC}/WAC_HAPKE_566NM_E350N2250.IMG", "b8b1cf48e37b56c352e90866593d58428a0f1fe5fc8add981ab4e80f073dc28f"),
            Source(f"{MOSAIC}/WAC_HAPKE_566NM_E350S2250.IMG", "5dc018cd3e59a55584910780e6a9a81ce83b163b38f99656cfa361c875a4cdda"),
            Source(f"{MOSAIC}/WAC_HAPKE_566NM_E350N3150.IMG", "4a676801e8ee8c6ab8ba96e6c3e286db29ea6988b7bcfa16e7a903522db1b60a"),
            Source(f"{MOSAIC}/WAC_HAPKE_566NM_E350S3150.IMG", "0aabaf2dfa0d84b2d73330b4e073fc8145294bc4cfe41b2faa6c20c676782d97"),
        ],
        604: [
            Source(f"{MOSAIC}/WAC_HAPKE_604NM_E350N0450.IMG", "ffed468e36286d6cbd03d34626b0e9cb794d2feaa7aadd64dbe7d066f5326d32"),
            Source(f"{MOSAIC}/WAC_HAPKE_604NM_E350S0450.IMG", "dd2cbd8da623cfbd72d7e842fefcc21c2cfe51bdff6a8660d2d3e429be504879"),
            Source(f"{MOSAIC}/WAC_HAPKE_604NM_E350N1350.IMG", "54e76709895731dfb9416b355f73f61f51215adad6542270b380013a9b54a0b5"),
            Source(f"{MOSAIC}/WAC_HAPKE_604NM_E350S1350.IMG", "4190933ac3ada3c1caa23b9217aa40fcb52d485ac6e46199f249742c84d9e777"),
            Source(f"{MOSAIC}/WAC_HAPKE_604NM_E350N2250.IMG", "dc59a651cc8899cde49d47a066906fb8377d63176b1afcf3737224399541dc3c"),
            Source(f"{MOSAIC}/WAC_HAPKE_604NM_E350S2250.IMG", "aca2431bbebd0e330f96635f4c9c242a2ff8d161a9a8ef65fb7d6f109ac98cb3"),
            Source(f"{MOSAIC}/WAC_HAPKE_604NM_E350N3150.IMG", "50a9f8798174646d3f59ee914a74eac543bd478ffd8b24aa67a06c9da3840bdb"),
            Source(f"{MOSAIC}/WAC_HAPKE_604NM_E350S3150.IMG", "3a5981b89c3b2f95e908d1eb3210ef3ebaaa3b582e352cadcee694cbcd723bd6"),
        ],
        643: [
            Source(f"{MOSAIC}/WAC_HAPKE_643NM_E350N0450.IMG", "1612ded2849aa784be4b1d7d8b46d6778e342a7838f5f59cd55373ac7347f2ec"),
            Source(f"{MOSAIC}/WAC_HAPKE_643NM_E350S0450.IMG", "967eb2a5fcbc7c550b1e411f7cb843c1d8c29f53d1f069353ec18fadaa05ef60"),
            Source(f"{MOSAIC}/WAC_HAPKE_643NM_E350N1350.IMG", "1be241a613ed2141bd7a7b62d3b050a9e7d20b4e15aaae92fb665558bcf23601"),
            Source(f"{MOSAIC}/WAC_HAPKE_643NM_E350S1350.IMG", "dc5c6b86f3a4dc6c6e3eb6e4766371b75aa97482980766377d7a3abb6775ad4c"),
            Source(f"{MOSAIC}/WAC_HAPKE_643NM_E350N2250.IMG", "8aa0b7b69501e606e3fd4968d99787443fd629bf7a2682b0854a8caecb759a1e"),
            Source(f"{MOSAIC}/WAC_HAPKE_643NM_E350S2250.IMG", "27f487579d0543daa630961b1922125d03fed685871d5088c1d4b4a15c5b8578"),
            Source(f"{MOSAIC}/WAC_HAPKE_643NM_E350N3150.IMG", "a92b6253b062b4c7b2a8da6eb0c8bb844b6d97c1bc5a518177970046c8e8b20a"),
            Source(f"{MOSAIC}/WAC_HAPKE_643NM_E350S3150.IMG", "4aeea79efcb6402b012abf07e41cbba9ca8c2e9af83535de67d1b3345f22918c"),
        ],
        689: [
            Source(f"{MOSAIC}/WAC_HAPKE_689NM_E350N0450.IMG", "86d2105ab2794cee5363bc7e69e2551e234ba3b81698e8c7bff1ee2fafb3e101"),
            Source(f"{MOSAIC}/WAC_HAPKE_689NM_E350S0450.IMG", "e64b0de91da8767b664aad4590f54268c6b6224e688364c7704a831e4e7a2e7c"),
            Source(f"{MOSAIC}/WAC_HAPKE_689NM_E350N1350.IMG", "dd072d4cbe162da60e3efca25d5cb7d7cf33f9c0d490640f0f7ccf0390066dec"),
            Source(f"{MOSAIC}/WAC_HAPKE_689NM_E350S1350.IMG", "abe8f40226b8acf23fe1396c75e48844970ba0f565d8dcfb3d17b26409e8897f"),
            Source(f"{MOSAIC}/WAC_HAPKE_689NM_E350N2250.IMG", "6305ce22ad4f8eb9b257a520ff9660b8e69b7f98024f2eab263d29b8709af3a5"),
            Source(f"{MOSAIC}/WAC_HAPKE_689NM_E350S2250.IMG", "8fb0ad03241d492b8b636a65488212aca8b21987a95d894616bff0881f2dbe8b"),
            Source(f"{MOSAIC}/WAC_HAPKE_689NM_E350N3150.IMG", "b13ea18691cbdd2f3fa89ce7647eab7af1135ff97f63f92ed0274b615c961439"),
            Source(f"{MOSAIC}/WAC_HAPKE_689NM_E350S3150.IMG", "811645f19be5fbe31e0c2cb0b26a65a57d2440b8c28fb4a1744de65c12138cfa"),
        ],
    }
    parameter_map = Source(f"{PARAMETER_MAPS}/WAC_HAPKEPARAMMAP_7BAND.IMG", "555b7924266d6a9fb26df64b5d790d4f5724f15050c0d0d27d3aac81b2cd6633")
    polar_albedo = [
        (Source(f"{ALBEDO}/ldam_50n_1000m_float.img", "f795fd56e19718e31885fb79f7d9cd197302df91741a4f39e3aa13f053a73273"), Source(f"{ALBEDO}/ldam_50n_1000m_float.lbl", "d437c6484d00564a749d90cc48622dbff8da12c931ffbdb797799f44a971e5f6")),
        (Source(f"{ALBEDO}/ldam_50s_1000m_float.img", "cd35ca62d322d48b2b809041e9baa086242842c25a50f8bff70936929910079a"), Source(f"{ALBEDO}/ldam_50s_1000m_float.lbl", "1f7bb6617eb1c3e18a93a37eb7b1f2957c89d352c280b496b4bbbadf74edbf35")),
    ]
    elevation = Source(
        "https://svs.gsfc.nasa.gov/vis/a000000/a004700/a004720/ldem_64.tif",
        "af55ea2ed9173d1744b98ac4205501bd0bb0dcfc6ac49a11473a411dc64afd90",
    )
    # Coarsest first: each covers the texels it fully covers, so every texel ends from the finest product covering it.
    polar_elevation = [
        (Source(f"{POLAR_ELEVATION}/ldem_60n_240m.img", "6307de84ed2af969e7b7ad089fb4d87b403afb56d1c4ec3f4fdc3dc3434f4d4f"), Source(f"{POLAR_ELEVATION}/ldem_60n_240m.lbl", "002af6231644625fa5aa6c1fcfdaf2879a1f4eff154e37d0c7aa0606d53091ad")),
        (Source(f"{POLAR_ELEVATION}/ldem_60s_240m.img", "0f2766bd111bcf9ff05fc93515b79af49696d687a09a801622bae392e384df5a"), Source(f"{POLAR_ELEVATION}/ldem_60s_240m.lbl", "dde15a7f2a64ff8d744913c31d606ef572a992c74a44a72356ff91ca9cf4bfcb")),
        (Source(f"{POLAR_ELEVATION}/ldem_75n_120m.img", "75c4f50576e763761dbcc73638d1eb65a3c3c73a54127fc8b5e1672829cf7fb1"), Source(f"{POLAR_ELEVATION}/ldem_75n_120m.lbl", "58ff3ddda471f707bc7536f394554e39fa631d93643c5f230a39acf4ef1d0e20")),
        (Source(f"{POLAR_ELEVATION}/ldem_75s_120m.img", "ae3afc3c75c33d43666ca06c83ca08f0b12ef03b7d36d2d791d972730794391b"), Source(f"{POLAR_ELEVATION}/ldem_75s_120m.lbl", "5c59b16ec8a610792b1776fa082e409c8cc9f6743757710d14876ef366acd99a")),
    ]
    credits = [
        Credit(
            "Moon surface (system-solar/resources/textures/bodies/luna/, system-solar/data/terrains/luna.ktx2)",
            "Color: the Lunar Reconnaissance Orbiter Camera (LROC) Wide Angle Camera Hapke-normalized mosaic "
            "(LRO-L-LROC-5-RDR-V1.0, WAC_HAPKE; Sato et al. 2017, Icarus 296) and Hapke parameter maps "
            "(WAC_HAPKEPARAMMAP; Sato et al. 2014, Journal of Geophysical Research: Planets 119), NASA/GSFC/Arizona "
            "State University; polar latitudes from the Lunar Orbiter Laser Altimeter (LOLA) 1064 nm albedo maps "
            "(LRO-L-LOLA-4-GDR-V1.0; Lemelin et al. 2016, Icarus 273). Elevation: LOLA gridded data, through the "
            "CGI Moon Kit of NASA's Scientific Visualization Studio (https://svs.gsfc.nasa.gov/4720); beyond 60° of "
            "latitude, LOLA's polar stereographic elevation models (LRO-L-LOLA-4-GDR-V1.0, 240 m and 120 m per "
            "pixel), NASA/GSFC. Public domain "
            "NASA data; the color is the diffusive reflectance solved from the mosaic with the Hapke parameters, the "
            "normal map is derived from the elevation.",
        )
    ]

    def build(self, context: BuildContext) -> None:
        color = self._color(context)
        TextureWriter.color().write(context.texture_path("bodies/luna/color"), color.faces)

        heights = self._heights(context)
        context.terrains.write("luna", heights)
        TextureWriter.normal().write(context.texture_path("bodies/luna/normal"), heights.normals(RADIUS).faces)

        hapke = self._hapke(context)
        context.materials.write_textured("luna", color.area_mean(), "bodies/luna/color", "bodies/luna/normal", hapke)

    # Meters relative to the radius, each texel from the finest elevation model covering it.
    def _heights(self, context: BuildContext) -> CubeMap:
        terrain = CubeAccumulator(TERRAIN_FACE_SIZE, CubeProjection.EQUAL_ANGLE, 1, TERRAIN_CELLS_PER_TEXEL)
        elevation = self.elevation.fetch(context.cache)
        terrain.add(EquirectangularSource.global_map(elevation, CGI_MOON_KIT, METERS_PER_KILOMETER))
        heights = terrain.cube_map()

        for image, label in self.polar_elevation:
            image.fetch(context.cache)
            source = PolarStereographicSource(label.fetch(context.cache))
            polar = CubeAccumulator(TERRAIN_FACE_SIZE, CubeProjection.EQUAL_ANGLE, 1, TERRAIN_CELLS_PER_TEXEL)
            polar.add(source)
            covered = polar.covered(source)
            polar_heights = polar.cube_map()

            for face in range(CUBE_FACE_COUNT):
                heights.faces[face].pixels[covered[face]] = polar_heights.faces[face].pixels[covered[face]]

        return heights

    # Hapke 2012's parameters of one band, as they are, in the medium and opposition maps: computed on the parameter
    # maps' own pixels, then averaged into the cubes.
    def _hapke(self, context: BuildContext) -> HapkeMaterial:
        parameters = self._parameters(self.parameter_map.fetch(context.cache), HAPKE_WAVELENGTH).extended_toward_poles()
        medium = self._channels(parameters, ["b", "c", "theta"])
        porosity = Luna._porosity(parameters.pixels[..., PARAMETERS.index("phi")])
        layers = {
            "bodies/luna/scatter": Raster(np.concatenate([medium.pixels, porosity[..., None]], axis=-1)),
            "bodies/luna/surge": self._channels(parameters, ["Bs0", "hs", "Bc0", "hc"]),
        }

        for layer, raster in layers.items():
            cube = CubeAccumulator(
                HAPKE_FACE_SIZE, CubeProjection.EQUAL_ANGLE, raster.pixels.shape[-1], HAPKE_CELLS_PER_TEXEL
            )
            cube.add(EquirectangularSource.from_raster(raster))
            TextureWriter.values().write(context.texture_path(layer), cube.cube_map().faces)

        return HapkeMaterial(*layers)

    # The porosity coefficient K from the filling factor φ (Hapke 2012, eq. 7.45b); K tends to 1 as φ tends to 0.
    @staticmethod
    def _porosity(filling_factor: np.ndarray) -> np.ndarray:
        term = 1.209 * np.power(np.clip(filling_factor, 0, None), 2 / 3)

        with np.errstate(divide="ignore", invalid="ignore"):
            porosity = -np.log(1 - term) / term

        return np.where(term > 0, porosity, 1.0).astype(np.float32)

    # Each mosaic pixel's diffusive reflectance, per band, from the single-scattering albedo that reproduces its
    # measurement with the Hapke parameters at that pixel, averaged into the cube; then the bands as sRGB.
    def _color(self, context: BuildContext) -> CubeMap:
        parameter_path = self.parameter_map.fetch(context.cache)
        reflectances = []

        for wavelength, sources in self.mosaic.items():
            parameters = self._parameters(parameter_path, wavelength).extended_toward_poles()
            cube = CubeAccumulator(COLOR_FACE_SIZE, CubeProjection.EQUAL_ANGLE, 1, COLOR_CELLS_PER_TEXEL)

            for tile in sources:
                source = EquirectangularSource.georeferenced(tile.fetch(context.cache))
                subdivisions = cube.subdivisions(source)

                for band in source.bands():
                    model = self._model(parameters.interpolated_at(band.latitudes, band.longitudes), parameters.pixels)
                    albedo = model.single_scattering_albedo(band.values[..., 0], STANDARD_GEOMETRY)
                    reflectance = HapkeModel.diffusive_reflectance(albedo)
                    cube.add_band(source, band.with_values(reflectance[..., None]), subdivisions)

                logger.info("%d nm: %s", wavelength, tile.file_name)

            reflectances.append(np.stack([face.pixels[..., 0] for face in cube.cube_map().faces]))

        bands = BandColor(list(self.mosaic)).linear_srgb(np.stack(reflectances, axis=-1)).astype(np.float32)
        color = CubeMap([Raster(face) for face in bands], CubeProjection.EQUAL_ANGLE)
        filled = CoverageFill(*POLAR_OVERLAP).fill(color, self._polar_albedo(context))
        return CubeMap([ColorGrade(SATURATION).apply(face) for face in filled.faces], CubeProjection.EQUAL_ANGLE)

    @staticmethod
    def _channels(parameters: Raster, names: list[str]) -> Raster:
        return Raster(np.stack([parameters.pixels[..., PARAMETERS.index(name)] for name in names], axis=-1))

    def _parameters(self, path, wavelength: int) -> Raster:
        first = CAMERA_WAVELENGTHS.index(wavelength) * len(PARAMETERS) + 1
        return Raster.read_tiles([path], 360, bands=list(range(first, first + len(PARAMETERS))))

    # Sato et al. hold Bc0, hc and θ constant over the Moon, taken here from the whole parameter map; their filling
    # factor φ = 0 makes the porosity K exactly 1.
    @staticmethod
    def _model(parameters: np.ndarray, whole_map: np.ndarray) -> HapkeModel:
        value = {name: parameters[..., index] for index, name in enumerate(PARAMETERS)}
        constant = {name: whole_map[..., index] for index, name in enumerate(PARAMETERS)}
        return HapkeModel(
            lobe_width=value["b"],
            lobe_balance=value["c"],
            shadow_hiding_amplitude=value["Bs0"],
            shadow_hiding_width=value["hs"],
            backscatter_amplitude=float(np.nanmean(constant["Bc0"])),
            backscatter_width=float(np.nanmean(constant["hc"])),
            porosity=1.0,
            roughness=float(np.radians(np.nanmean(constant["theta"]))),
        )

    def _polar_albedo(self, context: BuildContext) -> CubeMap:
        cube = CubeAccumulator(COLOR_FACE_SIZE, CubeProjection.EQUAL_ANGLE, 1, COLOR_CELLS_PER_TEXEL)

        for image, label in self.polar_albedo:
            image.fetch(context.cache)
            cube.add(PolarStereographicSource(label.fetch(context.cache)))

        return cube.cube_map()
