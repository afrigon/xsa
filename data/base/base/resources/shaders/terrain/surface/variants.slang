// How quickly one variant gives way to the next across the variant noise: most ground shows a single variant, and
// height blending shapes the borders between them.
static const float variantSharpness = 4.0;

// The two neighbouring variants a noise value (about -1 to 1) picks among `count`, and the second one's weight.
struct VariantPair {
    uint first;
    uint second;
    float secondWeight;
};

VariantPair variantPair(float noise, uint count) {
    float place = (noise * 0.5 + 0.5) * float(count) - 0.5;
    float first = clamp(floor(place), 0.0, float(count - 1));
    VariantPair pair;
    pair.first = uint(first);
    pair.second = min(pair.first + 1, count - 1);
    pair.secondWeight = pair.first == pair.second ? 0.0 : saturate((place - first - 0.5) * variantSharpness + 0.5);
    return pair;
}
