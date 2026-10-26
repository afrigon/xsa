// One placed model as a page stores it, mirroring `ScatterInstanceData` in the renderer: its offset from the page's
// center in the face's coordinates, its yaw (radians) and scale as two halves, its lean from upright and the direction
// it leans toward (radians) as two halves, the share of its height below the ground, and its model's place in its
// scatter.
struct ScatterInstanceData {
    float2 offset;
    uint yawAndScale;
    uint tiltAndHeading;
    float sink;
    uint model;
};

// A page of a scatter, mirroring `ScatterPageData` in the renderer: the quad sphere tile it covers, as `TileData`
// places a tile, with its instances and the scatter use they belong to.
struct ScatterPageData {
    float3 center;
    uint instanceCount;
    float3 previousCenter;
    uint use;
    float3 cubeCenter;
    float halfSize;
    float3 columnAxis;
    uint x;
    float3 rowAxis;
    uint y;
    ScatterInstanceData *instances;
    uint depth;
    uint face;
};

// A scatter placed on a body, mirroring `ScatterUseData` in the renderer: the body's object and terrain, its models
// from `firstModel`, its range (m) and tint. The camera stands at `cameraFromCenter` from the body's center, in radii
// of its lowest ground, and `horizonSquared` is that distance squared less one. The ground's average albedo stands in
// for its colour layer until that loads.
struct ScatterUseData {
    uint objectIndex;
    uint terrainIndex;
    uint firstModel;
    float range;
    float3 tint;
    float lowestRadius;
    float3 cameraFromCenter;
    float horizonSquared;
    float3 groundAlbedo;
    uint padding;
};

// One model of a scatter use, mirroring `ScatterModelData` in the renderer: its bounding box in the model's frame, the
// instance list it adds to (`noList` while it cannot be drawn) from `firstInstance`, and how it stands.
struct ScatterModelData {
    float3 boxCenter;
    uint list;
    float3 boxHalfExtent;
    uint firstInstance;
    uint orientation;
    uint padding[3];
};
