// Where the camera stands in a seeded noise, in the body's frame: its cell and the fraction across it, so a pixel adds
// only its small offset. Mirrors `NoiseAnchor` in the renderer.
struct NoiseAnchor {
    int3 cell;
    uint seed;
    float3 fraction;
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

// One of the twelve directions to a cube's edge midpoints, picked by the hash of a lattice point.
float3 latticeGradient(int3 cell, uint seed) {
    uint hash = pcg3d(uint3(cell) ^ uint3(seed, seed * 0x9E3779B9u, seed * 0x85EBCA6Bu)).x % 12u;
    float3 gradients[12] = {
        float3(1, 1, 0),
        float3(-1, 1, 0),
        float3(1, -1, 0),
        float3(-1, -1, 0),
        float3(1, 0, 1),
        float3(-1, 0, 1),
        float3(1, 0, -1),
        float3(-1, 0, -1),
        float3(0, 1, 1),
        float3(0, -1, 1),
        float3(0, 1, -1),
        float3(0, -1, -1),
    };
    return gradients[hash];
}

// Gradient noise (Perlin, 2002) with quintic fades, from about -1 to 1, at `fraction` across lattice cell `cell`.
float gradientNoise(int3 cell, float3 fraction, uint seed) {
    float3 fade = fraction * fraction * fraction * (fraction * (fraction * 6.0 - 15.0) + 10.0);
    float corners[8];

    for (int corner = 0; corner < 8; corner++) {
        int3 step = int3(corner & 1, (corner >> 1) & 1, corner >> 2);
        corners[corner] = dot(latticeGradient(cell + step, seed), fraction - float3(step));
    }

    float4 alongX = lerp(float4(corners[0], corners[2], corners[4], corners[6]),
                         float4(corners[1], corners[3], corners[5], corners[7]),
                         fade.x);
    float2 alongY = lerp(alongX.xz, alongX.yw, fade.y);
    return lerp(alongY.x, alongY.y, fade.z);
}

// The noise `offset` (m) away from where its anchor stands.
float noiseAt(NoiseAnchor anchor, float3 offset) {
    float3 place = anchor.fraction + offset * anchor.inverseSize;
    float3 whole = floor(place);
    return gradientNoise(anchor.cell + int3(whole), place - whole, anchor.seed);
}
