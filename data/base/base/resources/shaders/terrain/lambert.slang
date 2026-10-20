import light.lambert;
import light.star;
import terrain.output;
import terrain.texture;

[shader("vertex")]
TerrainVaryings vertexMain(VertexInput input) {
    return terrainVaryings(terrainVertex(input));
}

struct LambertData {
    float3 baseColor;
    uint colorTexture;
    uint normalTexture;
    uint emissiveTexture;
    float emissiveLuminance;
    uint padding;
};

[shader("fragment")]
FragmentOutput fragmentMain(TerrainVaryings vertex, uint primitive: SV_PrimitiveID) {
    DrawVaryings input = vertex.draw;
    LambertData material = currentMaterial<LambertData>();
    float3 modelNormal = normalize(input.modelNormal);
    TextureCoordinates coordinates = equirectangularCoordinates(modelNormal);
    float3 normal = mappedNormal(material.normalTexture, modelNormal, coordinates);
    float3 worldNormal = normalize(mul(currentObject().worldFromModel, float4(normal, 0.0)).xyz);
    float3 geometricNormal = normalize(input.normal);
    float3 albedo = mappedColor(material.colorTexture, material.baseColor, coordinates);
    float3 luminance = starlightLuminance(albedo);

    for (uint index = 0; index < constants.frame.starCount; index++) {
        StarData star = constants.frame.stars[index];
        float cosine = mappedCosine(star, input.worldPosition, geometricNormal, worldNormal);
        luminance += lambertLuminance(albedo, starIlluminance(star, input.worldPosition), cosine);
    }

    luminance += mappedEmission(material.emissiveTexture, material.emissiveLuminance, coordinates);
    MaterialSample sample;
    sample.albedo = albedo;
    sample.roughness = 0.0;
    return terrainOutput(luminance, sample, vertex, primitive);
}
