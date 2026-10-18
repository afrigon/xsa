mod terrain_create_options;
mod terrain_inspect_options;

use std::fs::File;
use std::io::{BufReader, BufWriter, Write};

use anyhow::{Context, bail};
use resource_loader::terrain::TerrainReader;
pub use terrain_create_options::TerrainCreateOptions;
pub use terrain_inspect_options::TerrainInspectOptions;

use crate::launchpad_context::LaunchpadContext;
use crate::terrain::{CubeMapInput, LayerEncoding, PageInspection, TerrainInspection, TerrainWriter};

/// Converts a terrain layer's cube map into its `.terrain` file.
pub fn create(context: &mut LaunchpadContext, options: &TerrainCreateOptions) -> anyhow::Result<()> {
    context.shell().status("Reading", options.input.display())?;
    let input = CubeMapInput::read(&options.input)?;
    let encoding =
        LayerEncoding::new(options.encoding, input.format()).with_context(|| format!("{}", options.input.display()))?;
    let writer = TerrainWriter::new(&input, encoding);
    context.shell().status(
        "Converting",
        format!(
            "{}-texel {} faces into {} levels, {} {} pages",
            input.face_extent(),
            encoding.input_format(),
            input.level_count(),
            writer.page_count(),
            encoding.page_format()
        ),
    )?;

    let file = File::create(&options.output).with_context(|| format!("creating {}", options.output.display()))?;
    let mut output = BufWriter::new(file);
    writer
        .write(&mut output)
        .and_then(|()| Ok(output.flush()?))
        .with_context(|| format!("writing {}", options.output.display()))?;
    context.shell().status("Created", options.output.display())?;

    Ok(())
}

/// Prints what a `.terrain` file holds, or one of its pages, and fails when the checks find problems.
pub fn inspect(context: &mut LaunchpadContext, options: &TerrainInspectOptions) -> anyhow::Result<()> {
    let file = File::open(&options.file).with_context(|| format!("opening {}", options.file.display()))?;
    let mut reader = TerrainReader::new(BufReader::new(file)).with_context(|| format!("{}", options.file.display()))?;

    if let Some(address) = options.page {
        let page = PageInspection::read(&mut reader, address, options.check)
            .with_context(|| format!("{}", options.file.display()))?;
        write!(context.shell().out(), "{page}")?;

        return Ok(());
    }

    let inspection =
        TerrainInspection::read(&mut reader, options.check).with_context(|| format!("{}", options.file.display()))?;
    write!(context.shell().out(), "{inspection}")?;

    match inspection.problems.len() {
        0 => Ok(()),
        1 => bail!("{}: 1 problem", options.file.display()),
        count => bail!("{}: {count} problems", options.file.display()),
    }
}
