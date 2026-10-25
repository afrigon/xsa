// Reoriented normal mapping (Barré-Brisebois and Hill, 2012): `detail` turned the way `base` turns the z axis, both
// unit vectors in the same tangent frame.
float3 reorientedNormal(float3 base, float3 detail) {
    float3 turned = base + float3(0.0, 0.0, 1.0);
    float3 mirrored = detail * float3(-1.0, -1.0, 1.0);
    return turned * dot(turned, mirrored) / turned.z - mirrored;
}
