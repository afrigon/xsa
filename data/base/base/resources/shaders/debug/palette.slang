// Neighboring levels get clearly different hues; the cycle repeats every few levels.
static const float huePerLevel = 0.17;
// Surfaces facing away from the camera keep this much of their color.
static const float ambientShade = 0.35;

float3 hueColor(float hue) {
    float3 channels = abs(frac(hue + float3(0.0, 2.0 / 3.0, 1.0 / 3.0)) * 6.0 - 3.0) - 1.0;
    return saturate(channels);
}

// 1 on a line through whole values of `coordinates`, fading to 0 over `widthPixels` on screen.
float lineCoverage(float2 coordinates, float widthPixels) {
    float2 distanceToLine = abs(coordinates - round(coordinates)) / fwidth(coordinates);
    return 1.0 - saturate(min(distanceToLine.x, distanceToLine.y) / widthPixels);
}
