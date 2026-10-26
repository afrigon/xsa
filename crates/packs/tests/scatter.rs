use std::env;
use std::fs;
use std::path::PathBuf;

use xsa_core::scatter::{ScatterOrientation, ScatterPhysics};
use xsa_packs::{BiomeDefinition, Id, PackStack, ScatterDefinition};

const MANIFEST: &str = "version \"0.1.0\"\nformat 1\n";
const ROCKS: &str = r#"spacing 40
range 3000
model "tree" {
    physics "static"
    orientation "up"
    scale 0.7 1.5
    weight 1 noise=400 contrast=4
}
model "other:cube" {
    physics "dynamic"
    orientation "ground"
    scale 0.3 1.2
    sink 0.1 0.3
    tilt 5 20
    weight 2
}
"#;

fn test_pack(name: &str, scatter: &str, biome: &str) -> PathBuf {
    let directory = env::temp_dir().join(format!("xsa-scatter-{name}-{}", std::process::id()));
    let data = directory.join("test/test/data");
    fs::create_dir_all(data.join("scatters")).unwrap();
    fs::create_dir_all(data.join("biomes")).unwrap();
    fs::write(directory.join("test/pack.kdl"), MANIFEST).unwrap();
    fs::write(data.join("scatters/rocks.kdl"), scatter).unwrap();
    fs::write(data.join("biomes/plains.kdl"), biome).unwrap();
    directory
}

fn load<T: xsa_packs::PackData>(name: &str, scatter: &str, biome: &str, id: &str) -> anyhow::Result<T> {
    let directory = test_pack(name, scatter, biome);
    let stack = PackStack::load(&directory, &["test".to_string()]).unwrap();
    let loaded = stack.load_data::<T>(&Id::parse(id, "test").unwrap());
    fs::remove_dir_all(&directory).unwrap();
    loaded
}

fn scatter_error(name: &str, scatter: &str) -> String {
    match load::<ScatterDefinition>(name, scatter, "", "rocks") {
        Ok(_) => panic!("the scatter loaded"),
        Err(error) => format!("{error:#}"),
    }
}

fn biome_error(name: &str, biome: &str) -> String {
    match load::<BiomeDefinition>(name, ROCKS, biome, "plains") {
        Ok(_) => panic!("the biome loaded"),
        Err(error) => format!("{error:#}"),
    }
}

#[test]
fn a_scatter_reads_its_models_in_order() {
    let rocks = load::<ScatterDefinition>("complete", ROCKS, "", "rocks").unwrap();
    let tree = &rocks.models[0].placement;
    let cube = &rocks.models[1].placement;
    let noise = tree.weight.noise.unwrap();

    assert_eq!((rocks.spacing, rocks.range), (40.0, 3000.0));
    assert_eq!(rocks.models[0].model.to_string(), "test:tree");
    assert_eq!(rocks.models[1].model.to_string(), "other:cube");
    assert_eq!(tree.physics, ScatterPhysics::Static);
    assert_eq!(tree.orientation, ScatterOrientation::Up);
    assert_eq!((tree.scale.minimum, tree.scale.maximum), (0.7, 1.5));
    assert_eq!((noise.size, noise.contrast), (400.0, 4.0));
    assert_eq!(cube.physics, ScatterPhysics::Dynamic);
    assert_eq!(cube.weight.value, 2.0);
    assert_eq!((cube.sink.minimum, cube.sink.maximum), (0.1, 0.3));
    assert!((cube.tilt.maximum - 20f64.to_radians()).abs() < 1e-12);
    assert_eq!((tree.sink.minimum, tree.sink.maximum), (0.0, 0.0));
    assert!(cube.weight.noise.is_none());
}

#[test]
fn noise_without_contrast_is_soft() {
    let scatter = ROCKS.replace(" contrast=4", "");
    let rocks = load::<ScatterDefinition>("soft", &scatter, "", "rocks").unwrap();

    assert_eq!(rocks.models[0].placement.weight.noise.unwrap().contrast, 1.0);
}

#[test]
fn contrast_without_noise_is_refused() {
    let scatter = ROCKS.replace("weight 2", "weight 2 contrast=3");

    assert!(scatter_error("contrast", &scatter).contains("needs a `noise` size"));
}

#[test]
fn a_sink_beyond_the_model_is_refused() {
    let scatter = ROCKS.replace("sink 0.1 0.3", "sink 0.1 1.5");

    assert!(scatter_error("sink", &scatter).contains("`sink` is a share of the model's height"));
}

#[test]
fn an_unknown_physics_is_refused() {
    let scatter = ROCKS.replace("\"dynamic\"", "\"bouncy\"");

    assert!(scatter_error("physics", &scatter).contains("unknown physics `bouncy`"));
}

#[test]
fn a_model_missing_a_key_is_refused() {
    let scatter = ROCKS.replace("    orientation \"up\"\n", "");

    assert!(scatter_error("orientation", &scatter).contains("`model` needs a `orientation`"));
}

#[test]
fn a_biome_places_scatters_with_full_density_and_no_tint_by_default() {
    let biome = "scatter \"rocks\"\nscatter \"rocks\" {\n    density 0.25\n    tint 0.5 0.6 0.7\n}\n";
    let plains = load::<BiomeDefinition>("biome", ROCKS, biome, "plains").unwrap();

    assert_eq!(plains.scatters[0].scatter.to_string(), "test:rocks");
    assert_eq!(plains.scatters[0].density, 1.0);
    assert_eq!(plains.scatters[0].tint.to_array(), [1.0, 1.0, 1.0]);
    assert_eq!(plains.scatters[1].density, 0.25);
    assert_eq!(plains.scatters[1].tint.to_array(), [0.5, 0.6, 0.7]);
}

#[test]
fn a_density_beyond_full_is_refused() {
    assert!(
        biome_error("dense", "scatter \"rocks\" {\n    density 1.5\n}\n").contains("`density` runs above 0 up to 1")
    );
}

#[test]
fn a_missing_scatter_is_refused() {
    assert!(biome_error("missing", "scatter \"boulders\"\n").contains("scatters test:boulders"));
}
