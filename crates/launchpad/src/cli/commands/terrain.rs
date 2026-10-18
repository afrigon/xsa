use std::path::PathBuf;

use resource_loader::terrain::{PageAddress, PageEncoding, PageKey};
use resource_loader::utils::CubeFace;
use usage::{Args, Subcommands};

use crate::cli::CommandHandler;
use crate::launchpad_context::LaunchpadContext;
use crate::ops;
use crate::ops::terrain::{TerrainCreateOptions, TerrainExtractOptions, TerrainInspectOptions};
use crate::terrain::{LevelSelection, PageCheck, TerrainRegion};

#[derive(Args)]
pub struct TerrainCommand {
    #[usage(subcommand)]
    pub command: TerrainSubcommand,
}

#[derive(Subcommands)]
pub enum TerrainSubcommand {
    /// Convert a terrain layer's cube map into the .terrain file the game streams
    #[usage(
        long_help = r#"Convert a terrain layer's cube map into the .terrain file the game streams

Each terrain layer (heights, colour, normals, each Hapke map) is authored as a
KTX2 cube map:

- Six square faces, in Vulkan's cube-map order: +X, -X, +Y, -Y, +Z, -Z of the
    body's frame, where +X points at longitude 0 on the equator and +Z at the
    north pole.
- Each face oriented as Vulkan samples it, its columns running left to right
    and its rows top to bottom along:
      +X  columns toward -Z, rows toward -Y
      -X  columns toward +Z, rows toward -Y
      +Y  columns toward +X, rows toward +Z
      -Y  columns toward +X, rows toward -Z
      +Z  columns toward +X, rows toward -Y
      -Z  columns toward -X, rows toward -Y
- A face size that is a power of two, at least 128 texels: 128, 256, 512, ...
- The full mip chain down to 1x1, each level the average of the one above.
- Uncompressed texels, in one of these formats:
      R32_SFLOAT           heights, with --height: metres relative to the
                           body's radius, signed
      R8G8B8A8_SRGB        colour, stored as BC7_SRGB_BLOCK
      R8G8_UNORM           normals, stored as BC5_UNORM_BLOCK
      R16G16B16A16_SFLOAT  Hapke maps, stored as is
    This command compresses each page itself: page borders are rotated across
    face edges, and compressed blocks cannot be.

The KTX2 file may use Zstandard supercompression. Without --output, the
.terrain file is written next to the input, with the same name."#
    )]
    Create(TerrainCreateCommand),
    /// Print a .terrain file's header, levels and page coverage, and check its index
    #[usage(
        long_help = r#"Print a .terrain file's header, levels and page coverage, and check its index

The summary lists each level over its six faces: its face size, the pages
present and expected, their compressed size and compression ratio and, for
heights, the range their pages span. A level missing pages gets a map of each
of its faces.

The index is always checked: every face and level's keys sorted, unique and
inside their level, and every page's data inside the file. --verify also
decompresses every page. The command fails when a check finds a problem.

--face, --level and --page together print one page's entry instead."#
    )]
    Inspect(TerrainInspectCommand),
    /// Write a .terrain file, or part of it, back out as a KTX2 texture in its cube map's texels
    #[usage(
        long_help = r#"Write a .terrain file, or part of it, back out as a KTX2 texture in its cube map's texels

Pages are decoded back to the texel format terrain create reads: heights to
R32_SFLOAT metres, BC7_SRGB_BLOCK to R8G8B8A8_SRGB, BC5_UNORM_BLOCK to
R8G8_UNORM, R16G16B16A16_SFLOAT as is. Faces are stitched from their pages'
interiors; absent pages are transparent, NaN heights, or 0 for R8G8_UNORM.

    (no options)             a cube map, every level as a mip level, finest
                             first, down to 128 texels
    --level                  a cube map of that level
    --face                   that face as a 2D texture, every level as a mip
                             level
    --face --level           that face at that level
    --face --level --page    that page, 136x136 texels with its border

The extract stops at 128 texels, so terrain create does not take it back;
compare it with the cube map level by level. Without --output, it is written
next to the file as <name>.extract.ktx2."#
    )]
    Extract(TerrainExtractCommand),
}

#[derive(Args)]
pub struct TerrainCreateCommand {
    #[usage(
        long,
        help = "The input holds heights: store them as 16 bits over each page's own range"
    )]
    height: bool,
    #[usage(
        long,
        help = "Where to write the .terrain file; defaults to the input's path with a .terrain extension"
    )]
    output: Option<PathBuf>,
    #[usage(arg, help = "The layer's KTX2 cube map")]
    input: PathBuf,
}

#[derive(Args)]
pub struct TerrainInspectCommand {
    #[usage(long, help = "Also decompress every page, checking it decodes to a whole page")]
    verify: bool,
    #[usage(
        long,
        allow_hyphen_values,
        requires("--level", "--page"),
        help = "The face of the page to print: +x, -x, +y, -y, +z or -z"
    )]
    face: Option<CubeFace>,
    #[usage(long, requires("--face", "--page"), help = "The level of the page to print, from 0")]
    level: Option<u32>,
    #[usage(
        long,
        value_name = "x,y",
        requires("--face", "--level"),
        help = "The column and row of the page to print, from 0 at the face's top left"
    )]
    page: Option<PageKey>,
    #[usage(arg, help = "The .terrain file")]
    file: PathBuf,
}

#[derive(Args)]
pub struct TerrainExtractCommand {
    #[usage(
        long,
        allow_hyphen_values,
        help = "Extract only this face, as a 2D texture: +x, -x, +y, -y, +z or -z"
    )]
    face: Option<CubeFace>,
    #[usage(long, help = "Extract only this level, from 0")]
    level: Option<u32>,
    #[usage(
        long,
        value_name = "x,y",
        requires("--face", "--level"),
        help = "Extract only the page at this column and row, from 0 at the face's top left"
    )]
    page: Option<PageKey>,
    #[usage(
        long,
        help = "Where to write the KTX2 file; defaults to the file's path with a .extract.ktx2 extension"
    )]
    output: Option<PathBuf>,
    #[usage(arg, help = "The .terrain file")]
    file: PathBuf,
}

impl CommandHandler for TerrainCommand {
    async fn handle(&self, context: &mut LaunchpadContext) -> anyhow::Result<()> {
        match &self.command {
            TerrainSubcommand::Create(command) => command.handle(context).await,
            TerrainSubcommand::Inspect(command) => command.handle(context).await,
            TerrainSubcommand::Extract(command) => command.handle(context).await,
        }
    }
}

impl CommandHandler for TerrainCreateCommand {
    async fn handle(&self, context: &mut LaunchpadContext) -> anyhow::Result<()> {
        let options = TerrainCreateOptions {
            input: self.input.clone(),
            output: self
                .output
                .clone()
                .unwrap_or_else(|| self.input.with_extension("terrain")),
            encoding: if self.height {
                PageEncoding::PageHeights
            } else {
                PageEncoding::Direct
            },
        };

        ops::terrain::create(context, &options)
    }
}

impl CommandHandler for TerrainInspectCommand {
    async fn handle(&self, context: &mut LaunchpadContext) -> anyhow::Result<()> {
        let page = match (self.face, self.level, self.page) {
            (Some(face), Some(level), Some(key)) => Some(PageAddress { face, level, key }),
            _ => None,
        };
        let options = TerrainInspectOptions {
            file: self.file.clone(),
            check: if self.verify {
                PageCheck::Pages
            } else {
                PageCheck::Index
            },
            page,
        };

        ops::terrain::inspect(context, &options)
    }
}

impl CommandHandler for TerrainExtractCommand {
    async fn handle(&self, context: &mut LaunchpadContext) -> anyhow::Result<()> {
        let levels = match self.level {
            Some(level) => LevelSelection::One(level),
            None => LevelSelection::Every,
        };
        let region = match (self.face, self.level, self.page) {
            (Some(face), Some(level), Some(key)) => TerrainRegion::Page(PageAddress { face, level, key }),
            (Some(face), _, _) => TerrainRegion::Face(face, levels),
            (None, _, _) => TerrainRegion::Cube(levels),
        };
        let options = TerrainExtractOptions {
            file: self.file.clone(),
            output: self
                .output
                .clone()
                .unwrap_or_else(|| self.file.with_extension("extract.ktx2")),
            region,
        };

        ops::terrain::extract(context, &options)
    }
}
