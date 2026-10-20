use std::env;
use std::fs;
use std::path::PathBuf;

use xsa_packs::{Id, MaterialDefinition, PackStack};

const MANIFEST: &str = "version \"0.1.0\"\nformat 1\n";
const OBJECT: &str = "material \"rock\"\n";
const MATERIAL: &str = r#"shader "base:terrain/hapke"
base-color 0.25 0.5 0.75
color-terrain "rock/color"
normal-terrain "rock/normal"
medium-terrain "rock/hapke-medium"
opposition-terrain "rock/hapke-opposition"
"#;

fn test_pack(name: &str, layers: &[&str]) -> PathBuf {
    let directory = env::temp_dir().join(format!("xsa-material-{name}-{}", std::process::id()));
    let pack = directory.join("test");
    let resources = pack.join("test/resources");
    fs::create_dir_all(resources.join("objects/bodies")).unwrap();
    fs::create_dir_all(resources.join("materials")).unwrap();
    fs::create_dir_all(resources.join("terrains/rock")).unwrap();
    fs::write(pack.join("pack.kdl"), MANIFEST).unwrap();
    fs::write(resources.join("objects/bodies/rock.kdl"), OBJECT).unwrap();
    fs::write(resources.join("materials/rock.kdl"), MATERIAL).unwrap();

    for layer in layers {
        fs::write(resources.join(format!("terrains/rock/{layer}.terrain")), []).unwrap();
    }

    directory
}

fn load(name: &str, layers: &[&str]) -> MaterialDefinition {
    let directory = test_pack(name, layers);
    let stack = PackStack::load(&directory, &["test".to_string()]).unwrap();
    let material = MaterialDefinition::load_for_body(&stack, &Id::parse("rock", "test").unwrap()).unwrap();
    fs::remove_dir_all(&directory).unwrap();
    material
}

#[test]
fn a_hapke_material_loads_without_its_layers() {
    let MaterialDefinition::Hapke {
        base_color,
        color,
        normal,
        medium,
        opposition,
        ..
    } = load("missing", &[])
    else {
        panic!("not a hapke material");
    };

    assert_eq!(base_color.to_array(), [0.25, 0.5, 0.75]);
    assert!(color.is_none() && normal.is_none() && medium.is_none() && opposition.is_none());
}

#[test]
fn a_hapke_material_finds_the_layers_it_names() {
    let MaterialDefinition::Hapke {
        color,
        normal,
        medium,
        opposition,
        ..
    } = load("complete", &["color", "normal", "hapke-medium", "hapke-opposition"])
    else {
        panic!("not a hapke material");
    };

    assert!(color.unwrap().ends_with("resources/terrains/rock/color.terrain"));
    assert!(normal.is_some() && medium.is_some() && opposition.is_some());
}
