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
`data/system-solar/system-solar/resources/textures/skyboxes/deep-star-maps.dds`
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

- **`category`**: `star`, `planet`, `dwarf-planet`, `moon`, `asteroid` or
  `comet`. A body is a `star` exactly when it has `luminosity` and
  `effective-temperature`.
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
- **`shader "base:planet"`** with **`color-texture`** (sRGB albedo) and
  optional **`normal-texture`** and **`emissive-texture`** (sRGB), each a
  texture id; an emissive texture needs **`emissive-luminance`** (cd/m² for a
  texture value of 1).
  An optional **`hapke`** node shades the surface with Hapke's model of
  particulate surfaces instead of Lambert: `scatter-texture` (red: single-
  scattering albedo w, green: phase-function lobe width b, blue: lobe balance
  (c + 1) / 2) and `surge-texture` (opposition-surge amplitudes and widths:
  B_S0 / 2, h_S, B_C0 / 2, h_C), both linear; `porosity` (Hapke's K) and
  `roughness` (mean slope angle, °); and Sol's look trims `blend`,
  `light-boost` and `gamma-boost`, used only by the Sol-exact variant.
  Maps are equirectangular with the first row at the south pole, longitude 0
  at u = 0.25 and u growing westward (the convention of KSP's Kopernicus and
  Parallax, which Sol's textures follow). Normal maps are tangent-space with
  red toward west and green toward north; z is reconstructed.

### Shaders — `base/resources/shaders/`

The built-in shaders live in the base pack as Slang sources compiled to SPIR-V
by `mise run shaders`; the game loads them by id (`base:lit`). Shaders from
other packs, with their own material properties, are planned.

### Textures — `<namespace>/resources/textures/`

DDS files with a DX10 header (BC7, BC5, BC4 or uncompressed RGBA8), uploaded
exactly as stored; whether a texture is sRGB or linear comes from how it is
used. Folders are organized by subject for people; a texture's purpose is
declared by whatever references it.

## Formats

- **Definitions:** KDL v2.
- **Textures:** DDS, never re-encoded at load, so no quality is lost.
- **3D models:** glTF 2.0 where a source offers it; other formats as needed by
  the models available.

## Not part of the first version

Parts, custom shaders, effects (plumes, explosions, particles), sounds, fonts,
localization and UI layouts. Their directories are reserved so packs can grow
into them without a format change.

## Distribution

Pack text files (`.kdl`, `.slang`) are committed to the repository; binary
files (`.dds`, `.spv`, …) are git-ignored. Releases publish each pack, text
and binaries together, as a versioned `.tar.zst` archive in object storage;
`mise run fetch-assets` downloads the latest release and extracts the binaries
into the working tree.
