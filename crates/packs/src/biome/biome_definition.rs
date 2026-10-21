use crate::{Document, PackData, ParseContext};

// What the server knows of a biome. It holds nothing beyond the biome existing until scatters (#46) and terrain detail
// (#236) give it content.
pub struct BiomeDefinition;

impl PackData for BiomeDefinition {
    const KIND: &'static str = "biomes";

    fn parse(_document: &Document, _context: &ParseContext) -> anyhow::Result<BiomeDefinition> {
        Ok(BiomeDefinition)
    }
}
