# Third-party assets

Assets are not stored in this repository. `mise run fetch-assets` downloads
them into `assets/`, next to their license or credit file.

## Deep Star Maps 2020

- **Source:** [NASA SVS 4851](https://svs.gsfc.nasa.gov/4851) —
  `starmap_2020_16k.exr`
- **Credit:** NASA/Goddard Space Flight Center Scientific Visualization
  Studio. Gaia DR2: ESA/Gaia/DPAC. Hipparcos-2, Tycho-2: ESA.
- **License:** NASA media usage guidelines — free to use, credit requested
- **Local copy:** source in `assets/deep-star-maps/`, credit in
  `assets/deep-star-maps/CREDIT.txt`; the converted cube map
  `data/system-solar/system-solar/resources/textures/skyboxes/deep-star-maps.dds`
  is a derived work under the same terms

## Solar system data

- **Source:** orbits of every body except Earth, the Moon and Dactyl are fitted to
  positions from [JPL Horizons](https://ssd.jpl.nasa.gov/horizons/), with
  Laplace planes from the [JPL planetary satellite mean
  elements](https://ssd.jpl.nasa.gov/sats/elem/); physical properties come
  from Horizons; rotation from the IAU WGCCRE report (Archinal et al. 2018,
  Celestial Mechanics and Dynamical Astronomy 130:22); values neither
  provides (some rotation periods and gravitational parameters, Dactyl's
  orbit) come from Sol's configs
- **Credit:** NASA/JPL-Caltech Solar System Dynamics Group
- **License:** US government work, public domain; values are facts and
  carry no license
- **Local copy:** the values are written into
  `data/system-solar/system-solar/data/`

## Sol 0.9.5 (body textures)

- **Source:** [RSS-Reborn](https://github.com/RSS-Reborn), the `Sol-*`
  repositories' releases
- **Credit:** per-file authors and sources in Sol's
  [credits file](https://github.com/RSS-Reborn/Sol-Configs/blob/f9e6fdf4e26c4a5ba1364ba93babfa2ada3e5a5c/Sol-Configs/Credits-License.md)
- **License:** CC BY-NC-SA 4.0 — attribution, no commercial use, derived
  textures under the same license
- **Local copy:** archives in `assets/sol/archives/`, extracted to
  `assets/sol/extracted/`; the license text and attribution ship with the pack
  in `data/system-solar/LICENSE`

## AdvancedPQSTools (Hapke shader code)

- **Source:** [CharonSSS/AdvancedPQSTools](https://github.com/CharonSSS/AdvancedPQSTools),
  `Shaders/Hapke/HapkeScaledFunctions.cginc` (Hapke shaders by ballisticfox)
- **License:** MIT, Copyright (c) 2022 Niako
- **Local copy:** ported into `data/base/base/resources/shaders/common/hapke.slang`;
  the license text ships with the base pack in `data/base/LICENSE`

## Inter 4.1

- **Source:** [rsms/inter](https://github.com/rsms/inter/releases/tag/v4.1) —
  `Inter-4.1.zip`, `InterVariable.ttf`
- **License:** SIL Open Font License 1.1, Copyright (c) 2016 The Inter Project
  Authors; no Reserved Font Name
- **Local copy:** placed by hand at
  `data/base/base/resources/fonts/inter/inter-variable.ttf`; the license text
  ships with the base pack in `data/base/LICENSE`

## JetBrains Mono 2.304

- **Source:** [JetBrains/JetBrainsMono](https://github.com/JetBrains/JetBrainsMono/releases/tag/v2.304) —
  `JetBrainsMono-2.304.zip`, `fonts/variable/JetBrainsMono[wght].ttf`
- **License:** SIL Open Font License 1.1, Copyright 2020 The JetBrains Mono
  Project Authors; no Reserved Font Name
- **Local copy:** placed by hand at
  `data/base/base/resources/fonts/jetbrains-mono/jetbrains-mono-variable.ttf`;
  the license text ships with the base pack in `data/base/LICENSE`
