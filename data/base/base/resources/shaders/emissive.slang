import common.scene;

struct EmissiveData {
    float3 luminance;
    uint padding;
};

[shader("fragment")]
SurfaceOutput fragmentMain(VertexOutput input) {
    return surfaceOutput(float4(currentMaterial<EmissiveData>().luminance * constants.frame.exposure, 1.0), input);
}
