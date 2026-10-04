use xui::{LinearColor, Primitive, Rect};

// Must match PrimitiveInstance in shaders/xui.slang (std430: 64-byte stride).
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct PrimitiveInstance {
    bounds: [f32; 4],
    atlas_region: [f32; 4],
    color: [f32; 4],
    kind: u32,
    padding: [u32; 3],
}

const _: () = assert!(size_of::<PrimitiveInstance>() == 64);

impl PrimitiveInstance {
    // Must match primitiveGlyph in shaders/xui.slang.
    const GLYPH: u32 = 0;

    pub fn new(primitive: &Primitive) -> PrimitiveInstance {
        match primitive {
            Primitive::Glyph(glyph) => PrimitiveInstance {
                bounds: PrimitiveInstance::rect(glyph.bounds),
                atlas_region: [
                    glyph.atlas_region.x as f32,
                    glyph.atlas_region.y as f32,
                    glyph.atlas_region.width as f32,
                    glyph.atlas_region.height as f32,
                ],
                color: PrimitiveInstance::color(glyph.color),
                kind: PrimitiveInstance::GLYPH,
                padding: [0; 3],
            },
        }
    }

    pub fn bytes(instances: &[PrimitiveInstance]) -> &[u8] {
        unsafe { std::slice::from_raw_parts(instances.as_ptr().cast::<u8>(), size_of_val(instances)) }
    }

    fn rect(rect: Rect) -> [f32; 4] {
        [rect.origin.x, rect.origin.y, rect.size.width, rect.size.height]
    }

    fn color(color: LinearColor) -> [f32; 4] {
        [color.red, color.green, color.blue, color.alpha]
    }
}
