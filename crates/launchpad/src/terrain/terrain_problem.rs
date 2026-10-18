use std::fmt;

use resource_loader::terrain::PageAddress;

/// A defect `terrain inspect` found in a `.terrain` file.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TerrainProblem {
    /// An entry's key is not greater than the one before it: out of order or repeated.
    KeyOutOfOrder { address: PageAddress },
    /// An entry's column or row is beyond its level's pages.
    KeyOutsideLevel { address: PageAddress },
    /// A page's data extends past the end of the file.
    DataOutsideFile { address: PageAddress, end: u64 },
    /// A page's data does not decompress to a page.
    UnreadablePage { address: PageAddress, reason: String },
}

impl fmt::Display for TerrainProblem {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TerrainProblem::KeyOutOfOrder { address } => {
                write!(formatter, "{}: out of order or repeated", Location(address))
            }
            TerrainProblem::KeyOutsideLevel { address } => {
                write!(
                    formatter,
                    "{}: outside the level's {} pages per side",
                    Location(address),
                    1u64 << address.level
                )
            }
            TerrainProblem::DataOutsideFile { address, end } => {
                write!(
                    formatter,
                    "{}: data ends at byte {end}, past the end of the file",
                    Location(address)
                )
            }
            TerrainProblem::UnreadablePage { address, reason } => write!(formatter, "{}: {reason}", Location(address)),
        }
    }
}

struct Location<'a>(&'a PageAddress);

impl fmt::Display for Location<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let address = self.0;

        write!(
            formatter,
            "{} level {} page {},{}",
            address.face,
            address.level,
            address.key.x(),
            address.key.y()
        )
    }
}
