use std::path::PathBuf;

use xui::{
    Alignment, EdgeInsets, Environment, Font, FontKey, Interface, LinearColor, Primitive, ScaleFactorKey, Size, Text,
    View, ViewModifier,
};

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
        .padding(EdgeInsets::top(TOP_PADDING))
        .max_frame(Some(f32::INFINITY), Some(f32::INFINITY), Alignment::TOP)
        .font(Some(Font::new(family, 24.0).weight(500.0)))
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

    let first = interface.render(&label(&family), VIEWPORT, &environment).unwrap();
    let second = interface.render(&label(&family), VIEWPORT, &environment).unwrap();

    assert_eq!(first.primitives.len(), 5);
    assert_eq!(first.atlas_updates.len(), first.primitives.len());
    assert!(second.atlas_updates.is_empty());
    assert_eq!(first.primitives, second.primitives);
}

#[test]
fn places_the_text_below_the_padding_and_centered() {
    let (mut interface, family) = interface();
    let draw_list = interface
        .render(&label(&family), VIEWPORT, &Environment::default())
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
        .render(&label(&family), VIEWPORT, &Environment::default())
        .unwrap();
    let doubled = interface
        .render(
            &label(&family),
            VIEWPORT,
            &Environment::default().with::<ScaleFactorKey>(2.0),
        )
        .unwrap();

    for (standard, doubled) in glyph_heights(&standard.primitives)
        .iter()
        .zip(glyph_heights(&doubled.primitives))
    {
        assert!((doubled / standard - 2.0).abs() < 0.25, "{standard} → {doubled}");
    }
}

struct Greeting {
    name: String,
}

impl View for Greeting {
    fn body(&self, _environment: &Environment) -> impl View {
        Text::new(format!("hi {}", self.name))
    }
}

struct Emphasis;

impl ViewModifier for Emphasis {
    fn body<'a, Content: View>(&'a self, content: &'a Content, environment: &Environment) -> impl View + 'a {
        let font = environment.get::<FontKey>().map(|font| font.weight(700.0));
        content.font(font).foreground_style(LinearColor::from_srgb(255, 0, 0))
    }
}

#[test]
fn custom_views_and_modifiers_read_the_cascading_environment() {
    let (mut interface, family) = interface();
    let view = Greeting {
        name: "luna".to_string(),
    }
    .modifier(Emphasis)
    .font(Some(Font::new(&family, 15.0)));
    let draw_list = interface.render(&view, VIEWPORT, &Environment::default()).unwrap();

    assert_eq!(draw_list.primitives.len(), 6);

    for primitive in &draw_list.primitives {
        let Primitive::Glyph(glyph) = primitive;
        assert_eq!(glyph.color, LinearColor::from_srgb(255, 0, 0));
    }
}

#[test]
fn text_without_a_font_draws_nothing() {
    let (mut interface, _) = interface();
    let draw_list = interface
        .render(&Text::new("earth"), VIEWPORT, &Environment::default())
        .unwrap();

    assert!(draw_list.primitives.is_empty());
}
