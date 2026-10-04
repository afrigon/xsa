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

## Input

- xui knows no windowing library. The app converts its windowing events into
  xui's `PointerEvent`s (moved, pressed, released, left, at a position in
  physical pixels) and calls `Interface::handle_event`, which returns whether
  a view used the event, so the app keeps it from what lies under the
  interface.
- Events are hit-tested against the layout of the last render: each node
  remembers where it was placed, and the event is offered from the topmost
  view down (later `ZStack` children first, children before their parent),
  telling lower views when a view above already took it.
- Interactive views keep their state in their node: a `Button` tracks hover
  and press there, and its style reads them on the next render. A button
  fires on a primary release inside it after a press that started in it;
  releasing outside cancels, as on iOS.
- `ButtonStyle` mirrors SwiftUI's: `body` builds on the label for the current
  `ButtonConfiguration` (`is_hovered`, `is_pressed`). `.button_style(style)`
  sets it through the environment for every button inside.

## Navigation

xui mirrors UIKit's controllers for full screens, with SwiftUI views inside
them. All of it is generic, for any app.

- **`ViewController`**, like `UIViewController`: an object that lives as long
  as it is in the stack and owns its fields.
  - `root()` returns the controller's content. By convention it is one named
    view (`MainMenuViewController::root()` returns `MainMenuView { … }`) whose
    `body` holds the layout; a controller never builds its layout inline.
  - `presentation()` is `FullScreen` (the controllers below are not drawn but
    keep their state) or `Overlay` (the controllers below keep drawing, as a
    pause menu over the game).
  - Lifecycle hooks, all optional: `did_load`/`did_unload` when it enters or
    leaves the stack, `did_appear`/`did_disappear` when it becomes visible or
    hidden.
- **`NavigationController`** holds one stack of controllers: `push`, `pop`
  and `set_root`. There is no separate modal presentation: what UIKit's
  `present` adds is a screen that leaves the one below visible, which is the
  `Overlay` presentation. The app owns the controller; `NavigationView`
  draws it, framing every screen to fill its container.
- **Transitions** are chosen per pair of controllers by a
  `NavigationDelegate`, like UIKit's `animationControllerFor:from:to:`: it
  gets the operation (push, pop, set root) and both controllers and returns a
  `Transition`. A transition has a duration and, for each moment, how the
  outgoing and incoming screens appear (opacity, offset); built-ins are
  `FadeTransition`, `SlideTransition` and `NoTransition`. The app sees each
  frame's progress too, so a transition can drive things outside the
  interface (the main menu to game zoom moves the 3D camera). Nothing is
  hit-testable while a transition runs. The app advances transitions with
  the frame's elapsed time.
- **Overlays that are not screens** (a debug overlay) are not in the stack:
  the app shows or hides them around its `NavigationView`, as SwiftUI's
  `.overlay`.
- **Signals out of the interface** go through the environment, as in
  SwiftUI's environment actions (`openURL`, `dismiss`): the app puts a typed
  actions handle in the environment, whose methods take arguments and build
  typed requests the app carries out.

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
- **Screens and navigation:**
  - Each screen is a `ViewController` in `ui/<feature>/`
    (`MainMenuViewController` with its root `MainMenuView`), addressed by a
    screen id in commands: `main-menu`, `config` (shown to players as
    Options), `load`, `game`, `pause-menu`.
  - `ui push <screen>`, `ui pop` and `ui set <screen>` change the stack;
    `ui show|hide|toggle <overlay>` change the overlays (`debug-overlay`).
    They are typed client commands: buttons send them through the `Actions`
    handle in the environment (`actions.push(Screen::Config)`), binds and the
    console and IPC run the same commands. `App` runs queued actions through
    the executor, never formatting text.
  - Back is per screen, mostly a BACK button popping. Escape pushes the pause
    menu in game and does nothing elsewhere; F3 toggles the debug overlay,
    hidden at start. There are no other interface binds; debug toggles are
    commands.
  - The pause menu is an `Overlay` controller over `game`. It pauses time
    while it is in the stack when the server is the integrated one; on a
    remote server time keeps running.
  - The camera gets input, and may capture the mouse, only while `game` is
    the visible top screen and no transition runs; leaving it releases the
    mouse.
  - The 3D world renders full-screen, owned by the game, and the interface
    is drawn on top. A view that contains 3D (a part preview, a minimap)
    would be a viewport primitive showing a render-target texture.
  - The main menu shows the simulation's spawn body large on the right, its
    buttons on the left. Time is frozen there, so continuing never jumps;
    the body turns with a cosmetic rotation (the simulation's rotation rate,
    sped up) eased away as CONTINUE zooms the camera into the game. The
    camera frames the body's lit side.
  - The config panel edits settings by building the typed config command
    objects and running them through the executor, never by formatting
    command text to parse.
- `App` offers pointer events to the interface first; what the interface
  uses never reaches the camera or the binds, and while the camera holds the
  mouse captured the interface sees nothing. Cursor moves are coalesced: the
  interface sees at most one move per frame, with the latest position, before
  it is updated; presses, releases and leaving the window are offered at
  once. Keyboard input stays with the binds until text fields exist.
- The renderer's `UserInterfacePass` runs after tonemapping and before
  capture, so `camera snap` includes the interface.
