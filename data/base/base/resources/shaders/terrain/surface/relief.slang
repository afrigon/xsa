import draw;
import terrain.layers;
import terrain.surface.blend;
import terrain.surface.classes;
import terrain.surface.range;
import terrain.surface.surface;
import terrain.tile;

// Moves the ground near the camera by its surfaces' displacement, blended between the classes by height as its pixels
// blend them. Collision does not see it.
struct SurfaceRelief : TileRelief {
    BiomeData *biome;
    SurfaceData *surfaces;
    bool loaded;

    float relief(uint tile, float3 world, float3 modelNormal, float2 faceOffset, TerrainShape shape) {
        if (!shadesSurfaces() || !loaded) {
            return 0.0;
        }

        float3x3 worldFromModel = (float3x3)currentObject().worldFromModel;
        float3 offset = mul(transpose(worldFromModel), world) / length(worldFromModel[0]);
        float distance = length(world);
        float pixelFootprint = distance * constants.frame.pixelAngle;
        float vertexSpacing = distance * constants.frame.vertexSpacingAngle;
        LayerCoordinates coordinates = magnifiedLayerCoordinates(tile, faceOffset);
        float3 normal = layerNormal(readLayer(coordinates, normalLayer), coordinates.tile, modelNormal);
        SurfaceAnchors *anchors = surfaceAnchors();
        float2 weights =
            surfaceClassWeights(*biome, anchors, offset, normal, modelNormal, shape.curvature, shape.relativeHeight);
        SurfaceData *flatSurface = &surfaces[uint(SurfaceClass.Flat)];
        SurfaceData *slopeSurface = &surfaces[uint(SurfaceClass.Slope)];
        ReliefSample flat = noRelief();
        ReliefSample sloped = noRelief();

        if (weights.x > 0.0) {
            flat = surfaceReliefSample(
                flatSurface, anchors, uint(SurfaceClass.Flat), offset, normal, pixelFootprint, vertexSpacing);
        }

        if (weights.y > 0.0) {
            sloped = surfaceReliefSample(
                slopeSurface, anchors, uint(SurfaceClass.Slope), offset, normal, pixelFootprint, vertexSpacing);
        }

        if (weights.x > 0.0 && weights.y > 0.0) {
            float contrast = 0.5 * (flatSurface.heightContrast + slopeSurface.heightContrast);
            weights = heightBlend(weights, float2(flat.displacement, sloped.displacement), contrast);
        }

        return flat.meters * weights.x + sloped.meters * weights.y;
    }
}
