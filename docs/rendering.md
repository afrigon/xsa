# Rendering

How the renderer organizes what it draws: the geometries that place vertices,
the programs that shade pixels, their variants and their debug views. The frame itself (render graph, passes,
temporal effects) is described in the project's `CLAUDE.md`.

## Geometry families

A geometry family is one way of placing vertices: its vertex input, the
function that turns it into positions, and the values its vertex stage passes
to the fragment stage.

- **Terrain:** quad sphere tiles of a body, from a shared grid mesh and each
  tile's data. Every body is drawn as tiles. A tile's shape is smooth (on its
  sphere, while its heights are not loaded) or displaced by its heights.
- **Meshes** (planned): vessels and their parts, from vertex buffers.

## Programs

A program is a material model for one geometry family: what a pixel's material
is, how it reflects light, and the typed parameters it reads. It is compiled
whole, with its family's vertex stage, into a single SPIR-V module. Compiling
both stages together lets the compiler check the interface between them and
pass on only the values the fragment stage reads.

A program's id is its family, then its light model. Terrain programs live
under `terrain/`; mesh programs, the family most objects use, sit at the
shader root.

| Program | Family | Draws |
| --- | --- | --- |
| `base:terrain/hapke` | terrain | a body's ground reflecting light by Hapke: its terrain layers, and its surfaces up close |
| `base:terrain/lambert` | terrain | a body's ground reflecting light by Lambert, from textures |
| `base:terrain/emissive` | terrain | a body that only emits light, such as a star |
| `base:hapke` (planned) | meshes | objects on regolith ground, such as scattered rocks |
| physically based programs (planned) | meshes | a vessel's metal, paint and glass |

The light models themselves are modules in `shaders/light/`, shared by every
program using them. Each program fills a material sample, what its material is at
the pixel (albedo and roughness), which the generic debug views read, then
shades it.

The shipped shaders grow with the number of programs, never with a product of
families, light models and options. A material names its program and fills
its parameters; `docs/packs.md` describes each one's keys. In the renderer, a
program's parameters are a struct implementing `ShaderData`, which names the
program reading it.

## Variants

What changes within a program is a specialization constant: a value fixed
when a pipeline is built, so the driver compiles away the paths it disables.
One program gives one pipeline per combination in use.

| Constant | Program | Values |
| --- | --- | --- |
| Tile shape | every terrain program | smooth, displaced |
| Shading range | terrain programs with surfaces | far (terrain layers only), near (with surfaces) |
| Debug view | every program | none, or one of its views |

Pipelines without a debug view are built at load; a debug view's pipeline is
built the first time it is shown.

A choice that differs between draws of the same pipeline is data the program
reads, branching the same way across a draw: whether a body's colour comes from
a layer, a texture or a constant.

## Debug views

A debug view replaces a draw's shaded colour with something to inspect. Each
view belongs to the level whose data it shows, and is a session setting (never
saved) of that level:

| Key | Views | Shown on |
| --- | --- | --- |
| `render.shader.override` | `normals`, `depth`, `motion`, `lighting`, `triangles`, `albedo`, `roughness` | every draw, from its vertices and material sample |
| `render.shader.terrain.override` | `lod`, `scale`, `height-levels` | every terrain program |
| `render.shader.surface.override` | `classes`, `planes`, `fade` | terrain programs with surfaces |

Where several are set, a draw shows the most specific one it supports: its
surface view, then its family's, then the generic one. Adding a family or a
feature adds its own key, and its views are code in that family or feature.

## Surfaces

Up close, the Hapke program adds a biome's surfaces (`docs/packs.md`) to the
terrain layers:

- **Classes:** each pixel weighs the biome's flat and slope surfaces by the
  terrain's slope, moved by curvature, relative height and a seeded noise, and
  blends them by height where both show.
- **Projection:** each surface is projected along the two body axes the ground
  faces most, one where it faces a single axis.
- **Repeat sizes:** a surface keeps its true size until its repeat spans fewer
  than 512 pixels, then grows 3× each time the pixel's footprint grows 3.4×. A
  pixel crossfades the two sizes around its own, each shifted by a seeded
  offset, so the ground never shows a dense grid and stays textured at every
  distance. The camera's place within every repeat is computed in f64 each
  frame, so pixels add only their small offset in f32.
- **Fade:** surfaces fade in over two levels as the colour layer is magnified
  past its finest texels, and cover every distance where it is.
- **Colour:** a surface multiplies the colour layer by its colour over its
  average, keeping the body's measured albedo.
- **Relief:** near the camera the vertex stage moves the ground by the
  displacement of the surfaces its pixels show, at their true size, fading out
  as their repeat starts to grow. Shading normals already hold that relief from
  the normal maps, and collision does not see it.

The surface debug views show how these choices land:

- **`classes`:** flat surface in red, slope surface in green.
- **`planes`:** the projection's weights, body axes x, y and z in red, green and
  blue.
- **`fade`:** how far surfaces have faded in, black to white.

## Shader files

```
shaders/
  frame.slang  draw.slang  fragment.slang      read by every program
  light/                       light models shared by every program: star, lambert, hapke, roughness
  noise.slang                  seeded noise
  debug/                       generic views
  terrain/                     the terrain family: vertex stage, cube map addressing, heights, terrain layers, output,
                               its views
    hapke.slang  lambert.slang  emissive.slang    its programs
    surface/                   surfaces and their views
  skybox.slang
  post/
```

A program is a file with `[shader(...)]` entry points, `vertexMain` and
`fragmentMain`; every other file is a module, imported by its full path from
the shader root (`import terrain.layers`).
