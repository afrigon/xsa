import noise;
import terrain.surface.surface_data;

// The hexagonal cells' centres are the corners of a grid of equilateral triangles: skewing the plane puts them on the
// integer lattice. `skewedFromGrid` mirrors `SKEWED_FROM_GRID` in the renderer.
static const float2x2 skewedFromGrid = float2x2(1.0, -0.57735027, 0.0, 1.15470054);
static const float2x2 gridFromSkewed = float2x2(1.0, 0.5, 0.0, 0.8660254);

// How far below the most raised cell another still shows: under a third, so a cell is gone before the far edge of its
// triangle, where its weight reaches 0.
static const float hexBlendDepth = 0.25;

static const float hashScale = 1.0 / 4294967296.0;

// The three cells around a point: their ids, the point's offset from each centre (cells) and their weights, which
// fall from 1 at a cell's centre to 0 at the far edge of the triangle.
struct HexCorners {
    int2 cells[3];
    float2 offsets[3];
    float3 weights;
};

// `offset` is the pixel's offset from the anchor, in cells.
HexCorners hexCorners(HexAnchor anchor, float2 offset) {
    float2 skewed = anchor.fraction + mul(skewedFromGrid, offset);
    float2 whole = floor(skewed);
    float2 fraction = skewed - whole;
    int2 base = anchor.cell + int2(whole);
    float rest = 1.0 - fraction.x - fraction.y;
    bool upper = rest < 0.0;
    int2 corners[3] = { upper ? int2(1, 1) : int2(0, 0),
                        upper ? int2(1, 0) : int2(0, 1),
                        upper ? int2(0, 1) : int2(1, 0) };

    HexCorners result;
    result.weights = upper ? float3(-rest, 1.0 - fraction.y, 1.0 - fraction.x) : float3(rest, fraction.y, fraction.x);

    [ForceUnroll]
    for (int corner = 0; corner < 3; corner++) {
        result.cells[corner] = base + corners[corner];
        result.offsets[corner] = mul(gridFromSkewed, fraction - float2(corners[corner]));
    }

    return result;
}

// How a cell lays the texture: turned by its own angle (as a matrix) and shifted (repeats), from its id hashed with
// the surface's seed. `layer` tells apart the lattices of a surface's projection planes.
struct HexCellPlacement {
    float2x2 turn;
    float2 shift;
    float hue;
};

HexCellPlacement hexCellPlacement(int2 cell, uint layer, uint seed) {
    uint3 hash = pcg3d(uint3(int3(cell, int(layer))) ^ uint3(seed, seed * 0x9E3779B9u, seed * 0x85EBCA6Bu));
    float3 random = float3(hash) * hashScale;
    float angle = random.x * 2.0 * float.getPi();
    float cosine = cos(angle);
    float sine = sin(angle);

    HexCellPlacement placement;
    placement.turn = float2x2(cosine, -sine, sine, cosine);
    placement.shift = random.yz;
    placement.hue = random.x;
    return placement;
}

// The cells' weights once the most raised shows through: each weight scaled up by its displacement (0 to 1), and only
// the cells within `hexBlendDepth` of the highest kept, so features stay whole and most pixels read one or two cells.
float3 hexBlend(float3 weights, float3 displacements) {
    float3 raised = weights * (1.0 + displacements);
    float top = max(raised.x, max(raised.y, raised.z)) - hexBlendDepth;
    float3 shown = max(raised - top, 0.0);
    return shown / (shown.x + shown.y + shown.z);
}
