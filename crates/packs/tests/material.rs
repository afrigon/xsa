use std::env;
use std::fs;
use std::path::PathBuf;

use xsa_packs::{Id, MeshMaterialDefinition, PackStack, TerrainMaterialDefinition};

const MANIFEST: &str = "version \"0.1.0\"\nformat 1\n";
const OBJECT: &str = "material \"rock\"\n";
const MATERIAL: &str = r#"shader "base:terrain/hapke"
base-color 0.25 0.5 0.75
color "rock/color"
normal "rock/normal"
scattering "rock/scattering"
surge "rock/surge"
macroscopic-roughness 23.5
porosity 1.0
"#;
const ROCK: &str = r#"shader "base:hapke"
base-color 0.2 0.3 0.4
lobe-width 0.25
lobe-balance 0.5
macroscopic-roughness 20
porosity 1.5
shadow-hiding amplitude=1.2 width=0.06
backscatter amplitude=0.3 width=0.05
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
    fs::write(resources.join("materials/pebble.kdl"), ROCK).unwrap();

    for map in maps {
        fs::write(resources.join(format!("textures/rock/{map}.ktx2")), []).unwrap();
    }

    directory
}

fn load(name: &str, maps: &[&str]) -> TerrainMaterialDefinition {
    let directory = test_pack(name, maps);
    let stack = PackStack::load(&directory, &["test".to_string()]).unwrap();
    let material = TerrainMaterialDefinition::load_for_body(&stack, &Id::parse("rock", "test").unwrap()).unwrap();
    fs::remove_dir_all(&directory).unwrap();
    material
}

#[test]
fn a_hapke_material_loads_without_its_maps() {
    let TerrainMaterialDefinition::Hapke {
        base_color,
        color,
        normal,
        scattering,
        surge,
        ..
    } = load("missing", &[])
    else {
        panic!("not a hapke material");
    };

    assert_eq!(base_color.to_array(), [0.25, 0.5, 0.75]);
    assert!(color.is_none() && normal.is_none() && scattering.is_none() && surge.is_none());
}

#[test]
fn a_hapke_material_finds_the_maps_it_names() {
    let TerrainMaterialDefinition::Hapke {
        color,
        normal,
        scattering,
        surge,
        ..
    } = load("complete", &["color", "normal", "scattering", "surge"])
    else {
        panic!("not a hapke material");
    };

    assert!(color.unwrap().ends_with("resources/textures/rock/color.ktx2"));
    assert!(normal.is_some() && scattering.is_some() && surge.is_some());
}

#[test]
fn a_hapke_material_holds_its_whole_body_parameters() {
    let TerrainMaterialDefinition::Hapke {
        macroscopic_roughness_degrees,
        porosity,
        ..
    } = load("values", &[])
    else {
        panic!("not a hapke material");
    };

    assert_eq!((macroscopic_roughness_degrees, porosity), (23.5, 1.0));
}

fn load_mesh(name: &str, material: &str) -> anyhow::Result<MeshMaterialDefinition> {
    let directory = test_pack(name, &[]);
    let stack = PackStack::load(&directory, &["test".to_string()]).unwrap();
    let loaded = MeshMaterialDefinition::load(&stack, &Id::parse(material, "test").unwrap());
    fs::remove_dir_all(&directory).unwrap();
    loaded
}

#[test]
fn a_mesh_hapke_material_holds_its_constants() {
    let MeshMaterialDefinition::Hapke {
        base_color,
        lobe_width,
        lobe_balance,
        macroscopic_roughness_degrees,
        porosity,
        shadow_hiding_amplitude,
        shadow_hiding_width,
        backscatter_amplitude,
        backscatter_width,
    } = load_mesh("mesh", "pebble").unwrap();

    assert_eq!(base_color.to_array(), [0.2, 0.3, 0.4]);
    assert_eq!((lobe_width, lobe_balance), (0.25, 0.5));
    assert_eq!((macroscopic_roughness_degrees, porosity), (20.0, 1.5));
    assert_eq!((shadow_hiding_amplitude, shadow_hiding_width), (1.2, 0.06));
    assert_eq!((backscatter_amplitude, backscatter_width), (0.3, 0.05));
}

#[test]
fn a_terrain_material_does_not_shade_meshes() {
    let error = format!("{:#}", load_mesh("terrain", "rock").err().unwrap());

    assert!(error.contains("does not shade meshes"), "{error}");
}
