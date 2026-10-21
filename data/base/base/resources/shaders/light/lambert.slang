import draw;

// cd/m² reflected by a Lambertian surface of the given albedo.
float3 lambertLuminance(float3 albedo, float3 illuminance, float cosine) {
    return albedo / pi * illuminance * cosine;
}
