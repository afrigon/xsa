use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use glam::DVec3;
use resource_loader::texture::ColorSpace;
use resource_vulkan::{CubeMap, LoadRequest, MipDemand, MipHandle, ResourceGroup, Resources, StreamedMips};
use xsa_packs::HeightMapDefinition;

use super::{TerrainLayer, TexelFootprint};
use crate::renderer::quad_sphere::DetailRanges;
use crate::resource_kinds::{HeightMap, HeightMapOptions};

// How many levels finer than the camera needs a map keeps resident, loading each before it is read.
const LEVELS_AHEAD: u32 = 1;

// A terrain's maps: its heights and the layers its material shades with, each a cube map whose resident levels follow
// how close the camera is. Shared by the renderer, which states their demands and draws them, and the world, which
// reads the ground from the heights.
pub struct TerrainMaps {
    heights: Option<MipHandle<HeightMap>>,
    layers: [Option<MipHandle<CubeMap>>; TerrainLayer::ALL.len()],
}

impl TerrainMaps {
    // `layers` by `TerrainLayer` index.
    pub fn new(
        resources: &Resources,
        group: &mut ResourceGroup,
        body: &str,
        heights: Option<&HeightMapDefinition>,
        layers: [Option<PathBuf>; TerrainLayer::ALL.len()],
    ) -> TerrainMaps {
        let heights = heights.map(|definition| {
            let options = HeightMapOptions {
                minimum: definition.minimum,
                maximum: definition.maximum,
            };
            let handle = resources.load_mips(LoadRequest::new(&definition.path, format!("{body} heights")), options);
            group.add_mips(&handle);
            handle
        });
        let mut paths = layers.into_iter();
        let layers = TerrainLayer::ALL.map(|layer| {
            let path = paths.next().flatten()?;
            let color_space = match layer {
                TerrainLayer::Color => ColorSpace::Srgb,
                TerrainLayer::Normal | TerrainLayer::Scattering | TerrainLayer::Surge => ColorSpace::Linear,
            };
            let handle = resources.load_mips(LoadRequest::new(path, format!("{body} {}", layer.name())), color_space);
            group.add_mips(&handle);

            Some(handle)
        });

        TerrainMaps { heights, layers }
    }

    // Meters relative to the radius toward `direction`, a unit vector in the body's frame; None until the heights
    // load.
    pub fn height_at(&self, direction: DVec3) -> Option<f64> {
        Some(self.heights()?.level().height_at(direction))
    }

    // Whether the ground is as fine as the camera needs: true for a terrain without heights.
    pub fn is_ground_settled(&self) -> bool {
        self.heights.as_ref().is_none_or(MipHandle::is_settled)
    }

    // None for a smooth terrain, and until its heights load.
    pub(in crate::renderer) fn heights(&self) -> Option<Arc<HeightMap>> {
        self.heights.as_ref()?.get()
    }

    // None without the layer, and until it loads.
    pub(in crate::renderer) fn layer(&self, layer: TerrainLayer) -> Option<Arc<CubeMap>> {
        self.layers[layer.index()].as_ref()?.get()
    }

    // The levels a camera `distance` meters from the nearest ground needs, one level finer still, so a level is
    // resident before the ground nearing the camera reads it and every transition stays a morph: the height level of
    // the finest detail drawn there, and layers whose texels are a pixel wide. A map's first levels load before it has
    // a demand, since its size is unknown until then.
    pub(in crate::renderer) fn demand(
        &self,
        distance: f64,
        ranges: &DetailRanges,
        footprint: &TexelFootprint,
        retention: Duration,
    ) {
        let demand = |level: f64| MipDemand {
            finest_level: (level.floor() - f64::from(LEVELS_AHEAD)).max(0.0) as u32,
            retention,
        };

        if let Some(handle) = &self.heights
            && let Some(heights) = handle.get()
        {
            let detail = ranges.finest_detail(distance);
            handle.set_demand(demand(f64::from(DetailRanges::height_level(detail, heights.extent()))));
        }

        for handle in self.layers.iter().flatten() {
            if let Some(layer) = handle.get() {
                handle.set_demand(demand(
                    footprint.layer_level(TerrainMaps::extent(layer.as_ref()), distance),
                ));
            }
        }
    }

    // The width of a cube map's faces at its level 0.
    pub(in crate::renderer) fn extent<K: StreamedMips>(map: &K) -> u32 {
        map.image().extent().width << map.first_level()
    }
}
