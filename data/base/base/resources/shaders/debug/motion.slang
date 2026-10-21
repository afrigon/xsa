import fragment;

static const float fullBrightnessPixels = 16.0;

// Red and green are the horizontal and vertical speeds across the screen; black is still.
float3 motionColor(DrawVaryings input) {
    float2 pixels = abs(motionVector(input.currentClip, input.previousClip) * constants.frame.viewportSize);
    return float3(saturate(pixels / fullBrightnessPixels), 0.0);
}
