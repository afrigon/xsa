use std::collections::HashMap;

use resource_loader::terrain::PageAddress;
use resource_vulkan::{PageRequest, StreamedSet};

use super::{LayerPageSource, PageDetail, TerrainHeights, TerrainLayer, TerrainPages};
use crate::renderer::gpu_data::{HEIGHT_LEVELS, LAYER_LEVELS, LAYER_PAGES, TileHeightPage, TileLayerPage, TilePages};
use crate::renderer::quad_sphere::{QuadSphereTile, SelectedTile};

// Writes the pages each drawn tile reads, and gathers the pages those tiles need at their distance into each
// terrain's requests: only drawn tiles ask for pages, so terrain out of view loads nothing.
pub(in crate::renderer) struct TilePageWriter {
    // By terrain index.
    requests: Vec<TerrainRequests>,
}

// The pages a terrain's drawn tiles need this frame, each with the highest score a tile gave it.
#[derive(Default)]
struct TerrainRequests {
    heights: HashMap<PageAddress, f32>,
    layers: [HashMap<PageAddress, f32>; TerrainLayer::ALL.len()],
}

impl TilePageWriter {
    pub fn new() -> TilePageWriter {
        TilePageWriter { requests: Vec::new() }
    }

    pub fn begin(&mut self, terrain_count: usize) {
        self.requests.resize_with(terrain_count, TerrainRequests::default);

        for requests in &mut self.requests {
            requests.heights.clear();
            requests.layers.iter_mut().for_each(HashMap::clear);
        }
    }

    pub fn write(
        &mut self,
        terrain: usize,
        pages: &TerrainPages,
        selected: &SelectedTile,
        detail: &PageDetail,
    ) -> TilePages {
        let requests = &mut self.requests[terrain];
        let mut tile_pages = TilePages::NONE;

        if let Some(heights) = pages.heights() {
            tile_pages.heights = TilePageWriter::heights(&heights, selected, detail, &mut requests.heights);
        }

        for layer in TerrainLayer::ALL {
            if let Some((set, finest_level)) = pages.layer(layer) {
                tile_pages.layers[layer.index()] =
                    TilePageWriter::layer(set, finest_level, selected, detail, &mut requests.layers[layer.index()]);
            }
        }

        tile_pages
    }

    // Hands each terrain's sets the pages its tiles need, replacing the previous frame's; `terrains` by index.
    pub fn submit<'a>(&mut self, terrains: impl Iterator<Item = Option<&'a TerrainPages>>) {
        for (pages, requests) in terrains.zip(&mut self.requests) {
            let Some(pages) = pages else {
                continue;
            };

            if let Some(heights) = pages.heights() {
                heights.set.request(TilePageWriter::drain(&mut requests.heights));
            }

            for layer in TerrainLayer::ALL {
                if let Some((set, _)) = pages.layer(layer) {
                    set.request(TilePageWriter::drain(&mut requests.layers[layer.index()]));
                }
            }
        }
    }

    fn heights(
        heights: &TerrainHeights,
        selected: &SelectedTile,
        detail: &PageDetail,
        requests: &mut HashMap<PageAddress, f32>,
    ) -> [TileHeightPage; HEIGHT_LEVELS] {
        let tile = selected.tile;
        let levels = heights.levels(tile.depth);
        let wanted = TilePageWriter::wanted_levels(
            detail.height_level(selected.farthest),
            detail.height_level(selected.nearest),
            levels[0],
            levels[HEIGHT_LEVELS - 1],
        );

        levels.map(|level| {
            let address = tile.page(level);

            if wanted.contains(&level) {
                TilePageWriter::want(requests, address, detail.texel_pixels(level, selected.nearest));
            }

            let Some(placement) = heights.set.page(address) else {
                return TileHeightPage::NONE;
            };
            let Some(range) = heights.ranges.get(placement.page) else {
                return TileHeightPage::NONE;
            };

            TileHeightPage {
                page: TileLayerPage::new(placement),
                minimum: (f64::from(range.minimum) / detail.radius) as f32,
                span: (f64::from(range.maximum - range.minimum) / detail.radius) as f32,
            }
        })
    }

    fn layer(
        set: &StreamedSet<LayerPageSource>,
        finest_level: u32,
        selected: &SelectedTile,
        detail: &PageDetail,
        requests: &mut HashMap<PageAddress, f32>,
    ) -> [TileLayerPage; LAYER_PAGES] {
        let tile = selected.tile;
        let wanted = TilePageWriter::wanted_levels(
            detail.layer_level(selected.farthest),
            detail.layer_level(selected.nearest),
            tile.depth,
            tile.depth + LAYER_LEVELS as u32 - 1,
        );

        for level in wanted {
            let level = level.min(finest_level);
            let score = detail.texel_pixels(level, selected.nearest);

            if level >= tile.depth {
                for address in tile.pages(level) {
                    TilePageWriter::want(requests, address, score);
                }
            } else {
                TilePageWriter::want(requests, tile.page(level), score);
            }
        }

        let mut pages = [TileLayerPage::NONE; LAYER_PAGES];
        let mut slot = 0;

        for finer in 0..LAYER_LEVELS as u32 {
            let side = 1 << finer;

            for index in 0..side * side {
                let address = TilePageWriter::layer_page(tile, finer, index, finest_level);
                pages[slot] = set.page(address).map_or(TileLayerPage::NONE, TileLayerPage::new);
                slot += 1;
            }
        }

        pages
    }

    // The page holding the `index`th part, row by row, of the tile split `finer` levels below its depth, at that
    // level or the layer's finest if coarser.
    fn layer_page(tile: QuadSphereTile, finer: u32, index: u32, finest_level: u32) -> PageAddress {
        let side = 1 << finer;
        let part = QuadSphereTile {
            face: tile.face,
            depth: tile.depth + finer,
            x: (tile.x << finer) + index % side,
            y: (tile.y << finer) + index / side,
        };

        part.page(part.depth.min(finest_level))
    }

    // The whole levels between the coarsest and finest a tile's samples use, within the levels it can read.
    fn wanted_levels(coarsest: f64, finest: f64, lowest: u32, highest: u32) -> std::ops::RangeInclusive<u32> {
        let clamp = |level: f64| (level.max(0.0) as u32).clamp(lowest, highest);
        let first = clamp(coarsest.floor());

        first..=clamp(finest.ceil()).max(first)
    }

    fn want(requests: &mut HashMap<PageAddress, f32>, address: PageAddress, score: f64) {
        let score = score as f32;
        let entry = requests.entry(address).or_insert(score);
        *entry = entry.max(score);
    }

    fn drain(requests: &mut HashMap<PageAddress, f32>) -> Vec<PageRequest<PageAddress>> {
        requests
            .drain()
            .map(|(key, score)| PageRequest { key, score })
            .collect()
    }
}
