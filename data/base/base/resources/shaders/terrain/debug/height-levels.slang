import debug.palette;
import terrain.heights;
import terrain.tile;

// Where the level the distance asks for is not resident, the pixel keeps this much of its color.
static const float unresidentShade = 0.4;
static const float3 noHeightsColor = float3(1.0);

// Each pixel takes the hue of the height level the ground there reads, by the vertices' rule at the pixel's distance:
// whole levels in their own hue, blends between them in between, dimmed where a finer level is wanted than resident.
float3 heightLevelsColor(TerrainVaryings input) {
    TerrainData *terrain = terrainData();

    if (terrain.heightTexture == noTexture) {
        return noHeightsColor;
    }

    float radius = length(currentObject().worldFromModel[0].xyz);
    float distance = length(input.draw.worldPosition);
    HeightLevels levels = heightLevels(terrain, distance, radius);
    float level = float(levels.finer) + levels.blend;
    float wanted = log2(distance / radius * constants.frame.vertexSpacingAngle / texelSpacing(terrain, 0));
    float facing = saturate(dot(normalize(input.draw.normal), normalize(-input.draw.worldPosition)));
    float3 color = hueColor(level * huePerLevel) * lerp(ambientShade, 1.0, facing);
    return wanted < float(terrain.heightFirstLevel) ? color * unresidentShade : color;
}
