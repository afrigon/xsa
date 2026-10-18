use std::fmt;
use std::io::{Read, Seek};

use anyhow::Context;
use resource_loader::terrain::{HeightRange, PageAddress, PageEntry, TerrainReader};

use super::PageCheck;

/// One page of a `.terrain` file: its entry and, when checked, what its data decodes to.
pub struct PageInspection {
    pub address: PageAddress,
    pub entry: PageEntry,
    decoded: Option<DecodedPage>,
}

struct DecodedPage {
    size: usize,
    heights: Option<DecodedHeights>,
}

struct DecodedHeights {
    range: HeightRange,
    mean: f64,
}

impl PageInspection {
    pub fn read<R: Read + Seek>(
        reader: &mut TerrainReader<R>,
        address: PageAddress,
        check: PageCheck,
    ) -> anyhow::Result<PageInspection> {
        let location = format!(
            "{} level {} page {},{}",
            address.face,
            address.level,
            address.key.x(),
            address.key.y()
        );
        let entry = reader
            .entry(address)
            .with_context(|| location.clone())?
            .with_context(|| format!("{location}: the file holds no such page"))?;
        let decoded = match check {
            PageCheck::Index => None,
            PageCheck::Pages => {
                let data = reader.page(&entry).with_context(|| location.clone())?;
                let heights = entry.range.map(|range| {
                    let heights: Vec<f32> = data
                        .as_chunks::<{ size_of::<u16>() }>()
                        .0
                        .iter()
                        .map(|value| range.height(u16::from_le_bytes(*value)))
                        .collect();
                    let sum: f64 = heights.iter().map(|&height| f64::from(height)).sum();

                    DecodedHeights {
                        range: HeightRange::spanning(heights.iter().copied()).unwrap(),
                        mean: sum / heights.len() as f64,
                    }
                });

                Some(DecodedPage {
                    size: data.len(),
                    heights,
                })
            }
        };

        Ok(PageInspection {
            address,
            entry,
            decoded,
        })
    }
}

impl fmt::Display for PageInspection {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let address = &self.address;
        writeln!(
            formatter,
            "page       {} level {} page {},{}",
            address.face,
            address.level,
            address.key.x(),
            address.key.y()
        )?;
        writeln!(formatter, "key        {:#x}", self.entry.key.code)?;
        writeln!(
            formatter,
            "data       {} bytes at offset {}",
            self.entry.data_size, self.entry.data_offset
        )?;

        if let Some(range) = self.entry.range {
            writeln!(formatter, "range      {} to {} m", range.minimum, range.maximum)?;
        }

        if let Some(decoded) = &self.decoded {
            writeln!(formatter, "decoded    {} bytes", decoded.size)?;

            if let Some(heights) = &decoded.heights {
                writeln!(
                    formatter,
                    "heights    {} to {} m, mean {:.3} m",
                    heights.range.minimum, heights.range.maximum, heights.mean
                )?;
            }
        }

        Ok(())
    }
}
