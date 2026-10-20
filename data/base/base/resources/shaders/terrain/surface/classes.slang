import terrain.surface.surface_data;

// The slope (radians) that picks a pixel's class: the terrain's own slope, moved by the seeded handover noise and by
// the ground's shape, convex ground leaning steep and concave or low ground leaning flat.
float classSlope(BiomeData biome, float3 normal, float3 up, float curvature, float relativeHeight, float noise) {
    float slope = acos(saturate(dot(normal, up)));
    slope += biome.handoverNoiseAmplitude * noise;
    slope += biome.convexBias * max(-curvature, 0.0) - biome.concaveBias * max(curvature, 0.0);
    return slope + biome.reliefBias * relativeHeight;
}

// How much of flat, slope and steep ground a slope is: smooth ramps around each handover, which never overlap.
float3 classWeights(BiomeData biome, float slope) {
    float toSlope = smoothstep(biome.flatToSlopeAngle - 0.5 * biome.flatToSlopeWidth,
                               biome.flatToSlopeAngle + 0.5 * biome.flatToSlopeWidth,
                               slope);
    float toSteep = smoothstep(biome.slopeToSteepAngle - 0.5 * biome.slopeToSteepWidth,
                               biome.slopeToSteepAngle + 0.5 * biome.slopeToSteepWidth,
                               slope);
    return float3(1.0 - toSlope, toSlope - toSteep, toSteep);
}
