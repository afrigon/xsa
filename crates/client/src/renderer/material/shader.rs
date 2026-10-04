macro_rules! shaders {
    ($($variant:ident => $file:literal),+ $(,)?) => {
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub enum Shader {
            $($variant),+
        }

        impl Shader {
            pub const ALL: &[Shader] = &[$(Shader::$variant),+];

            pub fn index(self) -> usize {
                self as usize
            }

            pub fn path(self) -> &'static str {
                match self {
                    $(Shader::$variant => $file),+
                }
            }
        }
    };
}

shaders! {
    Lit => "lit",
    Planet => "planet",
    Emissive => "emissive",
    Normals => "normals",
    Depth => "depth",
    Triangles => "triangles",
    Lighting => "lighting",
    Skybox => "skybox",
}
