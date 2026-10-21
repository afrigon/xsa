import terrain.layers;

// Surfaces fade in over this many levels as the colour layer is magnified past its finest texels.
static const float surfaceFadeLevels = 2.0;

float surfaceFade(LayerCoordinates coordinates) {
    float finest = float(finestLevel(coordinates.tile, colorLayer));
    return saturate((coordinates.pixelLevel - finest) / surfaceFadeLevels);
}
