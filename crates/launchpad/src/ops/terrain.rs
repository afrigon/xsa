use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use anyhow::Context;
use resource_loader::terrain::PageEncoding;

use crate::launchpad_context::LaunchpadContext;
use crate::terrain::{CubeMapInput, LayerEncoding, TerrainWriter};

pub struct TerrainCreateOptions {
    pub input: PathBuf,
    pub output: PathBuf,
    pub encoding: PageEncoding,
}

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
            "{}-texel {:?} faces into {} levels, {} {:?} pages",
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
