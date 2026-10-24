# xsa

A space flight simulator in the spirit of Kerbal Space Program and Kitten Space
Agency, written in Rust on Vulkan.

## Priorities

1. **Performance and visual quality.** The target is AAA (PBR, atmospheric
   scattering, volumetric clouds, ray-marched engine plumes and explosions)
   at a smooth frame rate.
2. **Code quality.** The project is built to grow large: clean architecture,
   small focused files, modern tech.
3. **Realism.** Physically based simulation and rendering, weighed against
   performance and visual quality with common sense: find the sweet spot
   instead of paying for accuracy nobody can see.

Every decision picks the best option by these priorities. A worse option is
never preferred because it is easier to implement or already exists.

The roadmap, known deferred work and loose ideas live in the `xsa-roadmap` GitHub
project (https://github.com/users/afrigon/projects/7): milestones, one issue per
item. Read it before proposing what to build next. Design documents live in
`docs/` (`docs/packs.md`: how game content is packaged and loaded;
`docs/launchpad.md`: releases, the pack repository, instances and `launchpad`;
`docs/xui.md`:
the vision and design of xui, the UI framework; read it before changing xui or
the game's interface; `docs/rendering.md`: how drawing is organized into
geometry families, programs, variants and debug views; read it before adding a
shader or a debug view).

Linux (Wayland, Hyprland) is the primary and tested platform. Windows must
build, but X11 and Windows are not a testing focus.

## Tasks

Everything runs through mise; tools are pinned in `mise.toml`.

| Task | Does |
| --- | --- |
| `mise run client [--release]` | Compile shaders, run the game with an integrated server (debug builds enable validation layers) |
| `mise run client -- --remote <host>:<port> --fingerprint <hash>` | Run the game against a dedicated server |
| `mise run server [--release]` | Run a dedicated server (`-- --host <address> --port <port>`) |
| `mise run ipc -- <command>` | Send a command to a running game or server (`--name <name>` when several run) |
| `mise run launchpad [--release] -- <arguments>` | Run `launchpad`, the instance and pack manager (`docs/launchpad.md`) |
| `mise run build [--release]` / `build-server` | Build everything / the server-only binary |
| `mise run test` / `fmt` / `lint` | Unit tests / format (120 columns) / Clippy with warnings as errors, full and server-only builds |
| `mise run shaders` | Compile the Slang shaders to SPIR-V and validate them |
| `mise run docs [-- --open]` | Build (and open) the protocol reference from the `proto` crate's documentation |
| `mise run generate -- <pack> [<argument> …]` | Generate a pack's assets and `LICENSE` from their sources with its `scripts/` (`system-solar [<recipe> …]`, `base-surfaces --density <texels per meter>`) |

Before calling a change done: `fmt`, `lint` and `test` pass, and a run of the
game prints no validation messages.

## Crates

Each crate is a directory under `crates/`; `data/` holds the game content (packs).
Dependencies point one way: `units` ← `core` ← `packs` ← `commands` ← `server`
← `client` ← `xsa`, and `proto` ← `commands`, `server`, `client`. `core` never
touches files or KDL (it uses `resource-loader`'s terrain types, never its readers), and `proto` does not depend on
`core` or `packs`.
`xui` and `xui-vulkan` depend on no xsa crate: they are a UI framework
incubated here to become its own repository (`xui-rs`), and the game uses them
like any external library. `render-graph-vulkan` depends on no xsa crate
either: the renderer uses it like an external library, through its public API
only. `memory-vulkan` depends on no xsa crate: `render-graph-vulkan`,
`xui-vulkan` and `client` all create their GPU objects through it.
`resource-loader` depends on no xsa crate: it decodes resource files, one module
per kind of file behind a feature of the same name (`texture`), so each crate
enables only the kinds it reads, and `utils` holds the value types they share
(`HalfFloat`, `CubeFace`). It only reads: writing files is the job of the tools
that produce them. `resource-vulkan` depends on no xsa crate
either, only on `memory-vulkan` and `resource-loader`: the client adds its own
kinds (terrain layer headers, lens textures) through its `ResourceKind` trait.
`config` depends on no xsa crate: `logging`, `commands`, `server` and `client`
define their settings through it.

| Crate | Responsibility |
| --- | --- |
| `units` | Plain value types shared by every crate: simulation time, time rate |
| `core` | The pure simulation: orbits, coordinate frames, bodies and their state, terrain heights over the pages a side holds (`HeightPages`) |
| `packs` | Pack loading (see `docs/packs.md`): reads KDL content into `core` types and plain definitions, and a terrain's height pages (`HeightPageReader`) |
| `proto` | Client ↔ server protocol (see Protocol): requests, replies, events, frames, the world state and users, connections and the QUIC transport, the client's session |
| `commands` | Text commands: the command tree, routing, REPL, IPC, completion; the transcripts of IPC and startup scripts and the `logging` settings every process has |
| `server` | The authoritative world and tick loop; integrated and dedicated servers |
| `client` | The game: app, camera, input, config, renderer, Vulkan |
| `xsa` | The only game binary: `xsa client`, `xsa server`, `xsa ipc` |
| `launchpad` | `xsa-launchpad`, the `launchpad` binary: the instance and pack manager (`docs/launchpad.md`), the converter of terrain layers into `.terrain` files (`terrain create`), their inspector (`terrain inspect`) and their extractor back to KTX2 (`terrain extract`); thin command handlers over a context carrying the output shell, calling operations in `ops` |
| `logging` | The verbosity flags (`-v`, `-q`), the log filter (a config value) and the `tracing` subscriber shared by `xsa` and `launchpad`, whose filter can change while the process runs |
| `config` | Config stores (see Config): the session and saved layers over a KDL file, sections that name each key once (`ConfigSection`), value types (`ConfigValue`, `ConfigChoice`) |
| `xui` | SwiftUI-style UI framework, renderer-agnostic: views, a retained node tree, layout, environment, text shaping, glyph atlas; outputs draw lists |
| `xui-vulkan` | Vulkan backend for `xui`: records a draw list into a host's command buffer, creating its objects through the host's `MemoryContext` |
| `resource-loader` | Resource file decoding, a feature per kind: `texture` reads KTX2, DDS and PNG into GPU-ready images with their mip chains; `terrain` defines the `.terrain` format (pages of a cube map at every level) and reads it |
| `memory-vulkan` | GPU memory and object lifetime: `MemoryContext` (device, allocator, deletion queue, video memory budget), `DeviceObject`, `Buffer`, `Image` (in shared or dedicated memory), timelines and present fences, the `TransferQueue` every copy between CPU and GPU goes through (whole resources, or regions of an image frames keep sampling), and the `BindlessTable` whose owned slots return once no frame reads them. Its `test-device` feature adds `TestDevice`, a headless device for GPU tests that collects validation messages |
| `resource-vulkan` | `Resources`: background, reference-counted loading of the files content refers to, on a worker pool of its own; typed `ResourceHandle`s, groups with byte progress, priorities, and a memory budget that evicts unreferenced cached resources least recently used first. Ships the texture and cube map kinds. Its `Streamer` keeps paged data resident: a client registers a `StreamedSet` with its `PageSource`, requests the pages it wants each frame with a score, and reads each page's `PagePlacement` (the page or its nearest resident coarser page); pages live in 2D atlas chunks of about 32 MiB per format that grow and shrink with the shared budget, and stay decoded in RAM once evicted, where clients that also read pages on the CPU (terrain heights) take their bytes |
| `render-graph-vulkan` | Frame graph for Vulkan: passes declare the resources each step uses; it culls passes, places every barrier, layout transition and attachment load and store op, and allocates transient and history images and transient buffers |

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
  request; the server validates it, refuses it with a `RequestError` or applies it,
  broadcasts the resulting events, then replies to the sender (see Protocol).
- **One implementation per action.** Commands (REPL, IPC, in-game console, on
  client or server), UI and key bindings all invoke the same action, so
  behavior never depends on how it was triggered.
- **Terrain streams by tile.** Quad sphere tiles split each cube face in the same equal-angle coordinates as
  `.terrain` pages, so a tile at depth d is exactly the page at level d. Each drawn tile requests the pages of every
  layer it needs at its distance and reads a fixed block of placements; terrain out of view requests nothing.
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
- **Logging:** diagnostics go through `tracing`, filtered by `logging.filter`: WARN by default, a level or per-crate
  directives (`warn,xsa_client=debug`). `-v` / `-vv` / `-vvv` / `-q` or `RUST_LOG` set it for the session only, a
  flag winning over the variable.

## Protocol

The `proto` crate defines it; `mise run docs -- --open` renders the reference from its documentation.

- **Three kinds of traffic.** A client sends requests; the server sends snapshots, events and replies
  (`ServerFrame`). A client changes its copy of the session (`SessionState`) only from snapshots and events, never
  from replies.
- **Every request gets one reply**, sent to its sender after the events the request caused, so the sender's state
  already reflects the request when its reply arrives. A request with nothing to return replies `()`. A refusal is
  a `RequestError` saying what the sender can do: `Invalid` (other arguments may succeed), `Forbidden` (the request
  is not open to it in this session) or `Conflict` (the session's state refuses it).
- **A request states its contract in its own file:** its `RequestType` names its reply type and the session
  environments it is available in. `requests!` lists the requests and `Reply` lists the reply types; nothing else
  pairs them.
- **Events go to every connection that acts as an actor**, the one whose request caused them included. An event
  never encodes who receives it: the server decides.
- **Joining:** a client sends `JoinRequest` and receives a snapshot, the `UserJoinedEvent` announcing its user, then
  a `JoinReply` naming that user. Until it joins, every other request is `Forbidden`.
- **The operator never joins.** The dedicated server admits its own connection as `Peer::Operator` when it creates
  it and sends it a snapshot. A client never claims who it is.
- **Disconnecting:** either side closes a connection with a `DisconnectReason` (`Connection::close`,
  `ClientLink::close`), carried as a QUIC close code; a connection that ends without one is `Lost`. A server stops
  only through its `ServerHandle`, which closes every connection with `ServerStopped`. A frame that fails
  to decode closes the connection as a `ProtocolViolation`, logged at WARN. The server handles every disconnect
  alike: it removes the user and broadcasts `UserLeftEvent`.
- **Naming:** a request is a `<Name>Request`, an event a `<Name>Event` in the past tense and a reply a `<Name>Reply`,
  each in its own file in `request/`, `event/` or `reply/`, which hold nothing else besides their dispatch enum
  (and `RequestType` in `request/`); refusals live in `error/`, ids in `frame/`. Requests say
  **Set** for an absolute value and **Step** for a change relative to the server's value.
- **Documentation:** every public type and field in `request`, `reply`, `event`, `error`, `frame`, `world` and `user`
  has a doc comment; their `missing_docs` lint makes `lint` fail otherwise.

## Terms

Names in code, data and documents use these words in these meanings only:

- **Terrain:** a body's shape and appearance seen from a distance, modeled after real data (heights, colour,
  normals, Hapke maps).
- **Terrain detail:** procedural detail added up close.
- **Surface:** the materials (textures) of the ground up close.
- **Biome:** a region's kind of ground.
- **Scatter:** props placed on the ground: rocks, trees, bushes.
- **Vehicle:** an assembled vessel.
- **Part:** a component of a vehicle.
- **Site:** a point of interest on a body: a launch site, a building, a base, a landing site.
- **Console:** the in-game console.
- **Shell:** the terminal output of a process (the game, the dedicated server, `launchpad`).
- **Launchpad:** the instance and pack manager for xsa.

### Protocol

- **Request:** what a client asks the server to do (`<Name>Request`).
- **Reply:** the server's answer to one request, sent only to its sender: the request's typed reply, or a
  `RequestError`.
- **Event:** a fact the server announces, named in the past tense (`<Name>Event`).
- **Snapshot:** the whole world state, sent to a client when it joins.
- **Frame:** a value a connection carries: a `ClientFrame` (a request and its id) or a `ServerFrame` (a snapshot, an
  event or a reply).
- **User:** a person in a session.
- **Actor:** who a connection acts as: a user, or the operator.
- **Operator:** the dedicated server's own process, running the commands of its shell, IPC and startup scripts.
- **Disconnect reason:** why a connection ended, whichever side ended it.

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
- **Hardware baseline: NVIDIA RTX 2080 SUPER.** Any Vulkan feature or extension
  that GPU supports may be required, with no fallback path, as long as later
  GPUs support it too.
- **Dynamic rendering and synchronization2** — no `VkRenderPass`, no legacy
  barriers.
- **Reverse-Z infinite depth.** Clear to 0, test `GREATER`, `D32_SFLOAT`.
  Depth is `near / distance`, and a float keeps the same relative precision at
  every magnitude, so one fixed near plane (0.1 m) serves 1 m to
  interplanetary distances.
- **Target architecture:** clustered forward (Forward+) with a depth prepass,
  HDR targets and tonemapping. Volumetrics (atmosphere via Hillaire's LUTs,
  clouds, plumes, explosions) are ray-marched passes reading depth, cleaned up
  by temporal anti-aliasing and upscaling.
- **Render graph:** every frame is built with the `render-graph-vulkan` crate,
  and every pass goes through it.
- **Materials are typed:** a `Shader` is a program (`docs/rendering.md`), a
  `Material` is a shader plus typed parameters in a per-frame GPU buffer.
- Every rendering feature is explained before it is implemented: what it is,
  why the renderer needs it, how it fits the pipeline.
- **Object lifetime:** `memory-vulkan`'s `MemoryContext` owns the device, its
  allocator and a deletion queue. Everything created from the device is wrapped
  in a `DeviceObject` (or built from them, like `Buffer` and `Image`) as soon as
  it exists; dropping it, from any thread, queues its destruction until the GPU
  has finished all work submitted so far and every recording open at that
  moment, so replacing an object never waits for the GPU and a failure halfway
  through a constructor leaks nothing. A `Recording` (`Timeline::begin`) begins
  a command buffer; submitting it ends and submits it, and dropping it
  abandons it. Plain actions wait in the
  same queue (`defer_action`). The device is destroyed with the last
  `MemoryContext` clone, so every GPU object drops before `GpuContext`; the
  instance and surface are destroyed by `Drop` after it.

## Shaders

- Slang, in `data/base/base/resources/shaders/`, compiled offline to SPIR-V by
  `mise run shaders` with `-matrix-layout-column-major` (matches glam). The
  game loads them from the pack and never embeds a shader compiler. The one
  exception is `xui-vulkan`, a library: its shader lives in
  `crates/xui-vulkan/shaders/`, is compiled by the same task and is embedded
  in the crate.
- Files are grouped by feature (`docs/rendering.md`): `terrain/` (the terrain
  family's vertex stage, layers, programs and views), `light/` (light models
  every program shares), `debug/` (the generic debug views) and `post/`
  (post-processing); `frame.slang`, `draw.slang` and `fragment.slang` at the
  root are read by every program. Imports name the full path from the shader
  root (`import light.hapke`). Slang looks for an import next to the importing
  file first, so a module path must not also exist under the importer's folder.
- A program is a file with `[shader(...)]` entry points, `vertexMain` and
  `fragmentMain`, compiled whole to a `.spv` next to it; every other file is a
  module, only imported. A program is named by its family and light model
  (`terrain/hapke`), registered in the renderer's `shaders!` list, which
  generates the `Shader` enum; the struct of its parameters implements
  `ShaderData`, naming that shader.
- What varies within a program (tile shape, debug views) is a specialization
  constant, declared with `[vk::constant_id(n)]` and mirrored by an enum in the
  renderer; `ForwardPipeline` lists each pipeline's values by constant id.
- **Struct layouts must match between Slang and Rust.** Rust size assertions
  can't see Slang's layout: after changing a shared struct, check `Offset` and
  `ArrayStride` in `spirv-dis` output. glam's `Mat4` forces 16-byte alignment,
  so Slang structs often need explicit padding.

## Commands

- Durations take `s`, `min`, `h` or `t` (ticks); altitudes take `m`, `km`,
  `Mm` or `Gm` (case-sensitive), a bare number is km; speeds take `m/s`, `km/s`, `Mm/s` or `Gm/s`, a bare number
  is m/s; angles are degrees.
- `time override <rate>` runs time at a rate without changing the configured one, until `time override none`; only
  singleplayer allows it (its request's `SESSION_ENVIRONMENTS`), and the pause menu stops time with one.
- Client commands may span frames: a handler returns `CommandProgress::Done`
  or a `CommandTask` that `App` polls once per frame after drawing, replying
  when it finishes. Commands that finish at once implement
  `ImmediateCommandHandler`; binds start tasks too, logging their result.
- `camera tp` stands the debug camera at an altitude above the ground and holds it there while the heights at that
  spot stream in, replying once the ground there is as fine as it will get.
- `camera snap [--output <path>] [--region x,y:widthxheight]` keeps rendering
  until temporal effects (exposure metering, anti-aliasing history) reflect the
  scene, then renders a frame, copies the presented image back from the GPU and
  writes a PNG before it replies (the game pauses while it encodes). Without
  `--output` it goes to `$XDG_PICTURES_DIR/xsa/xsa-<wall-clock time>.png`. It
  does not wait for animations: set `animation.duration-scale 0` first so
  camera transitions, eye adaptation and interface transitions finish at once.
- `xsa client --script <path>` and `xsa server --script <path>` run a file of commands at startup, one per line (`#`
  starts a comment), each waiting for the previous one's reply; each command and its reply are printed to the shell
  under `<script file name>:<line>`, and the first failure skips the rest. `init.xsa` is the repository's development
  setup: `mise run client -- --script init.xsa`.
- Commands are read from stdin with a `>` prompt (plain lines when stdin is
  not a terminal), and from `xsa ipc` over a socket in `$XDG_RUNTIME_DIR/xsa/`
  (named pipes on Windows).
- Commands received over IPC and their replies are printed to the shell, each pair under one `ipc #<sequence>` label;
  `logging.shell.ipc.enabled`, `.input.enabled` and `.output.enabled` choose which (`logging.shell.script.*` for
  startup scripts), and `logging.shell.enabled false` silences them with the log.

## Testing

### Launching

- Launch the game with `mise run client [--release] -- --name agent-<workspace>`. Always name the instance, so IPC
  can reach it.

#### Inside herdr

- Always start the game in a dedicated herdr pane with `herdr pane run`.
- Reuse an existing pane found with `herdr pane list` when there is one; otherwise create it with
  `herdr pane split --direction down --ratio 0.8 --no-focus`.
- The pane is named `xsa-agent` (`herdr pane rename <pane_id> xsa-agent` after creating it). Never use an existing
  pane that does not carry that name.
- If the game is already running in the pane, send it `exit` and start a fresh instance.

### IPC

- Always interact with the game through the `mise run ipc` task, never through stdin.
- Always pass `--name agent-<workspace>` so commands reach your own instance: other game instances may be running at
  the same time.

### Before testing

Always bring the game to a stable state first:

- A fresh instance starts on the main menu: enter the game with `ui push game`.
- Pause time with `time pause`.
- Set a specific time with `time set <datetime>`.
- Disable animations with `config set animation.duration-scale 0`.
- Disable player input, so it cannot interfere, with `config set input.enabled false`.
- Show the debug overlay with `ui show debug-overlay` when needed.

### Recording

- `camera tp --help` explains how to move the camera.
- Make the game window fullscreen before taking a snapshot, unless the test is about window sizing.
- Capture the game with `camera snap`:
  - `--output <path>` saves the snapshot to your scratchpad.
  - `--region <region>` captures only part of the screen; leave it out to capture the entire screen.
- When asked for a comparison, build a simple A/B comparison HTML page and open it automatically.

### After testing

- Inside herdr, leave the game running so I can try the implementation by hand; outside herdr, close it with `exit`.
- Re-enable player input with `config set input.enabled true`.

## Dependencies and assets

- Add a crate only when writing the code ourselves is unreasonable, and justify
  it first. Deliberately avoided: `ash-window` (surfaces are created by hand)
  and runtime shader compilers.
- Game content lives in `data/`. Text files are committed; binaries (`.dds`,
  `.spv`) are git-ignored and produced by tasks, except the base pack's
  textures: no task produces them, so they are committed, and only once final,
  never per iteration. A pack's `scripts/` holds the tools that produce its
  assets (`data/base/scripts/lens/` bakes the lens) and is left out of
  releases; scripts cache their source downloads in the pack's git-ignored
  `cache/`.
- A change to any file in `data/` needs a new version of each pack it touches:
  always remind the user to publish one.
- Every third-party asset is credited with source, author and terms: content a
  pack's scripts generate in that pack's `LICENSE`, which the scripts write,
  everything else in `LICENSES.md`.

## Config

- Each process has one config tree and one file, `config.kdl`: the game's is `$XDG_CONFIG_HOME/xsa/config.kdl`
  (falling back to `~/.config/xsa/config.kdl`), the dedicated server's `.xsa/server/config.kdl`; `--config <path>`
  picks another. The file in effect is logged at INFO when the process starts. Dotted keys are nested nodes:
  `video.bloom.enabled` is `video { bloom { enabled #false } }`. A repeated block is read last-wins.
- Keys are grouped by topic, never by owner: the game has its own sections (`video`, `input`, `terrain`,
  `animation`, `service`, `render`, `debug`) and, like every process, `logging`. No key is inherently saved or
  temporary.
- A value is its session value, else its saved value (the file in effect), else its default. `config set`, `toggle`
  and `unset` change the session value; with `--save` they change the saved value and write the file in effect
  instead, keeping its comments and layout. A file holds every key that was saved, defaults included; `unset`
  returns a key, or every key below it, to its default.
- `config get [<key>]` shows a value, or the tree below a node (the whole tree without a key) with each value marked
  session, saved or default; `--schema` shows each key's type (`<bool>`), whether it is optional and its default
  instead. `config save <path> [--key <key>]` exports the saved values; `config load [--save] [<path>]
  [--key <key>]` overlays a file onto the session (or saved) values, or reads the file in effect again without a
  path. Exported and loaded files are relative to `--key`.
- Config commands run in the process that receives them (`Route::Local`), through its `ConfigStore`.
- A section implements `ConfigSection`: one `visit` pairs each key with its field, which reads, writes, lists and
  edits it, so adding a key touches only its section. The root (`crates/client/src/config.rs`,
  `DedicatedConfig`) places each section in the tree; the game's defaults all live in `config/default_config.rs`.
- Binds are `input.bind.<category>.<action>` keys holding a key chord (`f4`,
  `shift+tab`, `` ` ``). Each action runs a typed action directly (the same
  command structs the text commands parse into), never command text.

## Controls

The game starts in the main menu. In game, right-drag orbits the target and
scroll zooms; the debug camera (`camera mode debug`) flies with WASD,
Space/Shift and mouse look (click to capture), and scroll sets its speed. The
camera gets input only while the game view controller is on top; menus release the
mouse. Debug toggles are commands (`config toggle video.stars`,
`config set render.shader.override normals`, …), not binds. The default binds:

| Key | Bind | Action |
| --- | --- | --- |
| Tab / Shift+Tab | `camera.target-next` / `target-previous` | Next / previous target |
| Escape | `interface.pause-menu` | Open the pause menu in game, close it in the pause menu |
| F3 | `interface.debug-overlay` | Show or hide the debug overlay (FPS, triangles) |
| . | `time.faster` | Step up from a pause through slow motion to real time, then double the time rate (`time faster`) |
| , | `time.slower` | Halve the time rate down to real time, then step through slow motion (0.5×, 0.25×, 0.1×) and pause (`time slower`) |
| / | `time.real-time` | Run time at real speed (`time resume`) |

## Gotchas

- Every queue submission goes through a `Recording` and every present
  through `Timeline::present`: the deletion queue only knows the GPU is
  done with an object from the timelines and present fences.
- Every upload and readback goes through `memory-vulkan`'s `TransferQueue`,
  never a staging buffer and a wait for the GPU. A copied resource is used only
  once its ticket completed, and every frame's submission waits on
  `TransferQueue::visibility_wait`, which makes those copies visible to it; the
  resources are created with `TransferQueue::sharing`. Data a frame rewrites and
  consumes itself (per-frame buffers, the interface's glyph atlas) stays in that
  frame.
- Every file content refers to loads through `Resources`, never on the thread
  that draws. Whoever uses a resource holds its `ResourceHandle` and derives what
  it needs (a material's parameters, a terrain layer's header, the lens flare)
  once the handle is ready; until then it draws its fallback.
- Validation layers (`vulkan-validation-layers`) are enabled in debug builds and
  logged under the `vulkan` target; any validation message is a bug. Messages mentioning
  `obs-vkcapture` come from that injected layer.
- `spirv-val` comes from the `spirv-tools` system package (no pinnable mise
  release).
- On Wayland a window appears only once a frame is presented: an invisible
  window means presentation is broken.
- Screenshots of the game window taken from outside the game are unreliable
  (Hyprland retiles it, it may be on another workspace). Use `camera snap`,
  which reads the presented image back from the GPU.
