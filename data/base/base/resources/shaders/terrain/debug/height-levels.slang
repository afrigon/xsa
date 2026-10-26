import debug.palette;
import terrain.tile;

// Where a finer level would match the tile than is resident, the pixel keeps this much of its color.
static const float unresidentShade = 0.4;
static const float3 noHeightsColor = float3(1.0);

// Each pixel takes the hue of the height level its vertices read: whole levels in their own hue, a tile morphing
// toward its parent's level in between, dimmed where the level matching its tile is not resident.
float3 heightLevelsColor(TerrainVaryings input) {
    TerrainData *terrain = terrainData();

    if (terrain.heightTexture == noTexture) {
        return noHeightsColor;
    }

    float facing = saturate(dot(normalize(input.draw.normal), normalize(-input.draw.worldPosition)));
    float3 color = hueColor(input.heightLevel * huePerLevel) * lerp(ambientShade, 1.0, facing);
    return input.matchingHeightLevel < int(terrain.heightFirstLevel) ? color * unresidentShade : color;
}
