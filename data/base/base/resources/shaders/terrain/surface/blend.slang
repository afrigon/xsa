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

// Two materials' weights once the higher one shows through: displacements from 0 to 1, and a sharpness from 0 (a
// cross-fade over the whole displacement range) toward 1 (only the higher material).
float2 heightBlend(float2 weights, float2 displacements, float sharpness) {
    float depth = 1.0 - sharpness;
    float2 raised = weights + displacements;
    float top = max(raised.x, raised.y) - depth;
    float2 shown = max(raised - top, 0.0);
    return shown / (shown.x + shown.y);
}
