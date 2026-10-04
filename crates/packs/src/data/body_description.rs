use anyhow::{anyhow, ensure};
use xsa_core::simulation::{BodyCategory, BodyDescription, Star};

use crate::{Document, NodeExtension, PackData, ParseContext};

impl PackData for BodyDescription {
    const KIND: &'static str = "bodies";

    fn parse(document: &Document, context: &ParseContext) -> anyhow::Result<BodyDescription> {
        let category: BodyCategory = document
            .node("category")?
            .string_argument()?
            .parse()
            .map_err(|err: String| anyhow!(err))?;
        let star = parse_star(document)?;

        Ok(BodyDescription {
            id: context.id.into(),
            category,
            radius: document.node("radius")?.number_argument()?,
            gravitational_parameter: document.node("gravitational-parameter")?.number_argument()?,
            rotation_period: document.node("rotation-period")?.number_argument()?,
            axial_tilt: document.node("axial-tilt")?.number_argument()?.to_radians(),
            star,
        })
    }
}

fn parse_star(document: &Document) -> anyhow::Result<Option<Star>> {
    let luminosity = document.optional_node("luminosity");
    let temperature = document.optional_node("effective-temperature");
    ensure!(
        luminosity.is_some() == temperature.is_some(),
        "a star needs both `luminosity` and `effective-temperature`"
    );

    let Some(luminosity) = luminosity else {
        return Ok(None);
    };
    let Some(temperature) = temperature else {
        return Ok(None);
    };

    Ok(Some(Star {
        luminosity: luminosity.number_argument()?,
        effective_temperature: temperature.number_argument()?,
    }))
}
