use crate::renderer::GeometryFamily;

macro_rules! shaders {
    ($($variant:ident => $file:literal in $family:ident),+ $(,)?) => {
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub enum Shader {
            $($variant),+
        }

        impl Shader {
            pub const ALL: &[Shader] = &[$(Shader::$variant),+];

            pub fn path(self) -> &'static str {
                match self {
                    $(Shader::$variant => $file),+
                }
            }

            pub fn family(self) -> GeometryFamily {
                match self {
                    $(Shader::$variant => GeometryFamily::$family),+
                }
            }
        }
    };
}

shaders! {
    TerrainLambert => "terrain/lambert" in Terrain,
    TerrainHapke => "terrain/hapke" in Terrain,
    TerrainEmissive => "terrain/emissive" in Terrain,
    Hapke => "hapke" in Mesh,
    Skybox => "skybox" in Sky,
}
