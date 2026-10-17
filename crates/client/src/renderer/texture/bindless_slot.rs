// A slot of the bindless tables, by the binding it belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::renderer) enum BindlessSlot {
    Texture(u32),
    Cube(u32),
}

impl BindlessSlot {
    pub fn index(self) -> u32 {
        match self {
            BindlessSlot::Texture(index) | BindlessSlot::Cube(index) => index,
        }
    }
}
