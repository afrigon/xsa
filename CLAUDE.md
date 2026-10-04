# xsa

A space flight simulator in the spirit of Kerbal Space Program and Kitten Space
Agency, written in Rust on Vulkan.

## Priorities

1. **Accuracy.** Physically correct orbital mechanics and flight.
2. **Visual quality.** The target is AAA: PBR, atmospheric scattering,
   volumetric clouds, ray-marched engine plumes and explosions.
3. **Performance.** Nothing above may cost a smooth frame rate.
4. **Code quality.** The project is built to grow large: clean architecture,
   small focused files, modern tech.

The roadmap lives in the `xsa-roadmap` GitHub project
(https://github.com/users/afrigon/projects/7): milestones, one issue per item.
Known deferred work and loose ideas live in `.todo`. Read both before proposing
what to build next. Design documents live in `docs/` (`docs/packs.md`: how game
content is packaged and loaded; `docs/xui.md`: the vision and design of xui, the
UI framework; read it before changing xui or the game's interface).

Linux (Wayland, Hyprland) is the primary and tested platform. Windows must
build, but X11 and Windows are not a testing focus.

## Tasks

Everything runs through mise; tools are pinned in `mise.toml`.

| Task | Does |
| --- | --- |
| `mise run client [--release]` | Compile shaders, run the game with an integrated server (debug builds enable validation layers) |
| `mise run client -- --remote <host>:<port> --fingerprint <hash>` | Run the game against a dedicated server |
| `mise run server [--release]` | Run a dedicated server (`-- --host <address> --port <port>`) |
| `mise run ipc -- <command>` | Send a command to a running instance (`--instance <name>` when several run) |
| `mise run build [--release]` / `build-server` | Build everything / the server-only binary |
| `mise run test` / `fmt` / `lint` | Unit tests / format (120 columns) / Clippy with warnings as errors, full and server-only builds |
| `mise run shaders` | Compile the Slang shaders to SPIR-V and validate them |
| `mise run fetch-assets` / `convert-skybox` | Download third-party source assets / build the skybox cube map from them |

Before calling a change done: `fmt`, `lint` and `test` pass, and a run of the
game prints no validation messages.

## Crates

Each crate is a directory under `crates/`; `data/` holds the game content (packs).
Dependencies point one way: `units` ← `core` ← `packs` ← `commands` ← `server`
← `client` ← `xsa`, and `proto` ← `commands`, `server`, `client`. `core` never
touches files or KDL, and `proto` does not depend on `core` or `packs`.
`xui` and `xui-vulkan` depend on no xsa crate: they are a UI framework
incubated here to become its own repository (`xui-rs`), and the game uses them
like any external library.

| Crate | Responsibility |
| --- | --- |
| `units` | Plain value types shared by every crate: simulation time, time rate |
| `core` | The pure simulation: orbits, coordinate frames, bodies and their state |
| `packs` | Pack loading (see `docs/packs.md`): reads KDL content into `core` types and plain definitions |
| `proto` | Client ↔ server protocol: messages, events, connections, QUIC transport |
| `commands` | Text commands: the command tree, routing, REPL, IPC, completion |
| `server` | The authoritative world and tick loop; integrated and dedicated servers |
| `client` | The game: app, camera, input, config, config documents, renderer, Vulkan |
| `xsa` | The only game binary: `xsa client`, `xsa server`, `xsa ipc` |
| `tools` | Offline asset tools (`convert-skybox`) |
| `xui` | SwiftUI-style UI framework, renderer-agnostic: views, a retained node tree, layout, environment, text shaping, glyph atlas; outputs draw lists |
| `xui-vulkan` | Vulkan backend for `xui`: records a draw list into a host's command buffer, given raw Vulkan handles |

## Architecture

- **Content is data.** Every body, effect and rendering feature is configured
  by pack data; adding an atmosphere to the Moon is a few lines in its data
  file. Nothing is hard-coded per body or per system.
- **One simulation runner.** Local play runs the same `Server` as the dedicated
  binary, on its own thread, over an in-process `Connection`; `--remote` swaps
  in QUIC. Client code is identical either way.
- **The server owns time.** Simulation state is plain data, separate from
  rendering, advanced at a fixed tick (`TICK_RATE_HERTZ`); the time rate scales
  how much simulation time a tick covers, never the tick rate. On join the
  client loads the same packs (refusing a version mismatch) and computes body
  positions from the orbits itself, so ticks carry only time.
- **The server is the authority.** Every change to server state is a client
  message; the server validates it, denies with a reason or applies it,
  broadcasts the resulting event, then replies to the sender.
- **One implementation per action.** Commands (REPL, IPC, in-game console, on
  client or server), UI and key bindings all invoke the same action, so
  behavior never depends on how it was triggered.
- **One scene at every scale.** 1 m to interplanetary distances in a single
  frame; there is no separate map scene.
- **The renderer never sees simulation types.** The client copies transforms
  into persistent scene objects each frame; objects and materials are created
  once and referenced by handle.
- **Pack content is resolved at load time**, never per frame.
- **The interface is built with xui** (`docs/xui.md`). Each page is a
  `ViewController` whose `root()` returns one named view, given the
  controller's actions object; the layout lives in that view's `body`, never
  inline in the controller, and the action handlers are methods of the
  controller.
- **Networking:** QUIC on UDP 1969. The dedicated server generates a
  self-signed identity in `.xsa/server/` and prints its fingerprint; clients
  pin it with `--fingerprint` (the SSH model).
- **Logging:** diagnostics go through `tracing`: WARN by default, `-v` / `-vv` /
  `-vvv` / `-q` to change it, `RUST_LOG` to filter per crate
  (`RUST_LOG=xsa_client=debug`).

## Coordinates, units and precision

- **SI units** everywhere: meters, seconds, radians internally.
- **World frame:** heliocentric, ecliptic J2000. XY is the ecliptic plane, +Z
  its north pole, +X the vernal equinox. Right-handed.
- **Camera local axes:** +Y forward, +X right, +Z up. `Camera::view_rotation`
  is the only place that converts to Vulkan view space (−Z forward, +Y up).
- **Precision:** simulation state is `f64`. Rendering is camera-relative `f32`
  (positions are subtracted from the camera in `f64` first), so solar-system
  distances never reach the GPU.
- **Time:** seconds since J2000.0 (2000-01-01T12:00:00Z); new simulations
  start at the current date.
- **Body ids** are kebab-case scientific names: `sol`, `earth`, `luna`.

## Graphics

- **Vulkan 1.3 baseline.** `ash` 0.38 has no 1.4 bindings; use 1.4 features
  through their extensions.
- **Dynamic rendering and synchronization2** — no `VkRenderPass`, no legacy
  barriers.
- **Bindless:** push constants carry buffer device addresses for the frame,
  object and material buffers; textures go through one bindless descriptor set.
- **Reverse-Z infinite depth.** Clear to 0, test `GREATER`. The near plane
  follows the nearest body surface: depth below ~1e-8 fails the test.
- **Target architecture:** clustered forward (Forward+) with a depth prepass,
  HDR targets and tonemapping. Volumetrics (atmosphere via Hillaire's LUTs,
  clouds, plumes, explosions) are ray-marched passes reading depth, cleaned up
  by temporal anti-aliasing and upscaling.
- **Passes:** each render pass implements `RenderPass`, owns its pipelines and
  resources, and is recorded in the renderer's pass order.
- **Few pipelines:** bindless resources and dynamic state over per-material
  pipelines.
- **Materials are typed:** a `Shader` is a program, a `Material` is a shader
  plus typed parameters in a per-frame GPU buffer.
- Every rendering feature is explained before it is implemented: what it is,
  why the renderer needs it, how it fits the pipeline.
- **Object lifetime:** `Instance`, `Surface`, `Device` and `Allocator` are
  destroyed by `Drop`, in `GpuContext`'s field order. Everything created from the
  device has an explicit `unsafe fn destroy`, called once the GPU is done with
  it — the shape a deletion queue needs.

## Shaders

- Slang, in `data/base/base/resources/shaders/`, compiled offline to SPIR-V by
  `mise run shaders` with `-matrix-layout-column-major` (matches glam). The
  game loads them from the pack and never embeds a shader compiler. The one
  exception is `xui-vulkan`, a library: its shader lives in
  `crates/xui-vulkan/shaders/`, is compiled by the same task and is embedded
  in the crate.
- One top-level file per shader, entry points `vertexMain` and
  `fragmentMain`. Shared code lives in `shaders/common/` and is only imported.
- A material shader is registered in the renderer's `shaders!` list, which
  generates the `Shader` enum.
- **Struct layouts must match between Slang and Rust.** Rust size assertions
  can't see Slang's layout: after changing a shared struct, check `Offset` and
  `ArrayStride` in `spirv-dis` output. glam's `Mat4` forces 16-byte alignment,
  so Slang structs often need explicit padding.

## Commands

- Durations take `s`, `min`, `h` or `t` (ticks); distances take `m`, `km`,
  `Mm` or `Gm` (case-sensitive), a bare number is km; angles are degrees.
- Client commands may span frames: a handler returns `CommandProgress::Done`
  or a `CommandTask` that `App` polls once per frame after drawing, replying
  when it finishes. Commands that finish at once implement
  `ImmediateCommandHandler`; binds start tasks too, logging their result.
- `camera snap [--output <path>] [--region x,y:widthxheight] [--delay 5s]`
  waits the delay in real time while the game keeps rendering, then renders a
  frame, copies the presented image back from the GPU and writes a PNG before
  it replies (the game pauses while it encodes). Without `--output` it goes to
  `$XDG_PICTURES_DIR/xsa/xsa-<wall-clock time>.png`.
- Commands are read from stdin with a `>` prompt (plain lines when stdin is
  not a terminal), and from `xsa ipc` over a socket in `$XDG_RUNTIME_DIR/xsa/`
  (named pipes on Windows).

## Dependencies and assets

- Add a crate only when writing the code ourselves is unreasonable, and justify
  it first. Deliberately avoided: `ash-window` (surfaces are created by hand)
  and runtime shader compilers.
- Game content lives in `data/`. Text files are committed; binaries (`.dds`,
  `.spv`) are git-ignored and produced by tasks. `assets/` holds git-ignored
  third-party source downloads.
- Every third-party asset is listed in `LICENSES.md` with source, author and
  terms, with its license file next to its download.

## Config

- The client's settings live in `$XDG_CONFIG_HOME/xsa/config.kdl` (falling back
  to `~/.config/xsa/config.kdl`), read at startup and by `config reload`.
  Dotted keys are nested nodes: `render.bloom.strength` is
  `render { bloom { strength 0.05 } }`. A repeated block is read last-wins.
- Two models: `crates/client/src/document/config/` holds the document models (KDL,
  key paths, text values, every `ConfigKey`), `crates/client/src/config/` the runtime
  `Config` the renderer and input read. Converting a document into the runtime
  model fills in the defaults, all of which live in `config/default_config.rs`.
- `config set|toggle` edit the document and rebuild `Config`; `--save` and
  `config save` write the file non-destructively (comments and layout kept,
  defaults removed rather than written). `debug.*` is never saved.
- Binds are `bind.<category>.<action>` keys holding a key chord (`f4`,
  `shift+tab`, `` ` ``). Each action runs a typed action directly (the same
  command structs the text commands parse into), never command text.

## Controls

The game starts in the main menu. In game, right-drag orbits the target and
scroll zooms; the debug camera (`camera mode debug`) flies with WASD,
Space/Shift and mouse look (click to capture), and scroll sets its speed. The
camera gets input only while the game view controller is on top; menus release the
mouse. Debug toggles are commands (`config toggle render.stars`,
`config set debug.shader normals`, …), not binds. The default binds:

| Key | Bind | Action |
| --- | --- | --- |
| Tab / Shift+Tab | `camera.target-next` / `target-previous` | Next / previous target |
| Escape | `interface.pause-menu` | Open the pause menu, in game only |
| F3 | `interface.debug-overlay` | Show or hide the debug overlay (FPS, triangles) |

## Gotchas

- Validation layers (`vulkan-validation-layers`) are enabled in debug builds and
  logged under the `vulkan` target; any validation message is a bug. Messages mentioning
  `obs-vkcapture` come from that injected layer.
- `spirv-val` comes from the `spirv-tools` system package (no pinnable mise
  release).
- On Wayland a window appears only once a frame is presented: an invisible
  window means presentation is broken.
- Agent screenshots of the game window are unreliable (Hyprland retiles it, it
  may be on another workspace). Verify rendering numerically and ask the user
  to confirm what is on screen.
