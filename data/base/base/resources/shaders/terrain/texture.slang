import draw;

// Where a pixel samples a material's maps, with the screen-space derivatives that pick the mip level.
struct TextureCoordinates {
    float2 uv;
    float2 uvPerPixelX;
    float2 uvPerPixelY;
};

float smallerChange(float wrappedAtZero, float wrappedAtHalf) {
    return abs(wrappedAtZero) <= abs(wrappedAtHalf) ? wrappedAtZero : wrappedAtHalf;
}

// Equirectangular maps put longitude 0 at u = 0.25 with u growing westward, and the south pole at v = 0.
TextureCoordinates equirectangularCoordinates(float3 direction) {
    float longitude = atan2(direction.y, direction.x);
    float latitude = asin(clamp(direction.z, -1.0, 1.0));
    float u = 0.25 - longitude / (2.0 * pi);
    float v = 0.5 + latitude / pi;
    // u jumps by 1 where it wraps; a copy wrapping half a turn away is continuous there, so each screen axis
    // takes whichever derivative is smaller to keep mip selection free of a seam line.
    float wrappedAtZero = frac(u);
    float wrappedAtHalf = frac(u + 0.5) - 0.5;
    TextureCoordinates coordinates;
    coordinates.uv = float2(wrappedAtZero, v);
    coordinates.uvPerPixelX = float2(smallerChange(ddx(wrappedAtZero), ddx(wrappedAtHalf)), ddx(v));
    coordinates.uvPerPixelY = float2(smallerChange(ddy(wrappedAtZero), ddy(wrappedAtHalf)), ddy(v));
    return coordinates;
}

float4 sampleTexture(uint texture, TextureCoordinates coordinates) {
    return textures[texture].SampleGrad(
        wrappingSampler, coordinates.uv, coordinates.uvPerPixelX, coordinates.uvPerPixelY);
}

float3 mappedNormal(uint normalTexture, float3 modelNormal, TextureCoordinates coordinates) {
    if (normalTexture == noTexture) {
        return modelNormal;
    }

    float2 tangent = sampleTexture(normalTexture, coordinates).rg * 2.0 - 1.0;
    float outward = sqrt(saturate(1.0 - dot(tangent, tangent)));
    float3 east = normalize(float3(-modelNormal.y, modelNormal.x, 0.0) + float3(1e-7, 0.0, 0.0));
    float3 north = cross(modelNormal, east);
    return normalize(-east * tangent.x + north * tangent.y + modelNormal * outward);
}

float3 mappedColor(uint colorTexture, float3 baseColor, TextureCoordinates coordinates) {
    if (colorTexture == noTexture) {
        return baseColor;
    }

    return sampleTexture(colorTexture, coordinates).rgb;
}

float3 mappedEmission(uint emissiveTexture, float emissiveLuminance, TextureCoordinates coordinates) {
    if (emissiveTexture == noTexture) {
        return float3(0.0);
    }

    return sampleTexture(emissiveTexture, coordinates).rgb * emissiveLuminance;
}
