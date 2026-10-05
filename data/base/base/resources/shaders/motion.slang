import common.scene;

static const float fullBrightnessPixels = 16.0;

[shader("vertex")]
VertexOutput vertexMain(VertexInput input)
{
    return transformVertex(input);
}

// Red and green are the horizontal and vertical speeds across the screen; black is still.
[shader("fragment")]
SurfaceOutput fragmentMain(VertexOutput input)
{
    float2 pixels = abs(motionVector(input.currentClip, input.previousClip) * constants.frame.viewportSize);
    return surfaceOutput(float4(saturate(pixels / fullBrightnessPixels), 0.0, 1.0), input);
}
