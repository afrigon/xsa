// Must match `SurfaceTiling` in the renderer.
enum SurfaceTiling : uint {
    Hex = 0,
    Offset = 1,
}

[vk::constant_id(5)]
const uint surfaceTiling = 0;
