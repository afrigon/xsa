# Packs

Everything that defines what exists in the game and how it looks — bodies,
star systems, simulations, parts, materials, textures, sounds, text — comes
from packs: directories of KDL files and assets, modeled on Minecraft's data
and resource packs. `data/base` holds only what the game itself needs
(built-in shaders, later default textures and UI) and defines no star system.
Star systems ship as their own packs — the real solar system is
`data/system-solar` — and custom systems, reskins and mods are packs layered
on top. With no system installed, the game refuses to start a simulation.

Packs are parsed and resolved once, at load time, into typed data with
indices; gameplay never reads KDL or looks anything up by name.

## Pack structure

A pack contains **namespace folders**, each holding its own `data/` and
`resources/`. A pack usually uses its own id as its only namespace, and adds a
folder for another namespace only to override that pack's content.

```
data/<pack>/
  pack.kdl                                  manifest
  LICENSE                                   terms of third-party content, if any
  scripts/                                  tools that produce the pack's assets; not a namespace
  cache/                                    the scripts' source downloads; git-ignored, never released
  <namespace>/
    data/                                   gameplay and physics — read by the server
      bodies/                               intrinsic properties of each body
      systems/                              how bodies relate: hierarchy and orbits
      simulations/                          which systems make up a playable simulation
      parts/                                rocket parts (reserved)
    resources/                              presentation — read by the client
      objects/                              mirrors data/ one-to-one
        bodies/  systems/  simulations/  parts/
      materials/                            shader and its parameters
      shaders/                              SPIR-V shaders (base only, see below)
      textures/  models/  sounds/  fonts/   shared assets, organized freely
      theme/                                design system: colors, roles, typefaces, text styles
      lang/                                 localization files (reserved)
```

The server only reads `data/`; the client reads both.

A pack that ships third-party content carries a `LICENSE` file at its root
stating which content is under which terms, with the full text of each license
that requires it and the attribution it asks for.

## Manifest

```kdl
version "0.1.0"
format 1
dependencies {
    base "^0.1.0"
}
```

- **`version`:** the pack's semantic version.
- **`format`:** the pack format version; the loader rejects formats it does
  not support.
- **`dependencies`:** one child per required pack, named by pack id, with a
  Cargo-style version requirement. A dependency must appear earlier in the
  pack list.

## Ids and namespaces

An id is `namespace:path`, where the path is the file's location inside the
folder for its kind, without the extension:
`data/system-solar/system-solar/resources/textures/skyboxes/deep-star-maps.ktx2`
is the texture `system-solar:skyboxes/deep-star-maps`. The kind comes from
where a reference is used — a `material` reference looks in `materials/`, a
skybox `texture` in `textures/` — so ids never repeat it. Inside a file, a
reference without a namespace means the file's own namespace. Body ids use
kebab-case scientific names (`sol`, `earth`, `luna`).

## Stack and overrides

The effective game content is built from an ordered list of packs. Every file
fills the slot `namespace/section/kind/path`; a later pack's file in the same
slot replaces the earlier one, one whole file at a time. Field-level merging is
deliberately not supported. Overriding another pack means adding a folder for
its namespace — `system-solar/resources/textures/…` in a reskin pack — which
makes the intent explicit.

## Data

### Bodies — `<namespace>/data/bodies/`

Properties intrinsic to a body, valid in any system. SI units; angles in
degrees.

```kdl
category "planet"
radius 6371000
gravitational-parameter 3.986004418e14
rotation-period 86164.0905
axial-tilt 23.4392811
```

- **`category`**: what the body is in its system: `star`, `brown-dwarf`,
  `white-dwarf`, `neutron-star`, `black-hole`, `planet`, `dwarf-planet`,
  `moon`, `asteroid` or `comet`. Set by hand, never derived from the body's
  parameters: light comes from `luminosity`, whatever the category.
- **`radius`** (m): mean radius.
- **`gravitational-parameter`** (m³/s²): μ = G·M, which is measured far more
  precisely than the mass itself.
- **`rotation-period`** (s): sidereal rotation period; negative when the body
  spins clockwise seen from above its north pole (Venus, Uranus), so the
  north pole stays the IAU one that textures are mapped to.
- **`axial-tilt`** (°): tilt of the spin axis relative to the body's own
  orbital plane.
- **`luminosity`** (W) and **`effective-temperature`** (K), together and only
  on stars: the star's total radiated power and the temperature of the
  blackbody that matches it. Lighting derives from them: illuminance at any
  distance, the star's surface brightness and its color. The first star in a
  simulation is its light source.

### Systems — `<namespace>/data/systems/`

How bodies relate. Nesting mirrors the orbit hierarchy: a node's children
orbit it.

```kdl
epoch "2000-01-01T12:00:00Z"
star "sol" spin-azimuth=75.76 {
    barycenter "earth-moon" {
        orbit semi-major-axis=1.49598261e11 eccentricity=0.01671123 inclination=-0.00001531 ascending-node=0 periapsis=102.93768193 mean-anomaly=-2.47311027 {
            rates semi-major-axis=840.7 eccentricity=-0.00004392 inclination=-0.01294668 periapsis=0.32327364
        }
        primary "earth" spin-azimuth=180 prime-meridian=100.46061837
        secondary "luna" tidally-locked=#true {
            orbit semi-major-axis=384399000 eccentricity=0.0549 inclination=5.145 ascending-node=125.0445479 periapsis=318.3086986 mean-anomaly=134.9633964 mean-motion=477198.8675055 {
                rates ascending-node=-1934.1362891 periapsis=6003.1500178
            }
        }
    }
}
```

**Top level**

- **`epoch`** (UTC timestamp): the instant at which every orbital element in
  the file is valid. Positions at any other time are propagated from it.
- **Root**, exactly one of `star "<body>"` (a body at the system origin) or
  `barycenter "<name>"` (an invisible center of mass at the origin, for binary
  stars).

**Nodes**

- **`body "<body id>"`:** a body from `data/bodies/`; needs an `orbit` unless
  it is the root.
- **`barycenter "<name>"`:** a pair orbiting their common center of mass. Its
  `orbit` is the barycenter's own orbit around its parent (omitted at the
  root); it contains a `primary "<body id>"` with no orbit and a
  `secondary "<body id>"` whose `orbit` is **relative to the primary**. The
  loader splits that relative orbit by mass ratio into exact orbits around the
  barycenter. Larger groupings nest barycenters. A barycenter may also contain
  `body` and `barycenter` children, which orbit the pair's center of mass with
  the pair's combined μ (Pluto's small moons around Pluto–Charon).

**`orbit`** — classical Keplerian elements relative to the parent, in the
ecliptic J2000 frame; angles in degrees.

- **`semi-major-axis`** (m): half the ellipse's longest diameter.
- **`eccentricity`:** shape, from 0 (circle) to below 1 (closed orbits only).
- **`inclination`:** tilt of the orbital plane relative to the ecliptic.
- **`ascending-node`:** where the orbit crosses the ecliptic going north,
  measured from the vernal equinox (+X).
- **`periapsis`:** argument of periapsis — angle in the orbital plane from the
  ascending node to the closest point.
- **`mean-anomaly`:** position along the orbit at the epoch, as an angle
  growing uniformly with time.
- **`mean-motion`** (optional, °/Julian century): how fast the mean anomaly
  grows. Derived from the semi-major axis and the combined μ of the body and
  its parent when absent; given explicitly where perturbations make the
  two-body value wrong (the Moon).
- **`rates`** (optional child): per-element change per Julian century
  (36,525 days) — `semi-major-axis` (m), `eccentricity`, `inclination`,
  `ascending-node`, `periapsis` (°). Absent rates are zero. Real bodies need
  them to stay accurate decades away from the epoch; fictional systems can
  omit them.
- **`plane`** (optional child): the reference plane the elements are measured
  against, given by its pole's `right-ascension` and `declination` (°, in the
  celestial equatorial frame). The ascending node is then measured from where
  that plane crosses the celestial equator going north. Moons close to an
  oblate planet precess around a plane between the planet's equator and its
  orbit (the Laplace plane); measured against it, that precession is a
  constant `ascending-node` rate. Absent, the elements are relative to the
  ecliptic.

**Spin properties** on `star`, `body`, `primary` and `secondary` nodes, all
optional:

- **`spin-azimuth`** (°): the direction, around the reference plane's normal,
  toward which the spin axis tilts. The reference plane is the body's orbit
  (for a primary, its barycenter's orbit; for the root, the ecliptic).
- **`prime-meridian`** (°): the body's rotation angle at the epoch.
- **`tidally-locked=#true`:** the rotation period equals the orbital period,
  phased so the body faces its parent on average. Rotation stays uniform, so
  libration appears naturally on eccentric or inclined orbits.

### Simulations — `<namespace>/data/simulations/`

What the player selects in game.

```kdl
system "sol"
spawn "earth"
```

- **`system`:** the system to run.
- **`spawn`** (optional): the body players start at, a body id from the
  system.

A new save starts at the current date; the date is a property of the save,
not of the simulation.

**Anything the physics needs lives in `data/`**, because the server never
reads resources. Terrain shape decides where a lander touches down, and
atmospheric density drives drag and heating, so both are data even when part
of them is binary.

### Terrains — `<namespace>/data/terrains/`

A body's shape: a height map and the range its heights span. A body names its
terrain with `terrain "<id>"`.

```kdl
height-map "luna"
minimum -9041
maximum 10752
```

- **`height-map`:** the height map, relative to `data/terrains/` like every
  reference is relative to its kind's folder: `luna` is
  `data/terrains/luna.ktx2`.
- **`minimum`, `maximum`** (m): the lowest and highest heights in the map,
  relative to the body's `radius`, signed.

The height map is a KTX2 cube map, `R32_SFLOAT` and uncompressed: six square
faces in Vulkan's face order and orientation, +X, −X, +Y, −Y, +Z, −Z of the
body's frame (+X at longitude 0, +Z the north pole). It is sampled by
direction, so neighbouring faces blend across their edges and no face
stretches toward the poles. Each texel is a height in meters relative to the
body's `radius`. It carries its full mip chain,
each level the average of the one above, so the game only reads and uploads
it. The server and every client read the same file, so the terrain has one
resolution whatever texture tier a player installs.

## Resources

### Objects — `<namespace>/resources/objects/`

Mirror `data/`: `<namespace>/data/bodies/earth.kdl` is presented by
`<namespace>/resources/objects/bodies/earth.kdl`.

```kdl
material "earth"
```

A simulation's object file holds its skybox:

```kdl
skybox texture="skyboxes/deep-star-maps" frame="equatorial" luminance=0.05
```

- **`texture`:** a cube-map texture.
- **`luminance`** (cd/m²): the sky's luminance for a texture value of 1.
- **`frame`:** `equatorial` for celestial-coordinate star maps, `ecliptic` for
  maps already aligned with the world frame.

### Materials — `<namespace>/resources/materials/`

```kdl
shader "base:lit"
base-color 0.1 0.25 0.6
```

Light is physical: surfaces reflect the star's illuminance (lux) as luminance
(cd/m²), and emissive values are luminances in cd/m².

- **`shader "base:lit"`** with **`base-color`**: the albedo (linear red green
  blue).
- **`shader "base:emissive"`** with optional **`color`** (a tint, white by
  default) and **`luminance`** (cd/m²). A star takes its luminance and color
  from its data and needs neither.
- **`shader "base:planet"`** with **`base-color`** (the body's average albedo,
  linear red green blue) and optional **`color-texture`** (sRGB albedo),
  **`normal-texture`** and **`emissive-texture`** (sRGB), each a texture id;
  an emissive texture needs **`emissive-luminance`** (cd/m² for a texture
  value of 1). Without a color texture the surface takes the base color. A
  texture no loaded pack provides is skipped with a warning, as is a `hapke`
  node missing either of its textures.
  An optional **`hapke`** node shades the surface with Hapke's model of
  particulate surfaces (Hapke 2012, eq. 12.55) instead of Lambert. The color
  map's linear values are the diffusive reflectance r0, from which the shader
  derives the single-scattering albedo w; the node names the textures holding
  the other standard parameters, as published: `medium-texture` (the
  particulate medium: phase-function lobe width b, lobe balance c, macroscopic
  roughness θ̄ in degrees, porosity coefficient K) and `opposition-texture`
  (the opposition effects: shadow-hiding amplitude B_S0 and width h_S,
  coherent-backscatter amplitude B_C0 and width h_C), both half-float,
  channels in that order. A body with one published value per parameter ships
  them as 1×1 textures.
  Maps are equirectangular with the first row at the south pole, longitude 0
  at u = 0.25 and u growing westward (the convention of KSP's Kopernicus and
  Parallax, which Sol's textures follow). Normal maps are tangent-space with
  red toward west and green toward north; z is reconstructed.

### Lenses — `<namespace>/resources/lenses/`

A lens profile describes what a camera's lens adds to the image around every
light source: a glow, a starburst and ghosts. The game uses `base:main`, and the
client config can turn the effects off (`render.lens.enabled`).

```kdl
off-axis-falloff exponent=2

glare intensity=0.9329 falloff=2.2

starburst texture="lens/starburst" inner=0.01 extent=90

ghost position=1.42 size=1.43 dispersion-per-degree=0.0167 intensity=0.0446 {
    tint 1 1 1.3
    texture "lens/ghost-0-12"
}
```

Every element is physical: its brightness is the light's illuminance at the
camera (lux) times a value in luminance per unit illuminance (1/sr), so a
light's lens effects scale with its brightness and the exposure, and a faint
light's fade out on their own. Angles are degrees.

- **`off-axis-falloff`:** the lens passes light arriving at an angle θ off its
  axis in proportion to cos^`exponent` θ: 4 for an ideal simple lens, about 2
  for a wide-angle design. Every element fades by this factor; the rest of the
  image is not darkened.
- **`glare`:** light scattered by the lens's surfaces, `intensity` /
  θ^`falloff` at an angle θ from the light, flat across the light's own disk.
- **`starburst`:** diffraction by the aperture, a texture centered on the light
  and fixed to the camera. It is stored log-polar: columns are directions
  around the light (u from 0 to 1 over a full turn, starting to the right of
  the light and turning clockwise on screen), rows are angles from the light on
  a log scale from `inner` to `extent` (v = log(θ / inner) / log(extent /
  inner)); closer than `inner`, the first row holds. Values are linear
  luminance per unit illuminance. The texture holds only the light the
  aperture scatters, not the light's own image, which the scene draws, so it
  also covers the light's disk: a body hiding part of the light leaves the
  scattered light over the body.
- **`ghost`** (any number): reflections between the lens's surfaces, each a
  texture centered on the line from the light through the screen center.
  - `position`: 0 on the light, 1 at the screen center, beyond 1 past it.
  - `size`: the texture's angular width.
  - The texture's u axis points from the screen center toward the light and v
    across it, so a ghost turns with the light around the screen. Its values
    are linear, and its border must be black: nothing past it is drawn.
  - `intensity` (optional, 1 by default): multiplies the texture, giving
    luminance per unit illuminance. A texture can then be stored with its
    brightest texel at 1, readable in an image viewer, with its physical level
    here.
  - `dispersion-per-degree` (optional, 0 by default): a stylized chromatic
    split. Red moves outward along the axis and blue inward, each by this
    fraction of the ghost's size per degree the light sits off the view axis,
    so a light at the screen center shows none.
  - `tint` (optional, 1 1 1 by default): a linear multiplier per color
    channel.

A light whose glow, starburst and ghosts cannot change a pixel of the output
is skipped.

The base pack's lens is baked from a 22 mm wide-angle lens design by the
scripts in `data/base/scripts/lens/` (Python 3 and NumPy): they trace the
ghosts through the design, compute the aperture's diffraction, draw the ring
ghost, and write the textures and `main.kdl`. Every value in the profile comes
from them.

```sh
python3 data/base/scripts/lens/bake.py            # full quality, about 45 minutes
python3 data/base/scripts/lens/bake.py --preview  # a few minutes, for tweaking
```

The design's prescription is downloaded on first use, and intermediate files
go to `target/lens-bake/`.

### Shaders — `base/resources/shaders/`

The built-in shaders live in the base pack as Slang sources compiled to SPIR-V
by `mise run shaders`; the game loads them by id (`base:lit`). Shaders from
other packs, with their own material properties, are planned.

### Textures — `<namespace>/resources/textures/`

KTX2 files (uncompressed or with Zstandard supercompression), DDS files with
a DX10 header, or PNG files, uploaded exactly as stored. KTX2 and DDS hold
BC7, BC5, BC4, RGBA8, half-float RGBA for values beyond 1 (such as lens
textures), or 16-bit R and RG for heights; a PNG becomes one RGBA8 level
without mipmaps. Whether a texture is sRGB or linear comes from how it is
used. A texture id names the file without its extension: a later pack's file
replaces an earlier pack's whatever the formats, and a pack holding the same
texture in several formats uses the first of KTX2, DDS and PNG, with a
warning. Folders are organized by subject for people; a texture's purpose is
declared by whatever references it.

### Fonts — `<namespace>/resources/fonts/`

TrueType files, preferably variable fonts (one file covering every weight).
A font is a plain asset referenced by id (`inter/inter-variable`); what it is
used for is declared by the typefaces that list it. The base pack's fonts are
not downloaded by any task: place them by hand from the sources listed in
`LICENSES.md`.

### Theme — `base/resources/theme/`

The interface's design system. Every file is read from the `base` namespace
and can be overridden by another pack like any other file. Color roles,
typefaces and text styles are open sets keyed by id: a pack adds its own
(`my-pack:highlight`), and the base game uses them only where they override a
base file.

**Colors** — `theme/colors/<hue>.kdl`: one hue of the palette, every tint from
100 (lightest) to 1000 (darkest) as an sRGB color.

```kdl
tint 100 "#FFF7D6"
tint 200 "#F8E6A0"
// … through 1000
```

A color is referenced as a hue and a tint: `"yellow" 500`. Colors are opaque;
transparency comes from the interface's opacity, never from the palette.

**Themed colors.** Every color the interface uses has a variant per color
scheme, chosen at runtime by the interface's environment: `light`, `dark`,
`light-increased-contrast` and `dark-increased-contrast`. Writing only a color
sets `light`, and the rest are derived: `dark` is the mirrored tint (100 ↔
1000, 500 ↔ 600), `light-increased-contrast` is one tint lighter and
`dark-increased-contrast` one tint darker than `dark`. A block sets variants
explicitly; `light` is required.

```kdl
default {
    light "gray" 100
    dark "gray" 800
}
```

**Color roles** — `theme/color-roles/<role>.kdl`: the colors for one meaning
(`warning`, `critical`, …). `emphasis` and `muted` are required;
`foreground` and `border-emphasis` default to `emphasis`, `border-muted` to
`muted`.

```kdl
emphasis "yellow" 500
muted "yellow" 100
border-emphasis "yellow" 600
border-muted "yellow" 200
```

**Surfaces** — `theme/surfaces/`: the colors shared by every role.
`foreground.kdl` sets `default`, `muted`, `on-emphasis`, `disabled` and
`link`; `background.kdl` sets `default`, `emphasis`, `muted`, `disabled` and
`inverse`; `border.kdl` sets `default`, `emphasis`, `muted` and `disabled`.

**Typefaces** — `theme/typefaces/<typeface>.kdl`: the font files of one
typeface, by role rather than by family, so a pack changes the interface's
font by overriding one file. The base pack defines `text` and `code`
(monospace).

```kdl
font "inter/inter-variable"
```

**Text styles** — `theme/text-styles/<style>.kdl`: a typeface with its size
(logical pixels, before the interface scale), weight (100–900), and optional
`letter-spacing` (em), `case` (`upper` or `lower`) and OpenType `features`.

```kdl
typeface "text"
size 15
weight 500
features "tnum" "zero"
```

## Formats

- **Definitions:** KDL v2.
- **Textures:** DDS, never re-encoded at load, so no quality is lost.
- **Fonts:** TrueType (`.ttf`).
- **3D models:** glTF 2.0 where a source offers it; other formats as needed by
  the models available.

## Not part of the first version

Parts, custom shaders, effects (plumes, explosions, particles), sounds,
localization and UI layouts. Their directories are reserved so packs can grow
into them without a format change.

## Distribution

Pack text files (`.kdl`, `.slang`) are committed to the repository; binary
files (`.dds`, `.spv`, …) are git-ignored, except the base pack's textures,
which no task produces. In the repository, system-solar's binaries come from its
scripts: `mise run generate`. Releases, the pack repository and installation are
described in `docs/launchpad.md`.
