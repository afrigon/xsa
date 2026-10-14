from operations.credit import Credit

PACK_CREDITS = [
    Credit(
        "Data (system-solar/data/)",
        "Orbits, physical properties and rotation of the bodies are facts, taken from:\n\n"
        "- NASA/JPL Horizons (https://ssd.jpl.nasa.gov/horizons/) and the JPL planetary satellite mean elements "
        "(https://ssd.jpl.nasa.gov/sats/elem/), NASA/JPL-Caltech Solar System Dynamics Group. US government work, "
        "public domain. Orbits are fitted to Horizons positions.\n"
        "- The IAU WGCCRE report on cartographic coordinates and rotational elements (Archinal et al. 2018, "
        "Celestial Mechanics and Dynamical Astronomy 130:22).\n"
        "- Sol by RSS-Reborn (https://github.com/RSS-Reborn), for a few values neither source provides.",
    ),
]
