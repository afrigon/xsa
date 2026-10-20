// Must match `ShadingRange` in the renderer.
enum ShadingRange : uint {
    Far = 0,
    Near = 1,
}

[vk::constant_id(3)]
const uint shadingRange = 0;

bool shadesSurfaces() {
    return shadingRange == uint(ShadingRange.Near);
}
