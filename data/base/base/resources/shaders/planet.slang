import common.hapke;
import common.scene;

[[vk::binding(2, 0)]]
SamplerState wrappingSampler;

[[vk::binding(3, 0)]]
Texture2D textures[];

static const uint noTexture = 0xFFFFFFFF;

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

float3 surfaceNormal(MaterialData material, float3 modelNormal, SurfaceCoordinates coordinates) {
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

HapkeSurface hapkeSurface(MaterialData material, SurfaceCoordinates coordinates, float3 albedo) {
    float4 scatter = sampleSurface(material.hapke.scatterTexture, coordinates);
    float4 surge = sampleSurface(material.hapke.surgeTexture, coordinates);
    HapkeSurface surface;
    surface.albedo = albedo;
    surface.singleScatteringAlbedo = scatter.r;
    surface.lobeWidth = scatter.g;
    surface.lobeBalance = 2.0 * scatter.b - 1.0;
    surface.shadowHidingAmplitude = 2.0 * surge.r;
    surface.shadowHidingWidth = surge.g;
    surface.backscatterAmplitude = 2.0 * surge.b;
    surface.backscatterWidth = surge.a;
    surface.porosity = material.hapke.porosity;
    surface.roughness = material.hapke.roughness;
    return surface;
}

// The scatter map gives the regional single-scattering albedo at a fraction of the colour map's resolution; the
// colour map adds colour and fine detail as its ratio to its own average at the scatter map's resolution.
float3 textbookAlbedo(MaterialData material, SurfaceCoordinates coordinates, float3 color,
                      float singleScatteringAlbedo) {
    uint colorWidth, colorHeight, scatterWidth, scatterHeight;
    textures[material.texture].GetDimensions(colorWidth, colorHeight);
    textures[material.hapke.scatterTexture].GetDimensions(scatterWidth, scatterHeight);
    float scale = max(float(colorWidth) / float(scatterWidth), 1.0);
    float3 regional = sampleSurfaceCoarser(material.texture, coordinates, scale).rgb;
    float regionalLuminance = max(dot(regional, float3(0.2126, 0.7152, 0.0722)), 1e-4);
    return singleScatteringAlbedo * color / regionalLuminance;
}

[shader("fragment")]
SurfaceOutput fragmentMain(VertexOutput input) {
    MaterialData material = currentMaterial();
    float3 modelNormal = normalize(input.modelNormal);
    SurfaceCoordinates coordinates = equirectangularCoordinates(modelNormal);

    float3 normal = surfaceNormal(material, modelNormal, coordinates);
    float3 worldNormal = normalize(mul(currentObject().worldFromModel, float4(normal, 0.0)).xyz);
    float facingSun = sunCosine(input.worldPosition, normalize(input.normal)) > 0.0 ? 1.0 : 0.0;
    float cosine = sunCosine(input.worldPosition, worldNormal) * facingSun;

    float3 albedo = sampleSurface(material.texture, coordinates).rgb;
    float3 illuminance = sunIlluminance(input.worldPosition);
    float3 luminance = lambertLuminance(albedo, illuminance, cosine);
    uint shading = constants.frame.shadingModel;
    if (material.hapke.scatterTexture != noTexture && cosine > 0.0 &&
        (shading == shadingHapkeSol || shading == shadingHapkeTextbook)) {
        float3 toCamera = normalize(-input.worldPosition);
        float3 toSun = normalize(constants.frame.sunPosition.xyz - input.worldPosition);
        HapkeGeometry geometry;
        geometry.incidenceCosine = clamp(dot(toSun, worldNormal), 1e-4, 1.0);
        geometry.emissionCosine = clamp(dot(toCamera, worldNormal), 1e-4, 1.0);
        geometry.phaseAngle = acos(clamp(dot(toSun, toCamera), -1.0, 1.0));
        HapkeSurface surface = hapkeSurface(material, coordinates, albedo);
        float3 reflectance;
        if (shading == shadingHapkeSol) {
            reflectance =
                hapkeSol(surface, geometry, material.hapke.blend, material.hapke.lightBoost, material.hapke.gammaBoost);
        } else {
            surface.albedo = textbookAlbedo(material, coordinates, albedo, surface.singleScatteringAlbedo);
            reflectance = hapkeTextbook(surface, geometry);
        }
        luminance = reflectance * illuminance + starlightLuminance(albedo);
    }
    if (material.emissiveTexture != noTexture) {
        luminance += sampleSurface(material.emissiveTexture, coordinates).rgb * material.luminance;
    }
    float3 color = luminance * constants.frame.exposure;
    return surfaceOutput(float4(color, 1.0), input);
}
