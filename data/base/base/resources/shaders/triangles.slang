import common.scene;

// PCG integer hash: unlike a plain multiply, it sends 0 to a well-mixed value rather than to black.
uint pcgHash(uint value) {
    uint state = value * 747796405u + 2891336453u;
    uint word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

[shader("fragment")]
SurfaceOutput fragmentMain(VertexOutput input, uint primitive: SV_PrimitiveID) {
    uint hash = pcgHash(primitive);
    float3 color = float3(hash & 0xff, (hash >> 8) & 0xff, (hash >> 16) & 0xff) / 255.0;
    return surfaceOutput(float4(color, 1.0), input);
}
