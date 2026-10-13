import argparse
import importlib
import logging
from pathlib import Path

from operations.build_context import BuildContext
from operations.license_writer import LicenseWriter
from operations.recipe import Recipe
from pack_credits import PACK_CREDITS

SCRIPTS = Path(__file__).resolve().parent
PACK = SCRIPTS.parent
RECIPE_ROOTS = [SCRIPTS / "skybox.py", SCRIPTS / "solar_system"]
TIER_WIDTH = 4096


def recipes() -> dict[str, Recipe]:
    for root in RECIPE_ROOTS:
        paths = [root] if root.is_file() else sorted(root.rglob("*.py"))

        for path in paths:
            importlib.import_module(".".join(path.relative_to(SCRIPTS).with_suffix("").parts))

    found = [recipe() for recipe in Recipe.__subclasses__()]
    return {recipe.name: recipe for recipe in found}


def main() -> None:
    known = recipes()
    parser = argparse.ArgumentParser(description="Generate system-solar's assets from their sources.")
    parser.add_argument("recipes", nargs="*", help=f"what to build, everything by default: {', '.join(sorted(known))}")
    parser.add_argument("--verbose", "-v", action="store_true")
    arguments = parser.parse_args()
    unknown = sorted(set(arguments.recipes) - set(known))

    if unknown:
        parser.error(f"unknown recipes: {', '.join(unknown)}")

    logging.basicConfig(level=logging.INFO, format="%(levelname)s %(message)s")

    if arguments.verbose:
        logging.getLogger("operations").setLevel(logging.DEBUG)

    context = BuildContext(PACK, TIER_WIDTH)

    for name in arguments.recipes or sorted(known):
        print(f"building {name}")
        known[name].build(context)

    LicenseWriter(PACK / "LICENSE").write(PACK_CREDITS + [credit for recipe in known.values() for credit in recipe.credits])


if __name__ == "__main__":
    main()
