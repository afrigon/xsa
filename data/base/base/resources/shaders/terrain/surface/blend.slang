// Sums of uncorrelated texture samples that keep their contrast: a weighted average halves the variation around the
// mean where two samples weigh the same, so deviations are scaled back by the weights' root sum of squares (Heitz and
// Neyret, "High-Performance By-Example Noise using a Histogram-Preserving Blending Operator", 2018).
struct ContrastBlend {
    float3 sum;
    float weights;
    float squaredWeights;

    [mutating]
    void add(float3 deviation, float weight) {
        sum += deviation * weight;
        weights += weight;
        squaredWeights += weight * weight;
    }

    float3 deviation() {
        return squaredWeights > 0.0 ? sum / sqrt(squaredWeights) : float3(0.0);
    }

    float3 average() {
        return weights > 0.0 ? sum / weights : float3(0.0);
    }
}

ContrastBlend contrastBlend() {
    ContrastBlend blend;
    blend.sum = float3(0.0);
    blend.weights = 0.0;
    blend.squaredWeights = 0.0;
    return blend;
}

// One class and variant of the ground, with its weight.
struct Material {
    uint surfaceClass;
    uint variant;
    float weight;
};

// The two strongest of the materials offered, each less the third's weight, so a material leaving the two fades out
// before it is dropped.
struct StrongestTwo {
    Material first;
    Material second;
    float third;

    [mutating]
    void offer(Material material) {
        if (material.weight > first.weight) {
            third = second.weight;
            second = first;
            first = material;
        } else if (material.weight > second.weight) {
            third = second.weight;
            second = material;
        } else {
            third = max(third, material.weight);
        }
    }

    // The two weights, summing to 1.
    float2 weights() {
        float2 kept = float2(first.weight, second.weight) - third;
        float sum = kept.x + kept.y;
        return sum > 0.0 ? kept / sum : float2(1.0, 0.0);
    }
}

StrongestTwo strongestTwo() {
    Material none;
    none.surfaceClass = 0;
    none.variant = 0;
    none.weight = 0.0;
    StrongestTwo strongest;
    strongest.first = none;
    strongest.second = none;
    strongest.third = 0.0;
    return strongest;
}

// Two materials' weights once the higher one shows through: displacements from 0 to 1, and a contrast from 0 (a
// cross-fade over the whole displacement range) toward 1 (only the higher material).
float2 heightBlend(float2 weights, float2 displacements, float contrast) {
    float depth = 1.0 - contrast;
    float2 raised = weights + displacements;
    float top = max(raised.x, raised.y) - depth;
    float2 shown = max(raised - top, 0.0);
    return shown / (shown.x + shown.y);
}
