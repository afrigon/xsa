import mesh.instance;
import scatter.page;
import terrain.cube;
import terrain.heights;
import terrain.layers;
import terrain.tile;

static const uint workgroupSize = 64;
// The far fade spans the last tenth of a scatter's range.
static const float fadeFraction = 0.1;
// An instance fades out as its box comes within this many meters of the near plane, gone as it reaches it.
static const float nearFadeMeters = 0.5;
// An instance fades out as its bounding box's diagonal shrinks from the first to the second length on screen, in
// pixels.
static const float fullSizePixels = 4.0;
static const float goneSizePixels = 2.0;
// Must match the client: `NEAR_PLANE` in camera.rs and `NO_LIST` in the renderer's scatter module.
static const float nearPlane = 0.1;
static const uint noList = 0xFFFFFFFF;

// Must match `ScatterModelData::orientation` in the renderer.
enum ScatterOrientation : uint {
    Ground = 0,
    Up = 1,
}

struct ScatterConstants {
    FrameData *frame;
    ObjectData *objects;
    TerrainData *terrains;
    ScatterPageData *pages;
    ScatterUseData *uses;
    ScatterModelData *models;
    // One counter per instance list, then the count of instances within range.
    uint *counts;
    MeshInstanceData *instances;
    uint listCount;
    uint padding;
};

[[vk::push_constant]]
ConstantBuffer<ScatterConstants> scatter;

// Two halves packed low first, read without 16-bit types, which would need device features of their own.
float2 unpackHalves(uint packed) {
    return spirv_asm { result : $$float2 = OpExtInst glsl450 UnpackHalf2x16 $packed };
}

TileData pageTile(ScatterPageData page) {
    TileData tile;
    tile.center = page.center;
    tile.detail = 0;
    tile.previousCenter = page.previousCenter;
    tile.gridCells = 1;
    tile.cubeCenter = page.cubeCenter;
    tile.morphStart = 0.0;
    tile.columnAxis = page.columnAxis;
    tile.morphEnd = 0.0;
    tile.rowAxis = page.rowAxis;
    tile.halfSize = page.halfSize;
    tile.x = page.x;
    tile.y = page.y;
    tile.depth = page.depth;
    tile.face = page.face;
    return tile;
}

// From the page's center to the ground `offset` from it, in radii: on the finest resident level of the heights,
// without the surfaces' relief.
float3 groundOffset(TerrainData *terrain, TileData tile, float2 offset) {
    float3 onSphere = sphereOffsetAt(tile, offset);
    float3 direction = normalize(normalize(tile.cubeCenter) + onSphere);
    float height = levelHeight(terrain, tile, terrain.heightFirstLevel, offset) * scatter.frame.terrainHeightScale;
    return onSphere + direction * height;
}

// The ground's normal across one texel of the level its height is read at.
float3 groundNormal(TerrainData *terrain, TileData tile, float2 offset, float3 ground, float3 up) {
    float texel = 2.0 / float(max(terrain.heightExtent >> terrain.heightFirstLevel, 1u));
    float3 alongColumns = groundOffset(terrain, tile, offset + float2(texel, 0.0)) - ground;
    float3 alongRows = groundOffset(terrain, tile, offset + float2(0.0, texel)) - ground;
    float3 normal = normalize(cross(alongColumns, alongRows));
    return dot(normal, up) < 0.0 ? -normal : normal;
}

// An oriented box, by its center and its half axes, against the frustum's side planes.
bool inFrustum(float3 center, float3 halfAxes[3]) {
    float4x4 viewProjection = scatter.frame.unjitteredViewProjection;
    float4 planes[4] = {
        viewProjection[3] + viewProjection[0],
        viewProjection[3] - viewProjection[0],
        viewProjection[3] + viewProjection[1],
        viewProjection[3] - viewProjection[1],
    };

    for (uint index = 0; index < 4; index++) {
        float4 plane = planes[index];

        float reach =
            abs(dot(plane.xyz, halfAxes[0])) + abs(dot(plane.xyz, halfAxes[1])) + abs(dot(plane.xyz, halfAxes[2]));

        if (dot(plane.xyz, center) + plane.w < -reach) {
            return false;
        }
    }

    return true;
}

// Behind the body's lowest ground as the camera sees it (Cozzi and Ring, "3D Engine Design for Virtual Globes").
bool belowHorizon(ScatterUseData use, float3 bodyCenter, float3 point) {
    if (use.horizonSquared <= 0.0) {
        return false;
    }

    float3 toPoint = (point - bodyCenter) / use.lowestRadius - use.cameraFromCenter;
    float alongCenter = dot(toPoint, -use.cameraFromCenter);
    return alongCenter > use.horizonSquared && alongCenter * alongCenter / dot(toPoint, toPoint) > use.horizonSquared;
}

// From the camera to the instance's box, in meters. `axes` are the model's axes in the world, `scale` long.
float boxDistance(ScatterModelData model, float3 axes[3], float3 origin, float scale) {
    float3 camera = -origin;
    float3 local = float3(dot(camera, axes[0]), dot(camera, axes[1]), dot(camera, axes[2])) / (scale * scale);
    float3 outside = max(abs(local - model.boxCenter) - model.boxHalfExtent, 0.0);
    return length(outside) * scale;
}

// The ground's albedo under the instance, from the terrain's colour layer where it is resident.
float3 groundAlbedo(ScatterUseData use, TerrainData *terrain, float3 direction) {
    uint texture = terrain.layerTextures[colorLayer];

    if (texture == noTexture) {
        return use.groundAlbedo;
    }

    return cubeTextures[NonUniformResourceIndex(texture)]
        .SampleLevel(wrappingSampler, equalAngleSampleDirection(direction), 0.0)
        .rgb;
}

void addInstance(ScatterModelData model, MeshInstanceData instance) {
    uint slot;
    InterlockedAdd(scatter.counts[model.list], 1, slot);
    scatter.instances[model.firstInstance + slot] = instance;
}

// Where an instance stands this frame and the last, and how visible it is; whether it lies within range.
bool cullInstance(ScatterPageData page, ScatterInstanceData stored) {
    ScatterUseData use = scatter.uses[page.use];
    ScatterModelData model = scatter.models[use.firstModel + stored.model];
    ObjectData object = scatter.objects[use.objectIndex];
    TerrainData *terrain = &scatter.terrains[use.terrainIndex];
    TileData tile = pageTile(page);
    float3 ground = groundOffset(terrain, tile, stored.offset);
    float3 world = tile.center + mul(object.worldFromModel, float4(ground, 0.0)).xyz;
    float distance = length(world);

    if (distance > use.range) {
        return false;
    }

    if (model.list == noList) {
        return true;
    }

    float3 radial = normalize(normalize(tile.cubeCenter) + sphereOffsetAt(tile, stored.offset));
    float3 up = radial;

    if (model.orientation == uint(ScatterOrientation.Ground)) {
        up = groundNormal(terrain, tile, stored.offset, ground, up);
    }

    float2 yawAndScale = unpackHalves(stored.yawAndScale);
    float2 tiltAndHeading = unpackHalves(stored.tiltAndHeading);
    float yaw = yawAndScale.x;
    float scale = yawAndScale.y;
    float3 reference = normalize(tile.columnAxis - dot(tile.columnAxis, up) * up);
    float3 side = cross(up, reference);
    float3 leanDirection = reference * cos(tiltAndHeading.y) + side * sin(tiltAndHeading.y);
    up = normalize(up * cos(tiltAndHeading.x) + leanDirection * sin(tiltAndHeading.x));
    reference = normalize(reference - dot(reference, up) * up);
    side = cross(up, reference);
    float3 modelX = reference * cos(yaw) + side * sin(yaw);
    float3 modelY = cross(up, modelX);
    float radius = length(object.worldFromModel[0].xyz);
    float3x3 axes = float3x3(modelX, modelY, up);
    // Sunk along its own up axis, by its share of the box's height; the ground offset is in radii.
    float sinkMeters = stored.sink * 2.0 * model.boxHalfExtent.z * scale;
    ground -= up * sinkMeters / radius;
    world = tile.center + mul(object.worldFromModel, float4(ground, 0.0)).xyz;
    MeshInstanceData instance;
    float3 previousWorld = tile.previousCenter + mul(object.previousWorldFromModel, float4(ground, 0.0)).xyz;

    for (uint row = 0; row < 3; row++) {
        instance.worldFromModel[row] = float4(0.0, 0.0, 0.0, world[row]);
        instance.previousWorldFromModel[row] = float4(0.0, 0.0, 0.0, previousWorld[row]);
    }

    float3 worldAxes[3];
    float3 previousAxes[3];

    for (uint axis = 0; axis < 3; axis++) {
        worldAxes[axis] = mul(object.worldFromModel, float4(axes[axis], 0.0)).xyz / radius * scale;
        previousAxes[axis] = mul(object.previousWorldFromModel, float4(axes[axis], 0.0)).xyz / radius * scale;

        for (uint row = 0; row < 3; row++) {
            instance.worldFromModel[row][axis] = worldAxes[axis][row];
            instance.previousWorldFromModel[row][axis] = previousAxes[axis][row];
        }
    }

    float3 boxCenter =
        world + worldAxes[0] * model.boxCenter.x + worldAxes[1] * model.boxCenter.y + worldAxes[2] * model.boxCenter.z;
    float3 halfAxes[3] = {
        worldAxes[0] * model.boxHalfExtent.x,
        worldAxes[1] * model.boxHalfExtent.y,
        worldAxes[2] * model.boxHalfExtent.z,
    };
    float boxRadius = length(model.boxHalfExtent) * scale;
    float3 bodyCenter = world - mul(object.worldFromModel, float4(normalize(tile.cubeCenter) + ground, 0.0)).xyz;
    float3 worldUp = normalize(worldAxes[2]);
    float far = saturate((use.range - distance) / (use.range * fadeFraction));
    float near = saturate((boxDistance(model, worldAxes, world, scale) - nearPlane) / nearFadeMeters);
    float pixels = 2.0 * boxRadius / (max(length(boxCenter), nearPlane) * scatter.frame.pixelAngle);
    float size = saturate((pixels - goneSizePixels) / (fullSizePixels - goneSizePixels));
    instance.tint = use.tint;
    instance.visibility = min(min(far, near), size);
    instance.groundUp = normalize(mul(object.worldFromModel, float4(radial, 0.0)).xyz);
    instance.padding = 0;
    instance.groundAlbedo = groundAlbedo(use, terrain, radial);
    instance.groundPadding = 0;

    if (instance.visibility > 0.0 && inFrustum(boxCenter, halfAxes) &&
        !belowHorizon(use, bodyCenter, boxCenter + worldUp * boxRadius)) {
        addInstance(model, instance);
    }

    return true;
}

[shader("compute")]
[numthreads(workgroupSize, 1, 1)]
void computeMain(uint3 group: SV_GroupID, uint3 thread: SV_GroupThreadID) {
    ScatterPageData page = scatter.pages[group.x];

    for (uint index = thread.x; index < page.instanceCount; index += workgroupSize) {
        bool inRange = cullInstance(page, page.instances[index]);
        uint inRangeCount = WaveActiveCountBits(inRange);

        if (WaveIsFirstLane() && inRangeCount > 0) {
            InterlockedAdd(scatter.counts[scatter.listCount], inRangeCount);
        }
    }
}
