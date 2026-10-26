use std::env;
use std::fs;
use std::path::PathBuf;

use xsa_packs::{Id, ModelDefinition, PackStack};

const MANIFEST: &str = "version \"0.1.0\"\nformat 1\n";
const TREE: &str = r#"mesh "tree"
material "bark" node="trunk"
material "other:foliage" node="leaves"
"#;

fn test_pack(name: &str, model: &str, mesh: bool) -> PathBuf {
    let directory = env::temp_dir().join(format!("xsa-model-{name}-{}", std::process::id()));
    let resources = directory.join("test/test/resources");
    fs::create_dir_all(resources.join("objects/models")).unwrap();
    fs::create_dir_all(resources.join("meshes")).unwrap();
    fs::write(directory.join("test/pack.kdl"), MANIFEST).unwrap();
    fs::write(resources.join("objects/models/tree.kdl"), model).unwrap();

    if mesh {
        fs::write(resources.join("meshes/tree.glb"), []).unwrap();
    }

    directory
}

fn load(name: &str, model: &str, mesh: bool) -> anyhow::Result<ModelDefinition> {
    let directory = test_pack(name, model, mesh);
    let stack = PackStack::load(&directory, &["test".to_string()]).unwrap();
    let definition = ModelDefinition::load(&stack, &Id::parse("tree", "test").unwrap());
    fs::remove_dir_all(&directory).unwrap();
    definition
}

fn load_error(name: &str, model: &str) -> String {
    match load(name, model, true) {
        Ok(_) => panic!("the model loaded"),
        Err(error) => format!("{error:#}"),
    }
}

#[test]
fn a_model_finds_its_mesh_and_names_a_material_per_node() {
    let model = load("complete", TREE, true).unwrap();
    let materials: Vec<(&str, String)> = model
        .materials
        .iter()
        .map(|material| (material.node.as_str(), material.material.to_string()))
        .collect();

    assert!(model.mesh.ends_with("resources/meshes/tree.glb"));
    assert_eq!(
        materials,
        [
            ("trunk", "test:bark".to_string()),
            ("leaves", "other:foliage".to_string())
        ]
    );
}

#[test]
fn a_model_without_its_mesh_is_refused() {
    let error = format!("{:#}", load("missing", TREE, false).err().unwrap());

    assert!(error.contains("meshes test:tree"), "{error}");
}

#[test]
fn a_node_with_two_materials_is_refused() {
    let model = "mesh \"tree\"\nmaterial \"bark\" node=\"trunk\"\nmaterial \"foliage\" node=\"trunk\"\n";

    assert!(load_error("twice", model).contains("node `trunk` has several materials"));
}

#[test]
fn a_material_without_its_node_is_refused() {
    assert!(load_error("unplaced", "mesh \"tree\"\nmaterial \"bark\"\n").contains("`node`"));
}

#[test]
fn a_model_without_materials_is_refused() {
    assert!(load_error("bare", "mesh \"tree\"\n").contains("needs a `material`"));
}

#[test]
fn unknown_model_nodes_are_refused() {
    let model = "mesh \"tree\"\nmaterial \"bark\" node=\"trunk\"\ncollider\n";

    assert!(load_error("unknown", model).contains("unknown model node `collider`"));
}
