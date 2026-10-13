use std::env;
use std::fs;
use std::path::PathBuf;

use xsa_packs::{Id, MaterialDefinition, PackStack};

const MANIFEST: &str = "version \"0.1.0\"\nformat 1\n";
const OBJECT: &str = "material \"rock\"\n";
const MATERIAL: &str = r#"shader "base:planet"
base-color 0.25 0.5 0.75
color-texture "rock/color"
normal-texture "rock/normal"
hapke medium-texture="rock/hapke-medium" opposition-texture="rock/hapke-opposition"
"#;

fn test_pack(name: &str, textures: &[&str]) -> PathBuf {
    let directory = env::temp_dir().join(format!("xsa-material-{name}-{}", std::process::id()));
    let pack = directory.join("test");
    let resources = pack.join("test/resources");
    fs::create_dir_all(resources.join("objects/bodies")).unwrap();
    fs::create_dir_all(resources.join("materials")).unwrap();
    fs::create_dir_all(resources.join("textures/rock")).unwrap();
    fs::write(pack.join("pack.kdl"), MANIFEST).unwrap();
    fs::write(resources.join("objects/bodies/rock.kdl"), OBJECT).unwrap();
    fs::write(resources.join("materials/rock.kdl"), MATERIAL).unwrap();

    for texture in textures {
        fs::write(resources.join(format!("textures/rock/{texture}.dds")), []).unwrap();
    }

    directory
}

fn load(name: &str, textures: &[&str]) -> MaterialDefinition {
    let directory = test_pack(name, textures);
    let stack = PackStack::load(&directory, &["test".to_string()]).unwrap();
    let material = MaterialDefinition::load_for_body(&stack, &Id::parse("rock", "test").unwrap()).unwrap();
    fs::remove_dir_all(&directory).unwrap();
    material
}

#[test]
fn a_planet_material_loads_without_its_textures() {
    let MaterialDefinition::Planet {
        base_color,
        color,
        normal,
        hapke,
        ..
    } = load("missing", &[])
    else {
        panic!("not a planet material");
    };

    assert_eq!(base_color.to_array(), [0.25, 0.5, 0.75]);
    assert!(color.is_none());
    assert!(normal.is_none());
    assert!(hapke.is_none());
}

#[test]
fn hapke_needs_both_of_its_textures() {
    let MaterialDefinition::Planet { color, hapke, .. } = load("partial", &["color", "hapke-medium"]) else {
        panic!("not a planet material");
    };

    assert!(color.is_some());
    assert!(hapke.is_none());
}

#[test]
fn a_planet_material_finds_the_textures_it_names() {
    let MaterialDefinition::Planet {
        color, normal, hapke, ..
    } = load("complete", &["color", "normal", "hapke-medium", "hapke-opposition"])
    else {
        panic!("not a planet material");
    };

    assert!(color.is_some());
    assert!(normal.is_some());
    assert!(hapke.is_some());
}
