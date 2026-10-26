import draw;
import light.star;

// Lux per color channel the sunlit ground reflects onto a surface: the ground below as a Lambertian plane of
// `groundAlbedo`, lit by the star at its own angle, seen over the share of the surface's view below the horizon
// (a hemisphere light). `up` points away from the ground; the surface reflects it as Lambert does.
float3 groundBounceIlluminance(StarData star, float3 worldPosition, float3 normal, float3 up, float3 groundAlbedo) {
    float3 toStar = normalize(star.positionAndRadius.xyz - worldPosition);
    float groundCosine = max(dot(up, toStar), 0.0);
    float belowHorizon = (1.0 - dot(normal, up)) * 0.5;
    return groundAlbedo * starIlluminance(star, worldPosition) * groundCosine * belowHorizon;
}
