# Bakes the lens into the base pack: traces the ghosts, computes the starburst, draws the ring and exports the textures
# and the main lens profile. `python3 bake.py --preview` runs everything at preview quality in a few minutes.
import os
import subprocess
import sys
import time

SCRIPTS = os.path.dirname(os.path.abspath(__file__))
flags = [flag for flag in sys.argv[1:] if flag.startswith("--")]

for step in ["trace_ghosts.py", "starburst.py", "ring.py", "export.py"]:
    start = time.time()
    subprocess.run([sys.executable, "-W", "ignore", os.path.join(SCRIPTS, step), *flags], check=True, cwd=SCRIPTS)
    print(f"{step}: {time.time() - start:.0f}s", flush=True)
