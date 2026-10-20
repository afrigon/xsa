use super::{
    AntialiasingDocument, BloomDocument, ConfigChoice, ConfigKey, ConfigLayer, ConfigValueKind, ConfigValues,
    LensDocument, TerrainLodDocument, WhiteBalanceDocument,
};
use crate::config::RenderConfig;
use crate::renderer::{DebugView, SurfaceDebugView, SurfaceTiling, TerrainDebugView, Tonemapper};

pub struct RenderDocument {
    tonemapper: Option<Option<Tonemapper>>,
    stars: Option<bool>,
    bloom: BloomDocument,
    lens: LensDocument,
    white_balance: WhiteBalanceDocument,
    antialiasing: AntialiasingDocument,
    wireframe: Option<bool>,
    shader_override: Option<Option<DebugView>>,
    terrain_shader_override: Option<Option<TerrainDebugView>>,
    surface_shader_override: Option<Option<SurfaceDebugView>>,
    surface_tiling: Option<SurfaceTiling>,
    terrain_lod: TerrainLodDocument,
}

impl RenderDocument {
    pub const TONEMAPPER: ConfigKey = ConfigKey::saved(
        "render.tonemapper",
        ConfigValueKind::Choice {
            names: <Option<Tonemapper> as ConfigChoice>::names,
        },
    );
    pub const STARS: ConfigKey = ConfigKey::saved("render.stars", ConfigValueKind::Bool);
    pub const WIREFRAME: ConfigKey = ConfigKey::session("render.wireframe", ConfigValueKind::Bool);
    pub const SHADER_OVERRIDE: ConfigKey = ConfigKey::session(
        "render.shader.override",
        ConfigValueKind::Choice {
            names: <Option<DebugView> as ConfigChoice>::names,
        },
    );
    pub const TERRAIN_SHADER_OVERRIDE: ConfigKey = ConfigKey::session(
        "render.shader.terrain.override",
        ConfigValueKind::Choice {
            names: <Option<TerrainDebugView> as ConfigChoice>::names,
        },
    );
    pub const SURFACE_SHADER_OVERRIDE: ConfigKey = ConfigKey::session(
        "render.shader.surface.override",
        ConfigValueKind::Choice {
            names: <Option<SurfaceDebugView> as ConfigChoice>::names,
        },
    );

    pub const SURFACE_TILING: ConfigKey = ConfigKey::session(
        "render.surface.tiling",
        ConfigValueKind::Choice {
            names: <SurfaceTiling as ConfigChoice>::names,
        },
    );

    pub fn read(values: &ConfigValues) -> RenderDocument {
        RenderDocument {
            tonemapper: values.choice(RenderDocument::TONEMAPPER),
            stars: values.bool(RenderDocument::STARS),
            bloom: BloomDocument::read(values),
            lens: LensDocument::read(values),
            white_balance: WhiteBalanceDocument::read(values),
            antialiasing: AntialiasingDocument::read(values),
            wireframe: values.bool(RenderDocument::WIREFRAME),
            shader_override: values.choice(RenderDocument::SHADER_OVERRIDE),
            terrain_shader_override: values.choice(RenderDocument::TERRAIN_SHADER_OVERRIDE),
            surface_shader_override: values.choice(RenderDocument::SURFACE_SHADER_OVERRIDE),
            surface_tiling: values.choice(RenderDocument::SURFACE_TILING),
            terrain_lod: TerrainLodDocument::read(values),
        }
    }

    pub fn into_config(self, defaults: &RenderConfig) -> RenderConfig {
        RenderConfig {
            tonemapper: self.tonemapper.unwrap_or(defaults.tonemapper),
            stars: self.stars.unwrap_or(defaults.stars),
            bloom: self.bloom.into_config(&defaults.bloom),
            lens: self.lens.into_config(&defaults.lens),
            white_balance: self.white_balance.into_config(&defaults.white_balance),
            antialiasing: self.antialiasing.into_config(&defaults.antialiasing),
            wireframe: self.wireframe.unwrap_or(defaults.wireframe),
            shader_override: self.shader_override.unwrap_or(defaults.shader_override),
            terrain_shader_override: self.terrain_shader_override.unwrap_or(defaults.terrain_shader_override),
            surface_shader_override: self.surface_shader_override.unwrap_or(defaults.surface_shader_override),
            surface_tiling: self.surface_tiling.unwrap_or(defaults.surface_tiling),
            terrain_lod: self.terrain_lod.into_config(&defaults.terrain_lod),
        }
    }

    pub fn write(config: &RenderConfig, layer: &mut ConfigLayer) {
        layer.set(RenderDocument::TONEMAPPER.path, config.tonemapper.config_name().into());
        layer.set(RenderDocument::STARS.path, config.stars.into());
        BloomDocument::write(&config.bloom, layer);
        LensDocument::write(&config.lens, layer);
        WhiteBalanceDocument::write(&config.white_balance, layer);
        AntialiasingDocument::write(&config.antialiasing, layer);
        layer.set(RenderDocument::WIREFRAME.path, config.wireframe.into());
        layer.set(
            RenderDocument::SHADER_OVERRIDE.path,
            config.shader_override.config_name().into(),
        );
        layer.set(
            RenderDocument::TERRAIN_SHADER_OVERRIDE.path,
            config.terrain_shader_override.config_name().into(),
        );
        layer.set(
            RenderDocument::SURFACE_SHADER_OVERRIDE.path,
            config.surface_shader_override.config_name().into(),
        );
        layer.set(
            RenderDocument::SURFACE_TILING.path,
            config.surface_tiling.config_name().into(),
        );
        TerrainLodDocument::write(&config.terrain_lod, layer);
    }
}
