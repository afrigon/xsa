import common.hapke;
import common.scene;

[[vk::binding(2, 0)]]
SamplerState wrappingSampler;

[[vk::binding(3, 0)]]
Texture2D textures[];

struct HapkeData {
    uint scatterTexture;
    uint surgeTexture;
    float porosity;
    float roughness;
    float blend;
    float lightBoost;
    float gammaBoost;
    uint padding;
};

struct PlanetData {
    float3 baseColor;
    uint colorTexture;
    uint normalTexture;
    uint emissiveTexture;
    float emissiveLuminance;
    uint padding;
    HapkeData hapke;
};

// Must match ShadingModel::shader_id in client/src/renderer/material.rs.
static const uint shadingHapkeSol = 0;
static const uint shadingHapkeTextbook = 1;

[shader("vertex")]
VertexOutput vertexMain(VertexInput input) {
    return transformVertex(input);
}

struct SurfaceCoordinates {
    float2 uv;
    float2 uvPerPixelX;
    float2 uvPerPixelY;
};

float smallerChange(float wrappedAtZero, float wrappedAtHalf) {
    return abs(wrappedAtZero) <= abs(wrappedAtHalf) ? wrappedAtZero : wrappedAtHalf;
}

// Equirectangular maps put longitude 0 at u = 0.25 with u growing westward, and the south pole at v = 0.
SurfaceCoordinates equirectangularCoordinates(float3 direction) {
    float longitude = atan2(direction.y, direction.x);
    float latitude = asin(clamp(direction.z, -1.0, 1.0));
    float u = 0.25 - longitude / (2.0 * pi);
    float v = 0.5 + latitude / pi;
    // u jumps by 1 where it wraps; a copy wrapping half a turn away is continuous there, so each screen axis
    // takes whichever derivative is smaller to keep mip selection free of a seam line.
    float wrappedAtZero = frac(u);
    float wrappedAtHalf = frac(u + 0.5) - 0.5;
    SurfaceCoordinates coordinates;
    coordinates.uv = float2(wrappedAtZero, v);
    coordinates.uvPerPixelX = float2(smallerChange(ddx(wrappedAtZero), ddx(wrappedAtHalf)), ddx(v));
    coordinates.uvPerPixelY = float2(smallerChange(ddy(wrappedAtZero), ddy(wrappedAtHalf)), ddy(v));
    return coordinates;
}

float4 sampleSurface(uint texture, SurfaceCoordinates coordinates) {
    return textures[texture].SampleGrad(
        wrappingSampler, coordinates.uv, coordinates.uvPerPixelX, coordinates.uvPerPixelY);
}

float3 surfaceNormal(PlanetData material, float3 modelNormal, SurfaceCoordinates coordinates) {
    if (material.normalTexture == noTexture) {
        return modelNormal;
    }
    float2 tangent = sampleSurface(material.normalTexture, coordinates).rg * 2.0 - 1.0;
    float outward = sqrt(saturate(1.0 - dot(tangent, tangent)));
    float3 east = normalize(float3(-modelNormal.y, modelNormal.x, 0.0) + float3(1e-7, 0.0, 0.0));
    float3 north = cross(modelNormal, east);
    return normalize(-east * tangent.x + north * tangent.y + modelNormal * outward);
}

float4 sampleSurfaceCoarser(uint texture, SurfaceCoordinates coordinates, float scale) {
    return textures[texture].SampleGrad(
        wrappingSampler, coordinates.uv, coordinates.uvPerPixelX * scale, coordinates.uvPerPixelY * scale);
}

HapkeSurface hapkeSurface(PlanetData material, SurfaceCoordinates coordinates, float3 albedo) {
    float4 scatter = sampleSurface(material.hapke.scatterTexture, coordinates);
    float4 surge = sampleSurface(material.hapke.surgeTexture, coordinates);
    HapkeSurface surface;
    surface.albedo = albedo;
    surface.singleScatteringAlbedo = scatter.r;
    surface.lobeWidth = scatter.g;
    surface.lobeBalance = scatter.b;
    surface.shadowHidingAmplitude = surge.r;
    surface.shadowHidingWidth = surge.g;
    surface.backscatterAmplitude = surge.b;
    surface.backscatterWidth = surge.a;
    surface.porosity = material.hapke.porosity;
    surface.roughness = material.hapke.roughness;
    return surface;
}

float3 surfaceColor(PlanetData material, SurfaceCoordinates coordinates) {
    if (material.colorTexture == noTexture) {
        return material.baseColor;
    }

    return sampleSurface(material.colorTexture, coordinates).rgb;
}

// The colour map's average at the scatter map's resolution.
float3 regionalColor(PlanetData material, SurfaceCoordinates coordinates) {
    if (material.colorTexture == noTexture) {
        return material.baseColor;
    }

    uint colorWidth, colorHeight, scatterWidth, scatterHeight;
    textures[material.colorTexture].GetDimensions(colorWidth, colorHeight);
    textures[material.hapke.scatterTexture].GetDimensions(scatterWidth, scatterHeight);
    float scale = max(float(colorWidth) / float(scatterWidth), 1.0);
    return sampleSurfaceCoarser(material.colorTexture, coordinates, scale).rgb;
}

// The scatter map gives the regional single-scattering albedo at a fraction of the colour map's resolution; the
// colour map adds colour and fine detail as its ratio to its own average at the scatter map's resolution.
float3 textbookAlbedo(PlanetData material, SurfaceCoordinates coordinates, float3 color, float singleScatteringAlbedo) {
    float3 regional = regionalColor(material, coordinates);
    float regionalLuminance = max(dot(regional, float3(0.2126, 0.7152, 0.0722)), 1e-4);
    return singleScatteringAlbedo * color / regionalLuminance;
}

// cd/m² the surface reflects from one star.
float3 reflectedLuminance(StarData star, PlanetData material, SurfaceCoordinates coordinates, VertexOutput input,
                          float3 worldNormal, float3 albedo) {
    float facingStar = starCosine(star, input.worldPosition, normalize(input.normal)) > 0.0 ? 1.0 : 0.0;
    float cosine = starCosine(star, input.worldPosition, worldNormal) * facingStar;
    float3 illuminance = starIlluminance(star, input.worldPosition);
    uint shading = constants.frame.shadingModel;
    bool hapke =
        material.hapke.scatterTexture != noTexture && (shading == shadingHapkeSol || shading == shadingHapkeTextbook);

    if (!hapke || cosine <= 0.0) {
        return lambertLuminance(albedo, illuminance, cosine);
    }

    float3 toCamera = normalize(-input.worldPosition);
    float3 toStar = normalize(star.positionAndRadius.xyz - input.worldPosition);
    HapkeGeometry geometry;
    geometry.incidenceCosine = clamp(dot(toStar, worldNormal), 1e-4, 1.0);
    geometry.emissionCosine = clamp(dot(toCamera, worldNormal), 1e-4, 1.0);
    geometry.phaseAngle = acos(clamp(dot(toStar, toCamera), -1.0, 1.0));
    HapkeSurface surface = hapkeSurface(material, coordinates, albedo);

    if (shading == shadingHapkeSol) {
        return hapkeSol(surface, geometry, material.hapke.blend, material.hapke.lightBoost, material.hapke.gammaBoost) *
               illuminance;
    }

    surface.albedo = textbookAlbedo(material, coordinates, albedo, surface.singleScatteringAlbedo);
    return hapkeTextbook(surface, geometry) * illuminance;
}

[shader("fragment")]
SurfaceOutput fragmentMain(VertexOutput input) {
    PlanetData material = currentMaterial<PlanetData>();
    float3 modelNormal = normalize(input.modelNormal);
    SurfaceCoordinates coordinates = equirectangularCoordinates(modelNormal);

    float3 normal = surfaceNormal(material, modelNormal, coordinates);
    float3 worldNormal = normalize(mul(currentObject().worldFromModel, float4(normal, 0.0)).xyz);
    float3 albedo = surfaceColor(material, coordinates);
    float3 luminance = starlightLuminance(albedo);
    float illuminance = 0.0;

    for (uint index = 0; index < constants.frame.starCount; index++) {
        StarData star = constants.frame.stars[index];
        luminance += reflectedLuminance(star, material, coordinates, input, worldNormal, albedo);
        illuminance += receivedIlluminance(star, input.worldPosition, worldNormal);
    }

    if (material.emissiveTexture != noTexture) {
        luminance += sampleSurface(material.emissiveTexture, coordinates).rgb * material.emissiveLuminance;
    }

    float exposure = constants.frame.exposure;
    return surfaceOutput(float4(luminance * exposure, 1.0), illuminance * exposure, input);
}
