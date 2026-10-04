use std::path::PathBuf;

use xsa_packs::{Id, PackStack, SrgbColor, ThemeDefinition};

fn load() -> ThemeDefinition {
    let packs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let stack = PackStack::load(&packs, &["base".to_string()]).unwrap();
    ThemeDefinition::load(&stack).unwrap()
}

fn base(path: &str) -> Id {
    Id::parse(path, "base").unwrap()
}

fn hex(text: &str) -> SrgbColor {
    SrgbColor::parse(text).unwrap()
}

#[test]
fn derives_scheme_variants_from_the_light_color() {
    let theme = load();
    let warning = &theme.color_roles[&base("warning")];

    assert_eq!(warning.emphasis.light, hex("#CF9F02"));
    assert_eq!(warning.emphasis.dark, hex("#B38600"));
    assert_eq!(warning.emphasis.light_increased_contrast, hex("#E2B203"));
    assert_eq!(warning.emphasis.dark_increased_contrast, hex("#946F00"));
    assert_eq!(warning.foreground, warning.emphasis);
}

#[test]
fn keeps_explicit_scheme_variants() {
    let theme = load();

    assert_eq!(theme.background.default.light, hex("#FFFFFF"));
    assert_eq!(theme.background.default.dark, hex("#464646"));
}

#[test]
fn text_styles_reference_loaded_typefaces() {
    let theme = load();
    let title = &theme.text_styles[&base("title")];

    assert!(theme.typefaces.contains_key(&title.typeface));
    assert_eq!(theme.typefaces[&base("text")].fonts.len(), 1);
}
