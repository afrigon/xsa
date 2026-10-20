macro_rules! shaders {
    ($($variant:ident => $file:literal),+ $(,)?) => {
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
        }
    };
}

shaders! {
    Lambert => "material/lambert",
    Hapke => "material/hapke",
    Emissive => "material/emissive",
    Normals => "material/debug/normals",
    Depth => "material/debug/depth",
    Triangles => "material/debug/triangles",
    Lighting => "material/debug/lighting",
    Motion => "material/debug/motion",
    Lod => "material/debug/lod",
    Scale => "material/debug/scale",
    TerrainLevels => "material/debug/terrain-levels",
    TerrainFallback => "material/debug/terrain-fallback",
    Skybox => "skybox",
}
