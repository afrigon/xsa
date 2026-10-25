import draw;

float starCosine(StarData star, float3 worldPosition, float3 normal) {
    float3 toStar = normalize(star.positionAndRadius.xyz - worldPosition);
    return max(dot(normal, toStar), 0.0);
}

// Lux per color channel: the star's luminous intensity falls off with the square of the distance.
float3 starIlluminance(StarData star, float3 worldPosition) {
    float3 toStar = star.positionAndRadius.xyz - worldPosition;
    return star.intensity.rgb / dot(toStar, toStar);
}

// cd/m² reflected from the faint starlight arriving from every direction.
float3 starlightLuminance(float3 albedo) {
    return albedo / pi * constants.frame.starlightIlluminance;
}

// The cosine with a mapped normal, zero where the geometric surface faces away from the star: a normal map never
// lights the night side.
float mappedCosine(StarData star, float3 worldPosition, float3 geometricNormal, float3 normal) {
    float facingStar = starCosine(star, worldPosition, geometricNormal) > 0.0 ? 1.0 : 0.0;
    return starCosine(star, worldPosition, normal) * facingStar;
}
