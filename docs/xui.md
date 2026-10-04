# xui

xui is a UI framework in the image of SwiftUI, written in Rust. The goal is to
stay as close to SwiftUI as Rust allows: the same concepts, the same layout
behavior, and code that reads like SwiftUI. Where Rust cannot express a SwiftUI
feature directly, xui picks the closest idiomatic equivalent and records the
difference here.

## Place in the project

- **Incubated in xsa, destined for its own repository.** `crates/xui` and
  `crates/xui-vulkan` depend on no xsa crate, and xsa uses them like any
  external library. They move to their own repository (`xui-rs`, same crate
  names) once the API is stable enough that most xsa UI work no longer changes
  xui, or a second project wants it. Before that move, xui needs its own error
  type instead of `anyhow`, and a test font of its own instead of reading
  Inter from xsa's `data/`.
- **Three layers, like SwiftUI, Core Animation and a design system:**
  - `xui` holds views, layout, environment, state, text shaping and the glyph
    atlas. It is renderer-agnostic: its output is a draw list of primitives in
    physical pixels plus the atlas regions that changed.
  - `xui-vulkan` is the Vulkan backend that records a draw list into a host's
    command buffer.
  - The design system lives in the app. xsa's theme comes from packs
    (`docs/packs.md`, inspired by stylx) and reaches views through the
    environment and modifiers such as `.text_style("title")`.

## Views

- **Values that own their data.** A view is a `'static` struct rebuilt every
  frame. It never borrows. Game data reaches views in three ways:
  - small values copied in (numbers, names);
  - shared handles (`Rc`/`Arc`) for large data;
  - environment values for data many views read, like the theme.
- **`body(&self, context: &Context) -> impl View`.** `Context` gives the
  environment, and will give state. Primitive views (`Text`, stacks, frames,
  padding) have a `Never` body, as in SwiftUI, and implement `update`
  instead, keeping what their layout needs in a `NodeLayout`.
- **Composition without exceptions.** Tuples (up to 12), `Option` (an `if`),
  `Either` (an `if/else` with different types), `Vec`, `ForEach`, `Group`,
  `EmptyView` and `AnyView` are all views usable anywhere. Stacks flatten
  their subviews, so an absent item takes neither space nor spacing.
- **Loops:** `ForEach::new(items, |item| view)` for `Identifiable` items, and
  `ForEach::with_id(items, |item| id, |item| view)` for other types.
- **No macro DSL.** Plain Rust (tuples, `Option`, `Either`, `ForEach`)
  replaces result builders. A `view_builder!` macro accepting literal
  `if`/`for` may come later; it would expand to these same types.

## Layout

- SwiftUI's rule: a parent proposes a size, the child chooses its size, and
  the parent places it, in logical points.
- `VStack`, `HStack` and `ZStack` follow SwiftUI's stack algorithm: the least
  flexible children are sized first, and `Spacer` expands along its stack's
  axis. The default spacing is 8 points.
- `.frame(width, height, alignment)` is a fixed frame. `.max_frame(max_width,
  max_height, alignment)` is a flexible one, where `f32::INFINITY` fills the
  proposed space. `.padding(insets)` insets the content.
- The root is centered in the viewport, and content larger than its container
  overflows, both as in SwiftUI. `ScrollView` and clipping are the answer to
  overflow.

## Environment and modifiers

- Typed keys: `impl EnvironmentKey for MyKey`, read with
  `environment.get::<MyKey>()`, set for a subtree with
  `.environment::<MyKey>(value)`.
- `font` and `foreground_style` cascade through the environment, as in
  SwiftUI. A background is a drawing modifier, not an environment value.
- `ViewModifier::body(&self, content: ModifierContent, context)` mirrors
  SwiftUI's `ViewModifier`. `ModifierContent` stands for the wrapped view.
- Nothing missing is fatal. An unknown text style keeps the inherited font,
  and text without a font draws nothing. Each logs one warning, not one per
  frame.

## Retained tree

- Views are matched every frame with last frame's nodes. Identity is the
  view's type and position (structural identity), or an explicit key:
  `ForEach` ids, or `.id(value)`. Switching `if/else` branches of different
  types creates a new node, as in SwiftUI. A `Vec` of views is matched by
  index, so lists that change belong in a `ForEach`.
- Each frame runs two passes:
  - **Update:** every `body` re-runs. Leaves compare their inputs with the
    ones stored in their node: a `Text` compares its string, font and scale
    factor, and shapes again only when they changed. A change marks the
    node and its ancestors, which drop their cached sizes.
  - **Layout:** runs over the nodes. Unchanged nodes answer measurements from
    their cache, so only the path from a changed leaf to the root
    recomputes.
- Upgrade path: skip the bodies whose inputs did not change, as SwiftUI's
  attribute graph does, once re-running every body shows up in profiles.

## State

- `context.state(|| initial)` returns a `State<T>` stored in the view's node,
  keyed by call site (`#[track_caller]`), so an `if` around one call does
  not shift the others.
- `state.binding()` gives a `Binding<T>`, a cheap cloneable handle that a
  child view stores to read and write the parent's state, like `$value`.
  Bindings to app data are built from get and set functions.
- State changes need no invalidation of their own: every body re-runs each
  frame and reads the new value, and the leaves see their inputs change.

## Navigation (planned)

- Navigation is state, as in SwiftUI: a value says which screen shows, and
  xui provides the mechanisms to react to it, never the screens.
- `NavigationStack` mirrors SwiftUI's: a root view, a path of routes bound
  through a `Binding`, and a function building the view for each route.
  Back pops the path.
- The path can live in the app rather than in xui state. Its binding's `set`
  then decides what changing it means, which lets an app route navigation
  through its own actions.

## Text

- parley shapes text and swash rasterizes glyphs, from fonts registered as
  bytes, with no system fonts.
- A `Font` carries family, size, weight, letter spacing and OpenType
  features (`tnum`, `zero`). The optical-size axis (`opsz`) follows the font
  size.
- Glyphs go into one 1024² single-channel atlas, rasterized on first use at
  quarter-pixel horizontal offsets. When the atlas is full, rendering fails
  with an error rather than evicting.

## xui-vulkan

- A binding-agnostic contract. The host passes raw handles as `u64` and
  `vkGetInstanceProcAddr`. `ash` is a private implementation detail.
  Requirements: Vulkan 1.3 with dynamic rendering and synchronization2.
- It owns its memory (a few raw allocations), its pipeline, its own
  descriptor set (the atlas and a storage buffer of instances) and its
  shader, compiled by `mise run shaders` and embedded in the crate.
- It records into the host's command buffer over an image in
  color-attachment layout and leaves it there, blending with premultiplied
  alpha. Atlas updates are copied in the same command buffer before drawing.

## In xsa

- Interface code lives in `crates/client/src/ui/`, one folder per feature
  (`debug/`, `hud/`, …). A view in `ui/<feature>/<name>.rs` is named
  `<Feature><Name>View` (`DebugOverlayView`, `HudTargetView`).
- `App` builds one root environment per frame (scale factor, color scheme,
  theme, default foreground) and renders a single root view.
- **Scenes and navigation:**
  - xsa owns the navigation state: a top-level scene (main menu, in game)
    plus a `NavigationStack` path for menus, so Config opens from both the
    main menu and an in-game pause menu, and Escape or Back pops.
  - Navigating is an action like any other: a typed command dispatched
    through the command executor, whether a `Button`, a key bind, the
    console or IPC triggered it. A `NavigationStack` binding's `set`
    dispatches that command instead of writing the state directly.
  - The 3D world renders full-screen, owned by the game, and the interface
    is an overlay drawn on top. A view that contains 3D (a part preview, a
    minimap) would be a viewport primitive showing a render-target texture.
  - The main menu is itself a simulation, its 3D scene showing behind the
    menu. Play loads the save, eventually while the camera zooms into the
    starting area.
  - The config panel edits settings by building the typed config command
    objects and running them through the executor, never by formatting
    command text to parse.
- The renderer's `UserInterfacePass` runs after tonemapping and before
  capture, so `camera snap` includes the interface. A snap without it is
  `ui hide`, `camera snap`, `ui show`.
