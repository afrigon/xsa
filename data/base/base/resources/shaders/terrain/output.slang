__exported import debug.view;
__exported import terrain.vertex;
import terrain.debug.terrain_view;

// What a terrain program writes: the view an override selects, the terrain's before the generic one, or the
// luminance it shaded.
FragmentOutput terrainOutput(float3 luminance, MaterialSample sample, TerrainVaryings input, uint primitive) {
    if (showsTerrainDebugView()) {
        return fragmentOutput(float4(terrainDebugViewColor(input), 1.0), input.draw);
    }

    if (showsDebugView()) {
        return fragmentOutput(float4(debugViewColor(input.draw, sample, primitive), 1.0), input.draw);
    }

    return fragmentOutput(float4(luminance * constants.frame.exposure, 1.0), input.draw);
}
