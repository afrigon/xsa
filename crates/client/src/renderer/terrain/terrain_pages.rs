use std::path::PathBuf;
use std::sync::Arc;

use glam::DVec3;
use resource_vulkan::{Residency, ResourceGroup, Resources, StreamedSet};
use xsa_core::terrain::HeightPages;
use xsa_packs::HeightPageReader;

use super::{
    HeightPageSource, LayerPageSource, PageRanges, StreamedLayer, TerrainHeights, TerrainLayer, TerrainPageStatistics,
};

const FINEST_LEVEL_BITS: u32 = 5;

// A terrain's streamed data: its heights and the layers its material shades with, each a `.terrain` file whose pages
// load as tiles drawn request them. Shared by the renderer, which requests and draws them, and the world, which reads
// the ground from the heights.
pub struct TerrainPages {
    resources: Resources,
    ranges: Arc<PageRanges>,
    heights: Option<StreamedLayer<HeightPageSource>>,
    layers: [Option<StreamedLayer<LayerPageSource>>; TerrainLayer::ALL.len()],
}

impl TerrainPages {
    // `layers` by `TerrainLayer` index.
    pub fn new(
        resources: &Resources,
        group: &mut ResourceGroup,
        body: &str,
        heights: Option<PathBuf>,
        layers: [Option<PathBuf>; TerrainLayer::ALL.len()],
    ) -> TerrainPages {
        let ranges = Arc::new(PageRanges::default());
        let heights = heights.map(|path| {
            let source = HeightPageSource {
                reader: HeightPageReader::new(path.clone()),
                ranges: Arc::clone(&ranges),
            };
            StreamedLayer::new(resources, group, path, source, format!("{body} heights"))
        });
        let mut paths = layers.into_iter();
        let layers = TerrainLayer::ALL.map(|layer| {
            let path = paths.next().flatten()?;
            let source = LayerPageSource { path: path.clone() };

            Some(StreamedLayer::new(
                resources,
                group,
                path,
                source,
                format!("{body} {}", layer.name()),
            ))
        });

        TerrainPages {
            resources: resources.clone(),
            ranges,
            heights,
            layers,
        }
    }

    // Meters relative to the radius toward `direction`, a unit vector in the body's frame; None until a page
    // covering it is loaded.
    pub fn height_at(&self, direction: DVec3) -> Option<f64> {
        self.heights()?.height_at(direction)
    }

    // Whether the ground toward `direction` is as fine as it will get: read from the finest heights, or with no
    // heights left loading. True for a terrain without heights.
    pub fn is_ground_settled(&self, direction: DVec3) -> bool {
        let Some(layer) = &self.heights else {
            return true;
        };
        let Some(heights) = self.heights() else {
            return false;
        };
        let loading = layer.set().is_some_and(|set| set.statistics().pending > 0);

        !loading
            || heights
                .height_sample_at(direction)
                .is_some_and(|sample| sample.level == heights.finest_level)
    }

    // Registers the sets of the files loaded since, and keeps every set's pages following its requests or frozen.
    pub(in crate::renderer) fn update(&self, residency: Residency) {
        if let Some(heights) = &self.heights {
            if heights.update(&self.resources)
                && let Some(file) = heights.file()
            {
                for &(address, range) in file.level_zero_ranges() {
                    self.ranges.record(address, range);
                }
            }

            if let Some(set) = heights.set() {
                set.set_residency(residency);
            }
        }

        for layer in self.layers.iter().flatten() {
            layer.update(&self.resources);

            if let Some(set) = layer.set() {
                set.set_residency(residency);
            }
        }
    }

    // None for a smooth terrain, and until its height file's header is loaded.
    pub(in crate::renderer) fn heights(&self) -> Option<TerrainHeights<'_>> {
        let heights = self.heights.as_ref()?;

        Some(TerrainHeights {
            set: heights.set()?,
            ranges: &self.ranges,
            finest_level: heights.finest_level()?,
        })
    }

    // A layer's set and finest level; None without the layer, and until its file's header is loaded.
    pub(in crate::renderer) fn layer(&self, layer: TerrainLayer) -> Option<(&StreamedSet<LayerPageSource>, u32)> {
        let layer = self.layers[layer.index()].as_ref()?;

        Some((layer.set()?, layer.finest_level()?))
    }

    // The finest level of each loaded layer, five bits each by `TerrainLayer` index, the first lowest.
    pub(in crate::renderer) fn layer_finest_levels(&self) -> u32 {
        TerrainLayer::ALL
            .into_iter()
            .filter_map(|layer| Some((layer, self.layer(layer)?.1)))
            .fold(0, |levels, (layer, finest)| {
                levels | finest << (layer.index() as u32 * FINEST_LEVEL_BITS)
            })
    }

    pub(in crate::renderer) fn statistics(&self) -> TerrainPageStatistics {
        let mut statistics = TerrainPageStatistics::default();

        if let Some(set) = self.heights.as_ref().and_then(StreamedLayer::set) {
            statistics.heights = set.statistics();
        }

        for layer in TerrainLayer::ALL {
            if let Some((set, _)) = self.layer(layer) {
                statistics.layers[layer.index()] = set.statistics();
            }
        }

        statistics
    }
}
