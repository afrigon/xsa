// Counted on the GPU as it culls, so they describe the frame a frame slot last drew.
#[derive(Clone, Copy, Default)]
pub struct ScatterStatistics {
    pub triangles: u64,
    pub drawn_instances: u64,
    pub instances_in_range: u64,
    // Meshes with at least one instance drawn.
    pub meshes: usize,
}

impl ScatterStatistics {
    // `counts` holds one counter per list, then the count within range; `lists` the triangles of each list's instances.
    pub(in crate::renderer) fn from_counts(counts: &[u32], lists: &[u64]) -> ScatterStatistics {
        let drawn = || counts.iter().zip(lists).filter(|(count, _)| **count > 0);

        ScatterStatistics {
            triangles: drawn().map(|(count, triangles)| u64::from(*count) * triangles).sum(),
            drawn_instances: drawn().map(|(count, _)| u64::from(*count)).sum(),
            instances_in_range: counts.get(lists.len()).copied().map_or(0, u64::from),
            meshes: drawn().count(),
        }
    }
}
