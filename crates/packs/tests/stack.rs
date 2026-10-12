use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use xsa_packs::{Id, PackStack};

const WORLD_MANIFEST: &str = "version \"0.1.0\"\nformat 1\n";
const RESKIN_MANIFEST: &str = "version \"0.1.0\"\nformat 1\ndependencies {\n    world \"^0.1.0\"\n}\n";
const WORLD_FILES: &[&str] = &[
    "world/data/bodies/earth.kdl",
    "world/resources/materials/earth.kdl",
    "world/resources/textures/bodies/earth/color.dds",
    "world/resources/textures/bodies/earth/normal.dds",
    "world/resources/textures/bodies/earth/height.dds",
    "world/resources/textures/bodies/earth/albedo.png",
    "world/resources/textures/bodies/earth/albedo.ktx2",
];
const RESKIN_FILES: &[&str] = &[
    "world/resources/materials/earth.kdl",
    "world/resources/textures/bodies/earth/color.dds",
    "world/resources/textures/bodies/earth/height.png",
];

// Each file holds the name of the pack that wrote it, so a lookup shows which pack won the slot.
fn test_packs(name: &str) -> PathBuf {
    let directory = env::temp_dir().join(format!("xsa-stack-{name}-{}", std::process::id()));
    write_pack(&directory, "world", WORLD_MANIFEST, WORLD_FILES);
    write_pack(&directory, "reskin", RESKIN_MANIFEST, RESKIN_FILES);
    directory
}

fn write_pack(directory: &Path, pack: &str, manifest: &str, files: &[&str]) {
    let pack_directory = directory.join(pack);
    fs::create_dir_all(&pack_directory).unwrap();
    fs::write(pack_directory.join("pack.kdl"), manifest).unwrap();

    for file in files {
        let path = pack_directory.join(file);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, pack).unwrap();
    }
}

fn load(directory: &Path, packs: &[&str]) -> anyhow::Result<PackStack> {
    let packs: Vec<String> = packs.iter().map(|pack| pack.to_string()).collect();
    PackStack::load(directory, &packs)
}

fn earth() -> Id {
    Id::parse("earth", "world").unwrap()
}

fn color() -> Id {
    Id::parse("bodies/earth/color", "world").unwrap()
}

fn normal() -> Id {
    Id::parse("bodies/earth/normal", "world").unwrap()
}

fn texture(path: &str) -> Id {
    Id::parse(path, "world").unwrap()
}

fn owner(path: &Path) -> String {
    fs::read_to_string(path).unwrap()
}

#[test]
fn an_overlay_replaces_the_files_it_provides() {
    let directory = test_packs("replaces");
    let stack = load(&directory, &["world", "reskin"]).unwrap();
    let texture = owner(stack.texture(&color()).unwrap());
    let material = owner(stack.resource("materials", &earth(), "kdl").unwrap());
    fs::remove_dir_all(&directory).unwrap();

    assert_eq!(texture, "reskin");
    assert_eq!(material, "reskin");
}

#[test]
fn files_an_overlay_lacks_come_from_earlier_packs() {
    let directory = test_packs("lacks");
    let stack = load(&directory, &["world", "reskin"]).unwrap();
    let texture = owner(stack.texture(&normal()).unwrap());
    let body = owner(stack.data("bodies", &earth()).unwrap());
    fs::remove_dir_all(&directory).unwrap();

    assert_eq!(texture, "world");
    assert_eq!(body, "world");
}

#[test]
fn without_the_overlay_every_file_comes_from_the_base_pack() {
    let directory = test_packs("without");
    let stack = load(&directory, &["world"]).unwrap();
    let texture = owner(stack.texture(&color()).unwrap());
    let material = owner(stack.resource("materials", &earth(), "kdl").unwrap());
    fs::remove_dir_all(&directory).unwrap();

    assert_eq!(texture, "world");
    assert_eq!(material, "world");
}

#[test]
fn an_overridden_file_is_listed_once() {
    let directory = test_packs("listed");
    let stack = load(&directory, &["world", "reskin"]).unwrap();
    let materials = stack.resource_ids("materials");
    fs::remove_dir_all(&directory).unwrap();

    assert_eq!(materials, vec![earth()]);
}

#[test]
fn an_overlay_must_follow_the_pack_it_depends_on() {
    let directory = test_packs("order");
    let loaded = load(&directory, &["reskin", "world"]);
    fs::remove_dir_all(&directory).unwrap();

    assert!(loaded.is_err());
}

#[test]
fn an_overlay_texture_replaces_one_in_another_format() {
    let directory = test_packs("format");
    let stack = load(&directory, &["world", "reskin"]).unwrap();
    let path = stack.texture(&texture("bodies/earth/height")).unwrap().to_path_buf();
    let height = owner(&path);
    fs::remove_dir_all(&directory).unwrap();

    assert_eq!(height, "reskin");
    assert_eq!(path.extension().unwrap(), "png");
}

#[test]
fn a_pack_with_two_files_for_a_texture_uses_the_preferred_format() {
    let directory = test_packs("preferred");
    let stack = load(&directory, &["world"]).unwrap();
    let path = stack.texture(&texture("bodies/earth/albedo")).unwrap().to_path_buf();
    fs::remove_dir_all(&directory).unwrap();

    assert_eq!(path.extension().unwrap(), "ktx2");
}
