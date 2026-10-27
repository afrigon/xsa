import light.bounce;
import light.lambert;
import light.star;
import terrain.layers;
import terrain.output;
import terrain.texture;

[shader("vertex")]
TerrainVaryings vertexMain(VertexInput input) {
    return terrainVaryings(terrainVertex(input, NoRelief()));
}

// Mirrors `TerrainLambertData` in the renderer. Its albedo and normals are the terrain's color and normal layers.
struct TerrainLambertData {
    float3 baseColor;
    uint emissiveTexture;
    float emissiveLuminance;
    uint padding[3];
};

[shader("fragment")]
FragmentOutput fragmentMain(TerrainVaryings vertex, uint primitive: SV_PrimitiveID) {
    DrawVaryings input = vertex.draw;
    TerrainLambertData material = currentMaterial<TerrainLambertData>();
    float3 modelNormal = normalize(input.modelNormal);
    LayerCoordinates layers = layerCoordinates(modelNormal);
    LayerRead color = readLayer(layers, colorLayer);
    float3 normal = layerNormal(readLayer(layers, normalLayer), layers, modelNormal);
    float3 worldNormal = normalize(mul(currentObject().worldFromModel, float4(normal, 0.0)).xyz);
    float3 geometricNormal = normalize(input.normal);
    float3 albedo = color.resident ? color.value.rgb : material.baseColor;
    TextureCoordinates coordinates = equirectangularCoordinates(modelNormal);
    float3 luminance = starlightLuminance(albedo);

    for (uint index = 0; index < constants.frame.starCount; index++) {
        StarData star = constants.frame.stars[index];
        float cosine = mappedCosine(star, input.worldPosition, geometricNormal, worldNormal);
        luminance += lambertLuminance(albedo, starIlluminance(star, input.worldPosition), cosine);
        float3 bounce = groundBounceIlluminance(star, input.worldPosition, worldNormal, geometricNormal, albedo);
        luminance += lambertLuminance(albedo, bounce, 1.0);
    }

    luminance += mappedEmission(material.emissiveTexture, material.emissiveLuminance, coordinates);
    MaterialSample sample;
    sample.albedo = albedo;
    sample.roughness = 0.0;
    return terrainOutput(luminance, sample, vertex, primitive);
}
