import terrain.layers;

// Surfaces fade in over this many levels as the colour layer is magnified past its finest texels, and out over as
// many mip levels before a pixel covers a whole repeat of their texture.
static const float surfaceFadeLevels = 2.0;

float surfaceFade(LayerCoordinates coordinates) {
    float finest = float(finestLevel(coordinates.tile, colorLayer));
    return saturate((coordinates.pixelLevel - finest) / surfaceFadeLevels);
}

// How much of a surface repeating every `size` meters still shows in a pixel covering `footprint` meters: none once the
// pixel covers a whole repeat, where the texture is its own average. Mirrored by `BiomeSurfaces::shows_at`.
float surfaceFadeOut(float size, float footprint) {
    return saturate(log2(size / footprint) / surfaceFadeLevels);
}
