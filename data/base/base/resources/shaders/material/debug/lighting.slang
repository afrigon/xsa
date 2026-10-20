import material.light.star;
import material.material;

[shader("vertex")]
DrawVaryings vertexMain<G : Geometry>(VertexInput input) {
    return G.transform(input).draw;
}

[shader("fragment")]
FragmentOutput fragmentMain(DrawVaryings input) {
    float3 normal = normalize(input.normal);
    float cosine = 0.0;

    for (uint index = 0; index < constants.frame.starCount; index++) {
        cosine += starCosine(constants.frame.stars[index], input.worldPosition, normal);
    }

    return fragmentOutput(float4(saturate(cosine).xxx, 1.0), input);
}
