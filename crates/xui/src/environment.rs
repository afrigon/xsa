use crate::ColorScheme;

#[derive(Clone, Debug)]
pub struct Environment {
    pub scale_factor: f32,
    pub color_scheme: ColorScheme,
}

impl Default for Environment {
    fn default() -> Environment {
        Environment {
            scale_factor: 1.0,
            color_scheme: ColorScheme::Dark,
        }
    }
}
