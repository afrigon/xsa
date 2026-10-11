# Bake quality: `--preview` trades accuracy for speed when tweaking settings. Intermediate files then carry a suffix,
# so a preview never overwrites a final bake.
import os
import sys
import numpy as np

PREVIEW = "--preview" in sys.argv
SUFFIX = "-preview" if PREVIEW else ""
GHOST_RAYS_ACROSS = 300 if PREVIEW else 1200
GHOST_WAVELENGTHS = np.linspace(410, 690, 6 if PREVIEW else 16)
STARBURST_PATCH = 1024 if PREVIEW else 4096
STARBURST_FAR_PATCH = 512 if PREVIEW else 2048
STARBURST_WAVELENGTHS = np.linspace(400e-9, 700e-9, 6 if PREVIEW else 32)

SCRIPTS = os.path.dirname(os.path.abspath(__file__))
REPOSITORY = os.path.normpath(os.path.join(SCRIPTS, "../../../.."))
# Intermediate files are large and can always be rebaked, so they live in the git-ignored build directory.
WORK = os.path.join(REPOSITORY, "target", "lens-bake")
os.makedirs(WORK, exist_ok=True)

def ghosts_file():
    return os.path.join(WORK, f"ghosts{SUFFIX}.npz")

def starburst_file():
    return os.path.join(WORK, f"starburst{SUFFIX}.npy")

def starburst_far_file():
    return os.path.join(WORK, f"starburst-far{SUFFIX}.npy")

def ring_file():
    return os.path.join(WORK, "ring.npy")
