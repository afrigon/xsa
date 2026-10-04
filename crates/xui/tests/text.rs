use std::path::PathBuf;

use xui::{Alignment, EdgeInsets, Environment, Font, Interface, Primitive, Size, Text, View};

const VIEWPORT: Size = Size {
    width: 800.0,
    height: 600.0,
};
const TOP_PADDING: f32 = 24.0;

fn interface() -> (Interface, String) {
    let font =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/base/base/resources/fonts/inter/inter-variable.ttf");
    let mut interface = Interface::new();
    let families = interface.fonts_mut().register(std::fs::read(font).unwrap()).unwrap();

    (interface, families[0].clone())
}

fn label(family: &str) -> impl View {
    Text::new("earth")
        .font(Font::new(family, 24.0).weight(500.0))
        .padding(EdgeInsets::top(TOP_PADDING))
}

fn glyph_heights(primitives: &[Primitive]) -> Vec<f32> {
    primitives
        .iter()
        .map(|primitive| match primitive {
            Primitive::Glyph(glyph) => glyph.bounds.size.height,
        })
        .collect()
}

#[test]
fn renders_one_primitive_per_glyph_and_caches_the_atlas() {
    let (mut interface, family) = interface();
    let environment = Environment::default();

    let first = interface
        .render(&label(&family), VIEWPORT, &environment, Alignment::TOP)
        .unwrap();
    let second = interface
        .render(&label(&family), VIEWPORT, &environment, Alignment::TOP)
        .unwrap();

    assert_eq!(first.primitives.len(), 5);
    assert_eq!(first.atlas_updates.len(), first.primitives.len());
    assert!(second.atlas_updates.is_empty());
    assert_eq!(first.primitives, second.primitives);
}

#[test]
fn places_the_text_below_the_padding_and_centered() {
    let (mut interface, family) = interface();
    let draw_list = interface
        .render(&label(&family), VIEWPORT, &Environment::default(), Alignment::TOP)
        .unwrap();
    let bounds: Vec<_> = draw_list
        .primitives
        .iter()
        .map(|primitive| match primitive {
            Primitive::Glyph(glyph) => glyph.bounds,
        })
        .collect();
    let left = bounds.iter().map(|rect| rect.origin.x).fold(f32::MAX, f32::min);
    let right = bounds
        .iter()
        .map(|rect| rect.origin.x + rect.size.width)
        .fold(f32::MIN, f32::max);
    let top = bounds.iter().map(|rect| rect.origin.y).fold(f32::MAX, f32::min);

    assert!((TOP_PADDING..TOP_PADDING + 24.0).contains(&top), "top {top}");
    assert!(
        ((left + right) / 2.0 - VIEWPORT.width / 2.0).abs() < 2.0,
        "{left}..{right}"
    );
}

#[test]
fn scales_glyphs_with_the_scale_factor() {
    let (mut interface, family) = interface();
    let standard = interface
        .render(&label(&family), VIEWPORT, &Environment::default(), Alignment::TOP)
        .unwrap();
    let doubled = interface
        .render(
            &label(&family),
            VIEWPORT,
            &Environment {
                scale_factor: 2.0,
                ..Environment::default()
            },
            Alignment::TOP,
        )
        .unwrap();

    for (standard, doubled) in glyph_heights(&standard.primitives)
        .iter()
        .zip(glyph_heights(&doubled.primitives))
    {
        assert!((doubled / standard - 2.0).abs() < 0.25, "{standard} → {doubled}");
    }
}
