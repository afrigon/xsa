# xsa

A space flight simulator in the spirit of Kerbal Space Program and Kitten Space
Agency, written in Rust on Vulkan.

## Priorities

1. **Accuracy.** Physically correct orbital mechanics and flight.
2. **Visual quality.** The target is AAA: PBR, atmospheric scattering,
   volumetric clouds, ray-marched engine plumes and explosions.
3. **Performance.** Nothing above may cost a smooth frame rate.

## Platforms

Linux (Wayland) is the primary and tested platform. Windows must build, but
X11 and Windows are not a testing focus — don't spend effort on them unless
asked.

## Dependencies

Add a crate only when writing the code ourselves is unreasonable. Before
adding one, justify it. Deliberately avoided:

- `ash-window` — surfaces are created by hand in `src/vulkan/surface.rs`.
- `shaderc` / runtime shader compilers — shaders are compiled offline.

## Graphics

- **Vulkan 1.3 baseline.** The released `ash` (0.38) has no 1.4 bindings; use
  1.4 features through their extensions.
- **Dynamic rendering and synchronization2** — no `VkRenderPass` objects, no
  legacy barriers.
- **Rendering architecture:** clustered forward (Forward+) with a depth
  prepass, HDR render targets and a tonemapping pass. Volumetric effects
  (atmosphere via Hillaire's LUT technique, clouds, plumes, explosions) are
  ray-marched passes that read depth, cleaned up by temporal anti-aliasing and
  upscaling.
- **Keep the pipeline count small** — bindless resources and dynamic state
  over per-material pipelines, and a `VkPipelineCache` persisted to disk.
- **Reverse-Z depth** to handle cockpit-to-planet depth ranges.

## Shaders

Written in Slang, compiled offline to SPIR-V with `slangc` (pinned in
`mise.toml`). The binary never embeds a shader compiler.

## Precision

The simulation runs in `f64`. Rendering is `f32`, relative to the camera, so
solar-system distances never reach the GPU. Math uses `glam`: `DVec3`/`DQuat`
for simulation, `Vec3`/`Mat4` for GPU data.

## Architecture

- **Simulation state is plain data, separate from rendering, advanced at a
  fixed timestep.** Saves, debug-mode editing and time warp all depend on it.
- **One scene at every scale.** Rendering must handle 1 m to interplanetary
  distances in a single frame; there is no separate map scene.

## Vulkan object lifetime

- Objects created once — `Instance`, `Surface`, `Device` — are destroyed by
  `Drop`, ordered by `Renderer`'s field declaration order.
- Everything created from the device has an explicit `unsafe fn destroy`,
  called after the GPU is idle. This is the shape a deletion queue needs:
  resources are freed only once the frames that used them have finished,
  which streaming terrain and textures will depend on.

## Development

- Tools (`rust`, `slang`) come from mise.
- Validation layers are a system package (`vulkan-validation-layers`), enabled
  automatically in debug builds; their messages print to stderr. Treat any
  validation message as a bug.
- On Wayland a window only appears once a frame has been presented — an
  invisible window means presentation is broken, not window creation.
