import draw;
import light.star;

float3 lightingColor(DrawVaryings input) {
    float3 normal = normalize(input.normal);
    float cosine = 0.0;

    for (uint index = 0; index < constants.frame.starCount; index++) {
        cosine += starCosine(constants.frame.stars[index], input.worldPosition, normal);
    }

    return saturate(cosine).xxx;
}
