import frame;

// Must match ProjectedLightData in client/src/renderer/gpu_data/projected_light_data.rs.
struct ProjectedLightData {
    float2 screenPosition;
    // Fraction of the light's disk no body hides.
    float visibility;
    float angularRadius;
    float3 illuminance;
    // Fraction of the light the lens passes at this angle off its axis.
    float acceptance;
};

// Must match GhostData in client/src/renderer/gpu_data/ghost_data.rs.
struct GhostData {
    float position;
    // Angular diameter, in radians.
    float size;
    uint texture;
    // Red moves outward along the axis and blue inward, each by this fraction of the ghost's size per radian the light
    // sits off the view axis.
    float dispersionPerRadian;
    float3 tint;
    // Multiplies the texture.
    float intensity;
};

// Must match LensPushConstants in client/src/renderer/gpu_data/lens_push_constants.rs.
struct LensConstants {
    FrameData *frame;
    ProjectedLightData *lights;
    GhostData *ghosts;
    float glareIntensity;
    float glareFalloff;
    float focalLengthPixels;
    uint lightCount;
    uint starburstTexture;
    float starburstInner;
    float starburstExtent;
    uint ghostCount;
};

// Direction of a pixel's ray, in a frame where the view axis is +Z.
float3 pixelRay(float2 pixel, float2 viewportSize, float focalLengthPixels) {
    return normalize(float3(pixel - viewportSize * 0.5, focalLengthPixels));
}
