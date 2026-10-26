__exported import debug.view;
__exported import mesh.vertex;

// A 16×16 Bayer matrix: 256 levels, so an instance first drops single pixels 16 apart, then fills in between them.
static const uint bayerLevels = 4;
static const uint bayerSize = 1 << bayerLevels;

// The matrix's value at a pixel: the bits of x xor y and of y interleaved, the lowest coordinate bits highest.
uint bayerValue(uint2 pixel) {
    uint crossed = pixel.x ^ pixel.y;
    uint value = 0;

    for (uint level = 0; level < bayerLevels; level++) {
        uint pair = (((crossed >> level) & 1) << 1) | ((pixel.y >> level) & 1);
        value |= pair << (2 * (bayerLevels - 1 - level));
    }

    return value;
}

// A cell of the pattern spans one pixel per this many rows of the output, so the fade looks alike at every resolution.
static const float ditherReferenceRows = 360.0;

// Fading instances drop the cells whose threshold their visibility falls under. The pattern is fixed to the screen's
// pixels, ignoring the antialiasing jitter.
void ditherVisibility(MeshVaryings input) {
    uint cellPixels = max(uint(round(constants.frame.viewportSize.y / ditherReferenceRows)), 1u);
    uint2 pixel = uint2(input.draw.position.xy) / cellPixels % bayerSize;
    float dithered = (float(bayerValue(pixel)) + 0.5) / float(bayerSize * bayerSize);
    float threshold = constants.frame.fadeEnabled != 0 ? dithered : 0.5;

    if (input.visibility <= threshold) {
        discard;
    }
}

// What a mesh program writes: the view an override selects, or the luminance it shaded.
FragmentOutput meshOutput(float3 luminance, MaterialSample sample, MeshVaryings input, uint primitive) {
    if (showsDebugView()) {
        return fragmentOutput(float4(debugViewColor(input.draw, sample, primitive), 1.0), input.draw);
    }

    return fragmentOutput(float4(luminance * constants.frame.exposure, 1.0), input.draw);
}
