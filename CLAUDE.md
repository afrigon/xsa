# xsa

A space flight simulator in the spirit of Kerbal Space Program and Kitten Space
Agency, written in Rust on Vulkan.

## Priorities

1. **Accuracy.** Physically correct orbital mechanics and flight.
2. **Visual quality.** The target is AAA: PBR, atmospheric scattering,
   volumetric clouds, ray-marched engine plumes and explosions.
3. **Performance.** Nothing above may cost a smooth frame rate.

The roadmap, planned features and known deferred work live in `.todo`, not
here. Read it before proposing what to build next. Design documents live in
`docs/`; `docs/packs.md` defines how game content (bodies, systems, materials,
textures, text) is packaged and loaded.

## Platforms

Linux (Wayland, Hyprland) is the primary and tested platform. Windows must
build, but X11 and Windows are not a testing focus — don't spend effort on
them unless asked.

## Tasks

Everything runs through mise; tools (`rust`, `slang`, `xh`) are pinned in
`mise.toml`.

| Task | Does |
| --- | --- |
| `mise run dev` | Run the game with an integrated server (debug build, validation layers on) |
| `mise run dev -- --remote <host>:<port> --fingerprint <hash>` | Run the game against a dedicated server |
| `mise run server` | Run the dedicated server (`-- --host <address> --port <port>`) |
| `mise run build` | Build every crate |
| `mise run test` | Unit tests |
| `mise run fmt` | Format (`rustfmt.toml`: 120 columns) |
| `mise run lint` | Clippy with warnings as errors |
| `mise run fetch-assets` | Download third-party assets into `assets/` |
| `mise run convert-skybox` | Build the skybox cube map from the star map EXR |

Before calling a change done: `fmt`, `lint` and `test` pass, and a run of the
game prints no validation messages.

## Assets and licensing

- `assets/` is git-ignored and populated by `fetch-assets`; the skybox then
  needs `convert-skybox`. The game starts without assets and skips what is
  missing.
- Every third-party asset is listed in `LICENSES.md` with source, author and
  terms, and its license or credit file sits next to it in `assets/`. Adding
  an asset means updating both.

## Project map

A Cargo workspace; each crate is a top-level directory.

| Crate | Kind | Responsibility |
| --- | --- | --- |
| `core/` (`xsa-core`) | library | Simulation data in SI units (`simulation.rs`), coordinate frames (`frames.rs`: ecliptic ↔ equatorial, with tests) |
| `proto/` (`xsa-proto`) | library | Client ↔ server protocol: `messages.rs` (`ClientMessage`, `ServerEvent`, wire structs, bitcode), `connection.rs` (`Connection`, `ClientLink`), `network.rs` (QUIC transport, server identity, fingerprint pinning) |
| `server/` (`xsa-server`) | library + binary | `lib.rs`: the authoritative `Server` tick loop and `start_local` for the integrated server; `main.rs`: the dedicated server |
| `client/` (`xsa`) | binary | The game: `app.rs` (window, frame loop, input polling, camera modes, server events → scene), `camera/`, `input.rs`, `mesh.rs`, `renderer/`, `vulkan/`, `shaders/`, `build.rs` (compiles shaders) |
| `tools/` (`xsa-tools`) | binary | `convert-skybox`: equirectangular EXR → cube map DDS |

Dependencies point one way: `core` ← `server` ← `client`, and `proto` ←
`server`, `client`. `proto` does not depend on `core`: its wire structs are
independent of simulation internals.

**One simulation runner.** Local play starts the same `Server` the dedicated
binary runs, on its own thread, connected through an in-process `Connection`;
`--remote` swaps in a QUIC `Connection`. The client code is identical either
way. The server owns simulation time; bodies on rails are computed from their
definitions, so ticks carry time rather than full state.

Each frame, `App` runs `update` (poll the connection, poll input, update the
camera) then `draw` (sync the scene from replicated bodies, render). Window
events only record input; `poll_input` acts on it.

**Networking:** QUIC (`quinn`, `rustls` with the `ring` provider only) on UDP
port 1969 by default. The dedicated server generates a self-signed identity on
first start in `.xsa/server/` and prints its SHA-256 fingerprint; clients pin
it with `--fingerprint` (the SSH model). Messages are length-prefixed bitcode
frames on one bidirectional stream.

## Coordinates, units and precision

- **SI units** everywhere: meters, seconds, radians internally.
- **World frame:** heliocentric, ecliptic J2000. XY is the ecliptic plane, +Z
  its north pole, +X the vernal equinox. Right-handed.
- **Camera local axes:** +Y forward, +X right, +Z up, matching the world at
  identity orientation. `Camera::view_rotation` is the only place that
  converts to Vulkan view space (−Z forward, +Y up).
- **Precision:** simulation state is `f64` (`DVec3`, `DQuat`). Rendering is
  camera-relative `f32` — positions are subtracted from the camera in `f64`
  before conversion — so solar-system distances never reach the GPU.
- **Body ids** are kebab-case scientific names: `sol`, `earth`, `luna`.

## Architecture

- **Simulation state is plain data, separate from rendering, advanced at a
  fixed timestep.** Saves, debug-mode editing and time warp all depend on it.
- **One scene at every scale.** Rendering must handle 1 m to interplanetary
  distances in a single frame; there is no separate map scene.
- **The renderer never sees simulation types.** `App` owns the simulation and
  copies transforms into persistent `SceneObject`s each frame; objects and
  materials are created once and referenced by handle.
- **Materials are typed, Unity-style:** a `Shader` is a program, a `Material`
  is a shader plus typed parameters, stored in a per-frame GPU buffer.

## Graphics

- **Vulkan 1.3 baseline.** The released `ash` (0.38) has no 1.4 bindings; use
  1.4 features through their extensions.
- **Dynamic rendering and synchronization2** — no `VkRenderPass` objects, no
  legacy barriers.
- **Shader data through buffer device addresses:** push constants carry GPU
  pointers to the frame, object and material buffers plus indices. Textures
  go through one bindless descriptor set (`vulkan/bindless.rs`).
- **Reverse-Z infinite depth.** Clear depth to 0, test `GREATER`. The near
  plane follows the nearest body surface (`Camera::fit_near_plane`); depth
  values below ~1e-8 fail the depth test, which is why it can't stay tiny.
- **Sub-pixel bodies** are drawn as fixed-size dots (`client/shaders/point.slang`)
  instead of vanishing.
- **Rendering architecture target:** clustered forward (Forward+) with a
  depth prepass, HDR render targets and a tonemapping pass. Volumetric effects
  (atmosphere via Hillaire's LUT technique, clouds, plumes, explosions) are
  ray-marched passes that read depth, cleaned up by temporal anti-aliasing and
  upscaling.
- **Keep the pipeline count small** — bindless resources and dynamic state
  over per-material pipelines, and a `VkPipelineCache` persisted to disk.

## Shaders

- Written in Slang, compiled offline to SPIR-V by `client/build.rs` with `slangc`
  (pinned in `mise.toml`) and `-matrix-layout-column-major` to match glam.
  The binary never embeds a shader compiler.
- One top-level file per shader, with entry points named `vertexMain` and
  `fragmentMain`. Shared code lives in `client/shaders/common/` and is imported, never
  compiled on its own.
- Adding a material shader: create the file, then add it to the `shaders!`
  list in `client/src/renderer/material.rs`, which generates the `Shader` enum, its
  pipeline order and its SPIR-V mapping.
- **Struct layouts must match between Slang and Rust.** The `#[repr(C)]`
  structs in `client/src/renderer/gpu_data.rs` and `material.rs` mirror
  `client/shaders/common/scene.slang`. Rust-side size assertions can't see Slang's
  layout: after changing a shared struct, check the compiled SPIR-V
  (`spirv-dis` on `target/debug/build/xsa-*/out/*.spv`, look at `Offset` and
  `ArrayStride`). glam's `Mat4` forces 16-byte alignment, so Slang structs
  often need explicit padding.

## Vulkan object lifetime

- Objects created once — `Instance`, `Surface`, `Device`, `Allocator` — are
  destroyed by `Drop`, ordered by `Renderer`'s field declaration order.
- Everything created from the device has an explicit `unsafe fn destroy`,
  called after the GPU is idle. This is the shape a deletion queue needs:
  resources are freed only once the frames that used them have finished,
  which streaming terrain and textures will depend on.

## Dependencies

Add a crate only when writing the code ourselves is unreasonable, and justify
it first. Deliberately avoided:

- `ash-window` — surfaces are created by hand in `client/src/vulkan/surface.rs`.
- `shaderc` / runtime shader compilers — shaders are compiled offline.

## Code conventions

- **No tuples in our own data shapes** — function arguments, return values,
  fields, constant tables and local destructuring use named structs or
  separate bindings; handles are structs with a named field, not tuple
  structs. Language-level tuples are fine: loop patterns over `enumerate` or
  `zip`, and external API return values (ash) destructured into named
  bindings where they arrive.
- Constants over magic numbers, including binary format offsets and flags.

## Controls

| Input | Action |
| --- | --- |
| Right-drag | Orbit the target (game camera) |
| Scroll | Zoom (game camera) / speed (debug camera) |
| Tab / Shift+Tab | Next / previous target |
| F1 | Toggle the debug fly camera (WASD, Space/Shift, mouse look; click to capture, Escape to release) |
| F2 | Toggle the skybox |
| 1–5 | Shader override: lit, normals, depth, triangles, lighting |
| \` | Wireframe |

## Development gotchas

- Validation layers are a system package (`vulkan-validation-layers`), enabled
  automatically in debug builds; their messages print to stderr. Treat any
  validation message as a bug. Messages mentioning `obs-vkcapture` come from
  that injected layer, not from this code.
- On Wayland a window only appears once a frame has been presented — an
  invisible window means presentation is broken, not window creation.
- Screenshots of the game window taken by an agent are unreliable: Hyprland
  retiles the window, and it may sit on another workspace or behind other
  windows. Verify rendering numerically where possible and ask the user to
  confirm what is on screen.
