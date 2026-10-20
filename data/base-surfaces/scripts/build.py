import argparse
import importlib
import logging
from pathlib import Path

from operations.build_context import BuildContext
from operations.license_writer import LicenseWriter
from operations.surface_recipe import SurfaceRecipe

SCRIPTS = Path(__file__).resolve().parent
PACK = SCRIPTS.parent
RECIPE_ROOT = SCRIPTS / "surfaces"


def recipes() -> dict[str, SurfaceRecipe]:
    for path in sorted(RECIPE_ROOT.rglob("*.py")):
        importlib.import_module(".".join(path.relative_to(SCRIPTS).with_suffix("").parts))

    found = [recipe() for recipe in SurfaceRecipe.__subclasses__()]
    return {recipe.name: recipe for recipe in found}


def main() -> None:
    known = recipes()
    parser = argparse.ArgumentParser(description="Generate base-surfaces' assets from their sources.")
    parser.add_argument("recipes", nargs="*", help=f"what to build, everything by default: {', '.join(sorted(known))}")
    parser.add_argument("--density", type=int, required=True, help="texels per meter of ground, e.g. 512 or 1024")
    parser.add_argument("--verbose", "-v", action="store_true")
    arguments = parser.parse_args()
    unknown = sorted(set(arguments.recipes) - set(known))

    if unknown:
        parser.error(f"unknown recipes: {', '.join(unknown)}")

    logging.basicConfig(level=logging.INFO, format="%(levelname)s %(message)s")

    if arguments.verbose:
        logging.getLogger("operations").setLevel(logging.DEBUG)

    context = BuildContext(PACK, arguments.density)

    for name in arguments.recipes or sorted(known):
        print(f"building {name}")
        known[name].build(context)

    LicenseWriter(PACK / "LICENSE").write([credit for recipe in known.values() for credit in recipe.credits(context.cache)])


if __name__ == "__main__":
    main()
