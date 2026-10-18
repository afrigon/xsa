use std::path::PathBuf;

use resource_loader::terrain::PageEncoding;
use usage::{Args, Subcommands};

use crate::cli::CommandHandler;
use crate::launchpad_context::LaunchpadContext;
use crate::ops;
use crate::ops::terrain::TerrainCreateOptions;

#[derive(Args)]
pub struct TerrainCommand {
    #[usage(subcommand)]
    pub command: TerrainSubcommand,
}

#[derive(Subcommands)]
pub enum TerrainSubcommand {
    /// Convert a terrain layer's cube map into the .terrain file the game streams
    ///
    /// Each terrain layer (heights, colour, normals, each Hapke map) is authored as a
    /// KTX2 cube map:
    ///
    /// - Six square faces, in Vulkan's cube-map order: +X, -X, +Y, -Y, +Z, -Z of the
    ///     body's frame, where +X points at longitude 0 on the equator and +Z at the
    ///     north pole.
    /// - Each face oriented as Vulkan samples it, its columns running left to right
    ///     and its rows top to bottom along:
    ///       +X  columns toward -Z, rows toward -Y
    ///       -X  columns toward +Z, rows toward -Y
    ///       +Y  columns toward +X, rows toward +Z
    ///       -Y  columns toward +X, rows toward -Z
    ///       +Z  columns toward +X, rows toward -Y
    ///       -Z  columns toward -X, rows toward -Y
    /// - A face size that is a power of two, at least 128 texels: 128, 256, 512, ...
    /// - The full mip chain down to 1x1, each level the average of the one above.
    /// - Uncompressed texels, in one of these formats:
    ///       R32_SFLOAT           heights, with --height: metres relative to the
    ///                            body's radius, signed
    ///       R8G8B8A8_SRGB        colour, stored as BC7_SRGB_BLOCK
    ///       R8G8_UNORM           normals, stored as BC5_UNORM_BLOCK
    ///       R16G16B16A16_SFLOAT  Hapke maps, stored as is
    ///     This command compresses each page itself: page borders are rotated across
    ///     face edges, and compressed blocks cannot be.
    ///
    /// The KTX2 file may use Zstandard supercompression. Without --output, the
    /// .terrain file is written next to the input, with the same name.
    #[usage(verbatim_doc_comment)]
    Create(TerrainCreateCommand),
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

impl CommandHandler for TerrainCommand {
    async fn handle(&self, context: &mut LaunchpadContext) -> anyhow::Result<()> {
        match &self.command {
            TerrainSubcommand::Create(command) => command.handle(context).await,
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
