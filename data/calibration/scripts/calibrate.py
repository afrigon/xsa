# /// script
# requires-python = ">=3.13"
# dependencies = ["colour-science", "numpy", "pillow"]
# ///
# Checks the renderer against known colors: one sphere per patch of a 24-patch color checker, lit by a Sun-like star
# and seen head-on from the star's side, so each sphere's center shows its reflectance × sunlight ÷ π, exposed. The
# camera's white balance takes the star's light as white, so each patch shows its own reflectance's color.
#
#   uv run calibrate.py generate            write the pack's bodies, system, simulation and materials
#   uv run calibrate.py measure <instance>  snap every patch in a running game and compare with the prediction

import argparse
import subprocess
import warnings
from pathlib import Path

import numpy as np
from PIL import Image

# colour-science warns at import about each optional dependency it lacks; only its data and conversions are used.
with warnings.catch_warnings(action="ignore"):
    import colour

PACK = Path(__file__).resolve().parent.parent
NAMESPACE = PACK / "calibration"
CHART = "BabelColor Average"
OBSERVER = "CIE 1931 2 Degree Standard Observer"
SRGB = colour.RGB_COLOURSPACES["sRGB"]
STAR = "calibration-sun"
ASTRONOMICAL_UNIT = 1.495978707e11
PATCH_RADIUS = 1000.0
PATCH_SPACING = 10_000.0
CAMERA_DISTANCE = "3km"
# The orbit camera looks along the yaw-rotated +Y axis; patches sit on +X, so -90° puts the camera between them and
# the star.
CAMERA_YAW = "-90"
CENTER_HALF_SIZE = 8
GRAY_CHROMA = 3.0

# The renderer's exposure (crates/client/src/renderer/auto_exposure.rs), from its published standards: an incident
# meter (ISO 2720, C = 250) and standard output sensitivity (ISO 12232). A Lambertian albedo a lit by E lux shows
# a × E / π cd/m², exposed by MIDDLE_GRAY / (10 / (ISO × q) × E × ISO / C), so E cancels.
ISO_SENSITIVITY = 100.0
INCIDENT_METER_CALIBRATION = 250.0
STANDARD_OUTPUT_EXPOSURE = 10.0
LENS_TRANSMITTANCE = 0.65
MIDDLE_GRAY = 0.18
DISPLAY_PER_ALBEDO = (
    MIDDLE_GRAY
    / np.pi
    / (STANDARD_OUTPUT_EXPOSURE / (ISO_SENSITIVITY * LENS_TRANSMITTANCE) * ISO_SENSITIVITY / INCIDENT_METER_CALIBRATION)
)


def patch_id(name: str) -> str:
    return name.split(" (")[0].replace(".", "-").replace(" ", "-")


# Each patch's reflectance as linear sRGB, under the D65 white sRGB is defined against: the color matching functions
# and the illuminant sampled at the chart's measured wavelengths, normalized so a perfect reflector has Y = 1.
def patches() -> dict[str, np.ndarray]:
    chart = colour.SDS_COLOURCHECKERS[CHART]
    wavelengths = next(iter(chart.values())).wavelengths
    weights = colour.MSDS_CMFS[OBSERVER][wavelengths] * colour.SDS_ILLUMINANTS["D65"][wavelengths][:, None]
    weights /= weights[:, 1].sum()
    return {patch_id(name): SRGB.matrix_XYZ_to_RGB @ (sd.values @ weights) for name, sd in chart.items()}


def write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)


def generate() -> None:
    colors = patches()
    write(PACK / "pack.kdl", 'version "0.1.0"\nformat 1\ndependencies {\n    base "^0.1.0"\n}\n')
    write(
        NAMESPACE / "data/bodies" / f"{STAR}.kdl",
        'category "star"\nradius 695700000\ngravitational-parameter 1.32712440018e20\nrotation-period 2192832\n'
        "axial-tilt 0\nluminosity 3.828e26\neffective-temperature 5772\n",
    )
    write(NAMESPACE / "resources/materials" / f"{STAR}.kdl", 'shader "base:emissive"\n')
    write(NAMESPACE / "resources/objects/bodies" / f"{STAR}.kdl", f'material "{STAR}"\n')
    orbits = []

    for index, (patch, color) in enumerate(colors.items()):
        if (color < 0).any():
            print(f"{patch}: outside sRGB {color.round(4)}, clipped")

        write(
            NAMESPACE / "data/bodies" / f"{patch}.kdl",
            f'category "asteroid"\nradius {PATCH_RADIUS:g}\ngravitational-parameter 1\nrotation-period 86400\n'
            "axial-tilt 0\n",
        )
        rgb = " ".join(f"{component:.5f}" for component in np.clip(color, 0, None))
        write(NAMESPACE / "resources/materials" / f"{patch}.kdl", f'shader "base:planet"\nbase-color {rgb}\n')
        write(NAMESPACE / "resources/objects/bodies" / f"{patch}.kdl", f'material "{patch}"\n')
        # A zero mean motion keeps every patch on +X whatever the date, side by side along the orbit.
        anomaly = np.degrees(index * PATCH_SPACING / ASTRONOMICAL_UNIT)
        orbits.append(
            f'    body "{patch}" {{\n'
            f"        orbit semi-major-axis={ASTRONOMICAL_UNIT:.10g} eccentricity=0 inclination=0 ascending-node=0 "
            f"periapsis=0 mean-anomaly={anomaly:.9f} mean-motion=0\n"
            "    }\n"
        )

    write(
        NAMESPACE / "data/systems/calibration.kdl",
        f'epoch "2000-01-01T12:00:00Z"\nstar "{STAR}" {{\n' + "".join(orbits) + "}\n",
    )
    write(NAMESPACE / "data/simulations/calibration.kdl", f'system "calibration"\nspawn "{list(colors)[0]}"\n')


def ipc(instance: str, *words: str) -> str:
    command = ["mise", "run", "ipc", "--", "--instance", instance, *words]
    return subprocess.run(command, capture_output=True, text=True, check=True).stdout


def center(path: Path) -> np.ndarray:
    pixels = np.asarray(Image.open(path).convert("RGB"), np.float64) / 255
    height, width = pixels.shape[:2]
    rows = slice(height // 2 - CENTER_HALF_SIZE, height // 2 + CENTER_HALF_SIZE)
    columns = slice(width // 2 - CENTER_HALF_SIZE, width // 2 + CENTER_HALF_SIZE)
    return colour.models.eotf_sRGB(pixels[rows, columns]).reshape(-1, 3).mean(axis=0)


def lab(linear: np.ndarray) -> np.ndarray:
    return colour.XYZ_to_Lab(SRGB.matrix_RGB_to_XYZ @ linear, SRGB.whitepoint)


def measure(instance: str, output: Path, tonemapper: str) -> None:
    output.mkdir(parents=True, exist_ok=True)

    for setting in ["animations.duration-scale 0", "render.bloom.enabled false", "render.lens.enabled false"]:
        ipc(instance, "config", "set", *setting.split())

    ipc(instance, "ui", "set", "game")
    colors = patches()
    measured = {}

    for setting in ["none", tonemapper]:
        ipc(instance, "config", "set", "render.tonemapper", setting)

        for patch in colors:
            ipc(instance, "camera", "target", patch, "--distance", CAMERA_DISTANCE, "--yaw", CAMERA_YAW, "--pitch", "0")
            path = output / f"{setting}-{patch}.png"
            ipc(instance, "camera", "snap", "--output", str(path))
            measured[setting, patch] = center(path)

    print(f"display value per unit albedo {DISPLAY_PER_ALBEDO:.4f}\n")
    print(f"{'patch':14s} {'expected (linear)':>24s} {'measured, no tone mapping':>27s} {'ΔE':>5s}   {tonemapper}: ΔL*  chroma  hue°")

    for patch, color in colors.items():
        expected = np.clip(color, 0, None) * DISPLAY_PER_ALBEDO
        plain, mapped = measured["none", patch], measured[tonemapper, patch]
        difference = colour.delta_E(lab(expected), lab(plain), method="CIE 2000")
        lab_plain, lab_mapped = colour.Lab_to_LCHab(lab(plain)), colour.Lab_to_LCHab(lab(mapped))
        # A gray has no hue, and its chroma is too small for a ratio to mean anything.
        if lab_plain[1] > GRAY_CHROMA:
            chroma = f"{lab_mapped[1] / lab_plain[1]:5.2f}"
            hue = f"{(lab_mapped[2] - lab_plain[2] + 180) % 360 - 180:+5.1f}"
        else:
            chroma, hue = f"{'—':>5s}", f"{'—':>5s}"

        print(
            f"{patch:14s} {np.array2string(expected, precision=4):>24s} {np.array2string(plain, precision=4):>27s} "
            f"{difference:5.2f}   {lab_mapped[0] - lab_plain[0]:+6.1f}  {chroma}  {hue}"
        )


def main() -> None:
    parser = argparse.ArgumentParser()
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("generate")
    measuring = commands.add_parser("measure")
    measuring.add_argument("instance")
    measuring.add_argument("--output", type=Path, default=PACK / "scripts" / "snaps")
    measuring.add_argument("--tonemapper", default="agx", help="compared with no tone mapping: agx, agx-punchy, …")
    arguments = parser.parse_args()

    if arguments.command == "generate":
        generate()
    else:
        measure(arguments.instance, arguments.output, arguments.tonemapper)


if __name__ == "__main__":
    main()
