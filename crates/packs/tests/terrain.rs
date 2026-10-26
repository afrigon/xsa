use std::env;
use std::fs;
use std::path::PathBuf;

use xsa_packs::{Id, PackStack, TerrainDefinition};

const MANIFEST: &str = "version \"0.1.0\"\nformat 1\n";
const BODY: &str = "category \"moon\"\nradius 1000\n";

fn test_pack(name: &str, body_terrain: Option<&str>, terrain: &str, height_map: bool) -> PathBuf {
    let directory = env::temp_dir().join(format!("xsa-terrain-{name}-{}", std::process::id()));
    let pack = directory.join("test");
    let data = pack.join("test/data");
    fs::create_dir_all(data.join("bodies")).unwrap();
    fs::create_dir_all(data.join("terrains")).unwrap();
    fs::write(pack.join("pack.kdl"), MANIFEST).unwrap();

    let body = match body_terrain {
        Some(terrain) => format!("{BODY}terrain \"{terrain}\"\n"),
        None => BODY.to_string(),
    };
    fs::write(data.join("bodies/rock.kdl"), body).unwrap();
    fs::write(data.join("terrains/rock.kdl"), terrain).unwrap();

    if height_map {
        fs::write(data.join("terrains/rock.ktx2"), []).unwrap();
    }

    directory
}

fn load(
    name: &str,
    body_terrain: Option<&str>,
    terrain: &str,
    height_map: bool,
) -> anyhow::Result<Option<TerrainDefinition>> {
    let directory = test_pack(name, body_terrain, terrain, height_map);
    let stack = PackStack::load(&directory, &["test".to_string()]).unwrap();
    let definition = TerrainDefinition::load_for_body(&stack, &Id::parse("rock", "test").unwrap());
    fs::remove_dir_all(&directory).unwrap();
    definition
}

const TERRAIN: &str = "height-map \"rock\"\nminimum -120\nmaximum 340\n";

#[test]
fn a_body_without_terrain_is_smooth() {
    assert!(load("none", None, TERRAIN, true).unwrap().is_none());
}

#[test]
fn a_terrain_finds_its_height_map() {
    let terrain = load("complete", Some("rock"), TERRAIN, true).unwrap().unwrap();
    let height_map = terrain.height_map.unwrap();

    assert!(height_map.path.ends_with("data/terrains/rock.ktx2"));
    assert_eq!((height_map.minimum, height_map.maximum), (-120.0, 340.0));
}

#[test]
fn a_terrain_loads_without_its_height_map() {
    let terrain = load("missing", Some("rock"), TERRAIN, false).unwrap().unwrap();

    assert!(terrain.height_map.is_none());
}

#[test]
fn a_body_naming_a_terrain_no_pack_provides_is_smooth() {
    assert!(load("unprovided", Some("basalt"), TERRAIN, true).unwrap().is_none());
}

#[test]
fn a_terrain_needs_its_height_map() {
    let without_height_map = "";

    assert!(load("no-height-map", Some("rock"), without_height_map, true).is_err());
}

#[test]
fn a_height_map_needs_its_range() {
    assert!(load("no-range", Some("rock"), "height-map \"rock\"\n", true).is_err());
}

#[test]
fn a_height_range_is_ordered() {
    let inverted = "height-map \"rock\"\nminimum 340\nmaximum -120\n";

    assert!(load("inverted", Some("rock"), inverted, true).is_err());
}
