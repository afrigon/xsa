import common.scene;

[[vk::binding(2, 0)]]
SamplerState wrappingSampler;

[[vk::binding(3, 0)]]
Texture2D textures[];

static const uint noTexture = 0xFFFFFFFF;

[shader("vertex")]
VertexOutput vertexMain(VertexInput input)
{
    return transformVertex(input);
}

struct SurfaceCoordinates
{
    float2 uv;
    float2 uvPerPixelX;
    float2 uvPerPixelY;
};

float smallerChange(float wrappedAtZero, float wrappedAtHalf)
{
    return abs(wrappedAtZero) <= abs(wrappedAtHalf) ? wrappedAtZero : wrappedAtHalf;
}

// Equirectangular maps put longitude 0 at u = 0.25 with u growing westward, and the south pole at v = 0.
SurfaceCoordinates equirectangularCoordinates(float3 direction)
{
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

float4 sampleSurface(uint texture, SurfaceCoordinates coordinates)
{
    return textures[texture].SampleGrad(
        wrappingSampler, coordinates.uv, coordinates.uvPerPixelX, coordinates.uvPerPixelY);
}

float3 surfaceNormal(MaterialData material, float3 modelNormal, SurfaceCoordinates coordinates)
{
    if (material.normalTexture == noTexture)
    {
        return modelNormal;
    }
    float2 tangent = sampleSurface(material.normalTexture, coordinates).rg * 2.0 - 1.0;
    float outward = sqrt(saturate(1.0 - dot(tangent, tangent)));
    float3 east = normalize(float3(-modelNormal.y, modelNormal.x, 0.0) + float3(1e-7, 0.0, 0.0));
    float3 north = cross(modelNormal, east);
    return normalize(-east * tangent.x + north * tangent.y + modelNormal * outward);
}

[shader("fragment")]
float4 fragmentMain(VertexOutput input) : SV_Target
{
    MaterialData material = currentMaterial();
    float3 modelNormal = normalize(input.modelNormal);
    SurfaceCoordinates coordinates = equirectangularCoordinates(modelNormal);

    float3 normal = surfaceNormal(material, modelNormal, coordinates);
    float3 worldNormal = normalize(mul(currentObject().worldFromModel, float4(normal, 0.0)).xyz);
    float facingSun = sunCosine(input.worldPosition, normalize(input.normal)) > 0.0 ? 1.0 : 0.0;
    float cosine = sunCosine(input.worldPosition, worldNormal) * facingSun;

    float3 albedo = sampleSurface(material.texture, coordinates).rgb;
    float3 luminance = lambertLuminance(albedo, sunIlluminance(input.worldPosition), cosine);
    if (material.emissiveTexture != noTexture)
    {
        luminance += sampleSurface(material.emissiveTexture, coordinates).rgb * material.luminance;
    }
    float3 color = luminance * constants.frame.exposure;
    return float4(color, 1.0);
}
