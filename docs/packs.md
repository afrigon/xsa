# Packs

Everything that defines what exists in the game and how it looks — bodies,
star systems, simulations, parts, materials, textures, sounds, text — comes
from packs: directories of KDL files and assets. `packs/base` holds only what
the game itself needs (built-in shader types, default textures, UI) and
defines no star system. Star systems ship as their own packs — the real solar
system is `packs/sol` — and custom systems, reskins and mods are packs layered
on top. With no system installed, the game asks the player to install one.

## Pack structure

```
packs/<pack>/
  pack.kdl                    manifest
  data/                       gameplay and physics — read by the server
    bodies/                   intrinsic properties of each body
    systems/                  how bodies relate: hierarchy and orbits
    simulations/              which systems make up a playable simulation
    parts/                    rocket parts
  resources/                  presentation — read by the client
    objects/                  mirrors data/ one-to-one
      bodies/  systems/  simulations/  parts/
    materials/                shader type, texture slots and usage parameters
    textures/  models/  sounds/  fonts/    shared assets, organized freely
    lang/                     localization files
```

A pack may contain `data/`, `resources/` or both. The server only reads `data/`;
the client reads both. Trimming the resources from a server installation, or
streaming a server's packs to connecting clients, is a distribution concern
and does not change the format.

## Manifest

`pack.kdl` declares the pack id, display name key, version, pack format
version, description key and the packs it depends on. The loader rejects packs
whose format version it doesn't support.

## Ids and namespaces

Every entry has a namespaced id, `<pack>:<path>`, derived from its location:
`packs/base/data/bodies/earth.kdl` is `base:bodies/earth`. Namespaces let two
packs add entries with the same name without colliding, and make overriding
an explicit act. Body ids use kebab-case scientific names (`sol`, `earth`,
`luna`).

## Stack and overrides

The effective game content is built from an ordered list of packs. A later
pack replaces an earlier pack's entry with the same id, one whole file at a
time. Field-level merging between packs is deliberately not supported.
Higher-resolution textures, reskins and rebalanced systems are all plain
overrides.

## Data

The split between bodies and systems follows what is intrinsic to a body and
what depends on its surroundings.

| `data/bodies/` — intrinsic | `data/systems/` — relational |
| --- | --- |
| radius and shape (flattening) | parent body |
| gravitational parameter (μ = G·M) | orbital elements and their epoch |
| rotation period | spin axis orientation in the system frame |
| axial tilt relative to its own orbital plane | initial rotation angle |
| terrain shape: heightmaps and procedural parameters | tidal locking, overriding the body's rotation |
| atmosphere physics: density and pressure profiles | |

A body definition can be reused in several systems. Systems also contain
**barycenters** — massless points that bodies orbit and that orbit something
themselves — so pairs like Earth–Moon, Pluto–Charon or binary stars stay on
exact Keplerian rails. `data/simulations/` composes one or more systems into
what the player selects in game, including their placement relative to each
other and the start date.

**Anything the physics needs lives in `data/`**, because the server never
reads resources. Terrain shape decides where a lander touches down, and
atmospheric density drives drag and heating, so both are data even when part
of them is binary.

## Resources

- **`objects/` mirrors `data/`.** `data/bodies/earth.kdl` is presented by
  `resources/objects/bodies/earth.kdl`: its material, levels of detail,
  atmosphere and cloud visuals, icon, and the localization keys of its name
  and description. Systems and simulations have object files too, for icons,
  text and — for simulations — the skybox.
- **Shared assets** (`textures/`, `models/`, `sounds/`, `fonts/`) sit at the
  top level so any object or material can use them. Their folders are
  organized by subject for people (`textures/bodies/earth/color.dds`); the
  engine gives folder and file names no meaning.
- **A texture's purpose is declared where it is used.** A material names which
  texture fills which slot (`base-color`, `normal`, …), so new texture roles
  need no new folder conventions.
- **A skybox is a cube-map texture.** The simulation's object file references
  it together with the frame it is aligned to.

### Texture metadata

- **What the texture is** lives in an optional sidecar next to it
  (`color.dds.kdl`): color space (sRGB or linear), mipmap generation,
  filtering, animation frames and timing, streaming hints.
- **How it is used** lives in the material: slot, tiling, scale, scroll
  speed. The same texture can be used differently by several materials.

## Localization

Every player-visible string in resources is a key (`base.bodies.earth.name`)
resolved through `resources/lang/<language>.kdl`. English is the reference
language.

## Formats

- **Definitions:** KDL.
- **Textures:** DDS, uploaded exactly as stored (BC7, BC6H or uncompressed) —
  never re-encoded at load, so no quality is lost.
- **3D models:** glTF 2.0 where a source offers it; other formats as needed by
  the models available. Simple geometry (spheres, cube-spheres, boxes) is
  defined inline in KDL.

## Not part of the first version

Parts, custom shaders, effects (plumes, explosions, particles), sounds, fonts
and UI layouts. Their directories are reserved so packs can grow into them
without a format change.

## Distribution

Pack definitions (`.kdl`) are committed to the repository next to the code
that reads them. Large binary resources are not: they are published as a
versioned archive in object storage, and `mise run fetch-assets` downloads a
pinned version, verifies its checksum and extracts it into the pack.
