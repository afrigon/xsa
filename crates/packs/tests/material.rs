use std::env;
use std::fs;
use std::path::PathBuf;

use xsa_packs::{Id, MaterialDefinition, PackStack};

const MANIFEST: &str = "version \"0.1.0\"\nformat 1\n";
const OBJECT: &str = "material \"rock\"\n";
const MATERIAL: &str = r#"shader "base:terrain/hapke"
base-color 0.25 0.5 0.75
color "rock/color"
normal "rock/normal"
scatter "rock/scatter"
surge "rock/surge"
"#;

fn test_pack(name: &str, maps: &[&str]) -> PathBuf {
    let directory = env::temp_dir().join(format!("xsa-material-{name}-{}", std::process::id()));
    let pack = directory.join("test");
    let resources = pack.join("test/resources");
    fs::create_dir_all(resources.join("objects/bodies")).unwrap();
    fs::create_dir_all(resources.join("materials")).unwrap();
    fs::create_dir_all(resources.join("textures/rock")).unwrap();
    fs::write(pack.join("pack.kdl"), MANIFEST).unwrap();
    fs::write(resources.join("objects/bodies/rock.kdl"), OBJECT).unwrap();
    fs::write(resources.join("materials/rock.kdl"), MATERIAL).unwrap();

    for map in maps {
        fs::write(resources.join(format!("textures/rock/{map}.ktx2")), []).unwrap();
    }

    directory
}

fn load(name: &str, maps: &[&str]) -> MaterialDefinition {
    let directory = test_pack(name, maps);
    let stack = PackStack::load(&directory, &["test".to_string()]).unwrap();
    let material = MaterialDefinition::load_for_body(&stack, &Id::parse("rock", "test").unwrap()).unwrap();
    fs::remove_dir_all(&directory).unwrap();
    material
}

#[test]
fn a_hapke_material_loads_without_its_maps() {
    let MaterialDefinition::Hapke {
        base_color,
        color,
        normal,
        scatter,
        surge,
        ..
    } = load("missing", &[])
    else {
        panic!("not a hapke material");
    };

    assert_eq!(base_color.to_array(), [0.25, 0.5, 0.75]);
    assert!(color.is_none() && normal.is_none() && scatter.is_none() && surge.is_none());
}

#[test]
fn a_hapke_material_finds_the_maps_it_names() {
    let MaterialDefinition::Hapke {
        color,
        normal,
        scatter,
        surge,
        ..
    } = load("complete", &["color", "normal", "scatter", "surge"])
    else {
        panic!("not a hapke material");
    };

    assert!(color.unwrap().ends_with("resources/textures/rock/color.ktx2"));
    assert!(normal.is_some() && scatter.is_some() && surge.is_some());
}
