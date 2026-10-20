import terrain.debug.pages;

// Each pixel takes the hue of the level of the color page it reads; page edges are dark, missing pages white.
float3 pageLevelsColor(TerrainVaryings input) {
    ColorPage page = colorPage(input);
    return shadedPage(page, hueColor(float(page.read.level) * huePerLevel));
}
