import terrain.surface.surface_data;

// The slope (radians) that picks a pixel's class: the terrain's own slope, moved by the seeded handover noise and by
// the ground's shape, convex ground leaning toward slopes and concave or low ground leaning flat.
float classSlope(BiomeData biome, float3 normal, float3 up, float curvature, float relativeHeight, float noise) {
    float slope = acos(saturate(dot(normal, up)));
    slope += biome.handoverNoiseAmplitude * noise;
    slope += biome.convexBias * max(-curvature, 0.0) - biome.concaveBias * max(curvature, 0.0);
    return slope + biome.reliefBias * relativeHeight;
}

// How much of flat and sloped ground a slope is: a smooth ramp around the handover.
float2 classWeights(BiomeData biome, float slope) {
    float toSlope = smoothstep(biome.transitionAngle - 0.5 * biome.transitionWidth,
                               biome.transitionAngle + 0.5 * biome.transitionWidth,
                               slope);
    return float2(1.0 - toSlope, toSlope);
}
