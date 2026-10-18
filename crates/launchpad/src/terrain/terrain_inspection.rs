use std::collections::HashSet;
use std::fmt;
use std::io::{Read, Seek};

use resource_loader::terrain::{
    CubeFace, PAGE_BORDER, PAGE_EXTENT, PAGE_INTERIOR, PageAddress, PageEncoding, PageEntry, PageKey, TerrainHeader,
    TerrainReader,
};

use super::{CoverageMap, LevelSummary, PageCheck, TerrainProblem};

/// What a `.terrain` file holds, level by level, and every defect found in it.
pub struct TerrainInspection {
    pub header: TerrainHeader,
    pub file_size: u64,
    pub levels: Vec<LevelSummary>,
    pub coverage: Vec<CoverageMap>,
    pub problems: Vec<TerrainProblem>,
}

impl TerrainInspection {
    pub fn read<R: Read + Seek>(
        reader: &mut TerrainReader<R>,
        check: PageCheck,
    ) -> anyhow::Result<TerrainInspection> {
        let header = *reader.header();
        let mut inspection = TerrainInspection {
            header,
            file_size: reader.length(),
            levels: Vec::new(),
            coverage: Vec::new(),
            problems: Vec::new(),
        };

        for level in 0..header.level_count {
            let mut summary = LevelSummary::new(level);

            for face in CubeFace::ALL {
                let entries = reader.entries(face, level)?;

                inspection.check_entries(face, level, &entries);

                if check == PageCheck::Pages {
                    inspection.check_pages(reader, face, level, &entries);
                }

                summary.add(&entries);

                let inside: HashSet<PageKey> = entries
                    .iter()
                    .map(|entry| entry.key)
                    .filter(|&key| Self::inside(key, level))
                    .collect();

                if (inside.len() as u64) < LevelSummary::pages_per_face(level) {
                    inspection.coverage.push(CoverageMap::new(face, level, inside));
                }
            }

            inspection.levels.push(summary);
        }

        Ok(inspection)
    }

    fn check_entries(&mut self, face: CubeFace, level: u32, entries: &[PageEntry]) {
        let mut previous: Option<PageKey> = None;

        for entry in entries {
            let address = PageAddress {
                face,
                level,
                key: entry.key,
            };

            if previous.is_some_and(|previous| entry.key <= previous) {
                self.problems.push(TerrainProblem::KeyOutOfOrder { address });
            }

            if !Self::inside(entry.key, level) {
                self.problems.push(TerrainProblem::KeyOutsideLevel { address });
            }

            let end = entry.data_offset.checked_add(u64::from(entry.data_size));

            if end.is_none_or(|end| end > self.file_size) {
                self.problems.push(TerrainProblem::DataOutsideFile {
                    address,
                    end: end.unwrap_or(u64::MAX),
                });
            }

            previous = Some(entry.key);
        }
    }

    fn check_pages<R: Read + Seek>(
        &mut self,
        reader: &mut TerrainReader<R>,
        face: CubeFace,
        level: u32,
        entries: &[PageEntry],
    ) {
        for entry in entries {
            let inside_file = entry
                .data_offset
                .checked_add(u64::from(entry.data_size))
                .is_some_and(|end| end <= self.file_size);

            if !inside_file {
                continue;
            }

            if let Err(error) = reader.page(entry) {
                self.problems.push(TerrainProblem::UnreadablePage {
                    address: PageAddress {
                        face,
                        level,
                        key: entry.key,
                    },
                    reason: format!("{error:#}"),
                });
            }
        }
    }

    fn inside(key: PageKey, level: u32) -> bool {
        let pages_per_side = 1u64 << level;

        u64::from(key.x()) < pages_per_side && u64::from(key.y()) < pages_per_side
    }
}

impl fmt::Display for TerrainInspection {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let header = &self.header;
        let encoding = match header.encoding {
            PageEncoding::Direct => "direct",
            PageEncoding::PageHeights => "page heights",
        };
        writeln!(formatter, "format     {}, {encoding}", header.format)?;
        writeln!(
            formatter,
            "pages      {PAGE_EXTENT}×{PAGE_EXTENT} texels: {PAGE_INTERIOR}×{PAGE_INTERIOR} and a {PAGE_BORDER}-texel border"
        )?;

        match (self.levels.first(), self.levels.last()) {
            (Some(coarsest), Some(finest)) => writeln!(
                formatter,
                "levels     {}, faces of {} to {} texels",
                self.levels.len(),
                coarsest.face_extent(),
                finest.face_extent()
            )?,
            _ => writeln!(formatter, "levels     none")?,
        }

        writeln!(formatter, "size       {}", ByteSize(self.file_size))?;
        writeln!(formatter)?;
        write!(
            formatter,
            "{:>5}  {:>11}  {:>13}  {:>10}  {:>6}",
            "level", "face texels", "pages", "data", "ratio"
        )?;

        if header.encoding == PageEncoding::PageHeights {
            write!(formatter, "  heights (m)")?;
        }

        writeln!(formatter)?;

        for level in &self.levels {
            let ratio = if level.compressed_bytes == 0 {
                "-".to_owned()
            } else {
                let uncompressed = level.present * header.page_size() as u64;
                format!("{:.1}:1", uncompressed as f64 / level.compressed_bytes as f64)
            };
            write!(
                formatter,
                "{:>5}  {:>11}  {:>13}  {:>10}  {ratio:>6}",
                level.level,
                level.face_extent(),
                format!("{}/{}", level.present, level.expected()),
                ByteSize(level.compressed_bytes).to_string()
            )?;

            if let Some(range) = level.range {
                write!(formatter, "  {:.1} to {:.1}", range.minimum, range.maximum)?;
            }

            writeln!(formatter)?;
        }

        for map in &self.coverage {
            writeln!(formatter)?;
            write!(formatter, "{map}")?;
        }

        writeln!(formatter)?;

        match self.problems.len() {
            0 => writeln!(formatter, "no problems found"),
            1 => {
                writeln!(formatter, "1 problem:")?;
                writeln!(formatter, "  {}", self.problems[0])
            }
            count => {
                writeln!(formatter, "{count} problems:")?;

                for problem in &self.problems {
                    writeln!(formatter, "  {problem}")?;
                }

                Ok(())
            }
        }
    }
}

struct ByteSize(u64);

impl fmt::Display for ByteSize {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        const UNITS: [&str; 5] = ["B", "kB", "MB", "GB", "TB"];
        const STEP: f64 = 1000.0;
        let mut value = self.0 as f64;
        let mut unit = 0;

        while value >= STEP && unit < UNITS.len() - 1 {
            value /= STEP;
            unit += 1;
        }

        match unit {
            0 => write!(formatter, "{} B", self.0),
            _ => write!(formatter, "{value:.1} {}", UNITS[unit]),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use resource_loader::terrain::LevelRecord;
    use resource_loader::texture::{HalfFloat, TextureFormat};

    use super::*;
    use crate::terrain::{LayerEncoding, TerrainWriter, TestCubeMap};

    const FACE_EXTENT: u32 = 256;
    const DATA_OFFSET_OFFSET: usize = 8;

    fn converted() -> Vec<u8> {
        let input = TestCubeMap::full_chain(TextureFormat::Rgba16Float, FACE_EXTENT)
            .input(|_, _, _, _| HalfFloat::from_f32(0.5).bits.to_le_bytes().repeat(4))
            .unwrap();
        let encoding = LayerEncoding::new(PageEncoding::Direct, input.format()).unwrap();
        let mut file = Vec::new();
        TerrainWriter::new(&input, encoding).write(&mut file).unwrap();

        file
    }

    fn inspected(file: Vec<u8>, check: PageCheck) -> TerrainInspection {
        TerrainInspection::read(&mut TerrainReader::new(Cursor::new(file)).unwrap(), check).unwrap()
    }

    // The byte offset of a face's entry at level 1.
    fn entry_offset(file: &[u8], face: CubeFace, index: usize) -> usize {
        let record = TerrainHeader::SIZE + (face.index() * 2 + 1) * LevelRecord::SIZE;
        let record = LevelRecord::read(file[record..record + LevelRecord::SIZE].try_into().unwrap());

        record.entries_offset as usize + index * PageEntry::size(PageEncoding::Direct)
    }

    fn address(face: CubeFace, x: u32, y: u32) -> PageAddress {
        PageAddress {
            face,
            level: 1,
            key: PageKey::from_coordinates(x, y),
        }
    }

    #[test]
    fn a_converted_file_is_complete_and_sound() {
        let inspection = inspected(converted(), PageCheck::Pages);

        assert_eq!(inspection.levels.len(), 2);
        assert!(inspection.levels.iter().all(|level| level.present == level.expected()));
        assert!(inspection.coverage.is_empty());
        assert!(inspection.problems.is_empty());
        assert!(inspection.to_string().contains("    1          256          24/24"));
    }

    #[test]
    fn every_defect_of_a_broken_index_is_reported() {
        let mut file = converted();
        let entry_size = PageEntry::size(PageEncoding::Direct);
        let first = entry_offset(&file, CubeFace::PositiveX, 0);
        let (head, tail) = file[first..first + 2 * entry_size].split_at_mut(entry_size);
        head.swap_with_slice(tail);

        let outside = entry_offset(&file, CubeFace::NegativeX, 3);
        file[outside..outside + size_of::<u64>()].copy_from_slice(&PageKey::from_coordinates(2, 1).code.to_le_bytes());

        let past_the_end = entry_offset(&file, CubeFace::PositiveZ, 0);
        let length = file.len() as u64;
        file[past_the_end + DATA_OFFSET_OFFSET..past_the_end + DATA_OFFSET_OFFSET + size_of::<u64>()]
            .copy_from_slice(&length.to_le_bytes());

        let inspection = inspected(file, PageCheck::Index);

        assert_eq!(
            inspection.problems,
            [
                TerrainProblem::KeyOutOfOrder {
                    address: address(CubeFace::PositiveX, 0, 0)
                },
                TerrainProblem::KeyOutsideLevel {
                    address: address(CubeFace::NegativeX, 2, 1)
                },
                TerrainProblem::DataOutsideFile {
                    address: address(CubeFace::PositiveZ, 0, 0),
                    end: length + u64::from(PageEntry::read(&converted()[past_the_end..], PageEncoding::Direct).data_size),
                },
            ]
        );
        assert_eq!(inspection.coverage.len(), 1);
        assert_eq!(inspection.coverage[0].face, CubeFace::NegativeX);
    }

    #[test]
    fn a_corrupted_page_is_reported_only_when_verified() {
        let mut file = converted();
        let corrupted = entry_offset(&file, CubeFace::PositiveY, 2);
        let entry = PageEntry::read(&file[corrupted..], PageEncoding::Direct);
        let data = entry.data_offset as usize;
        file[data..data + entry.data_size as usize].fill(0xFF);

        assert!(inspected(file.clone(), PageCheck::Index).problems.is_empty());

        let problems = inspected(file, PageCheck::Pages).problems;
        assert!(matches!(
            problems.as_slice(),
            [TerrainProblem::UnreadablePage { address: found, .. }]
                if *found == address(CubeFace::PositiveY, entry.key.x(), entry.key.y())
        ));
    }
}
