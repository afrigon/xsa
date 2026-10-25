import draw;

// Where the camera stands in a seeded noise, in the body's frame: its place in the noise volume (periods), and the
// volume's periods per meter, so a pixel adds only its small offset. Mirrored by `NoiseAnchor` in the renderer.
struct NoiseAnchor {
    float3 place;
    float inverseSize;
};

// PCG3D (Jarzynski and Olano, 2020): three well-mixed words from three.
uint3 pcg3d(uint3 value) {
    value = value * 1664525u + 1013904223u;
    value.x += value.y * value.z;
    value.y += value.z * value.x;
    value.z += value.x * value.y;
    value ^= value >> 16u;
    value.x += value.y * value.z;
    value.y += value.z * value.x;
    value.z += value.x * value.y;
    return value;
}

// The offset's dot product with one of the directions to a cube's edge midpoints, picked from the bits of a lattice
// point's hash (Perlin, "Improving Noise", 2002) rather than from a table, which a GPU would keep in slow memory.
float latticeDot(int3 cell, uint seed, float3 offset) {
    uint hash = pcg3d(uint3(cell) ^ uint3(seed, seed * 0x9E3779B9u, seed * 0x85EBCA6Bu)).x & 15u;
    float first = hash < 8u ? offset.x : offset.y;
    float second = hash < 4u ? offset.y : (hash == 12u || hash == 14u ? offset.x : offset.z);
    return ((hash & 1u) == 0u ? first : -first) + ((hash & 2u) == 0u ? second : -second);
}

// Gradient noise (Perlin, 2002) with quintic fades, from about -1 to 1, at `fraction` across lattice cell `cell`.
float gradientNoise(int3 cell, float3 fraction, uint seed) {
    float3 fade = fraction * fraction * fraction * (fraction * (fraction * 6.0 - 15.0) + 10.0);
    float4 near = float4(latticeDot(cell, seed, fraction),
                         latticeDot(cell + int3(0, 1, 0), seed, fraction - float3(0, 1, 0)),
                         latticeDot(cell + int3(0, 0, 1), seed, fraction - float3(0, 0, 1)),
                         latticeDot(cell + int3(0, 1, 1), seed, fraction - float3(0, 1, 1)));
    float4 far = float4(latticeDot(cell + int3(1, 0, 0), seed, fraction - float3(1, 0, 0)),
                        latticeDot(cell + int3(1, 1, 0), seed, fraction - float3(1, 1, 0)),
                        latticeDot(cell + int3(1, 0, 1), seed, fraction - float3(1, 0, 1)),
                        latticeDot(cell + int3(1, 1, 1), seed, fraction - float3(1, 1, 1)));
    float4 alongX = lerp(near, far, fade.x);
    float2 alongY = lerp(alongX.xz, alongX.yw, fade.y);
    return lerp(alongY.x, alongY.y, fade.z);
}

// The noise `offset` (m) away from where its anchor stands, read from the noise volume at bindless index `volume`.
float noiseAt(uint volume, NoiseAnchor anchor, float3 offset) {
    return volumes[volume].SampleLevel(tilingSampler, anchor.place + offset * anchor.inverseSize, 0.0).r;
}
