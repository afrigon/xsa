use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use xsa_packs::{Id, LensDefinition, PackStack};

const MANIFEST: &str = "version \"0.1.0\"\nformat 1\n";
const LENS: &str = r#"glare intensity=1.0 falloff=2.2
starburst texture="lens/starburst" inner=0.01 extent=90
ghost position=2.0 size=8.0 dispersion=0.1 {
    tint 1.0 1.0 1.3
    texture "lens/ghost"
}
ghost position=0.5 size=4.0 {
    texture "lens/ghost"
}
"#;

fn test_pack(name: &str, textures: &[&str]) -> PathBuf {
    let directory = env::temp_dir().join(format!("xsa-lens-{name}-{}", std::process::id()));
    let pack = directory.join("test");
    let resources = pack.join("test/resources");
    fs::create_dir_all(resources.join("lenses")).unwrap();
    fs::create_dir_all(resources.join("textures/lens")).unwrap();
    fs::write(pack.join("pack.kdl"), MANIFEST).unwrap();
    fs::write(resources.join("lenses/test.kdl"), LENS).unwrap();

    for texture in textures {
        fs::write(resources.join(format!("textures/lens/{texture}.dds")), []).unwrap();
    }

    directory
}

fn load(directory: &Path) -> anyhow::Result<LensDefinition> {
    let stack = PackStack::load(directory, &["test".to_string()]).unwrap();
    LensDefinition::load(&stack, &Id::parse("test", "test").unwrap())
}

#[test]
fn a_lens_reads_its_layers() {
    let directory = test_pack("complete", &["starburst", "ghost"]);
    let lens = load(&directory).unwrap();
    fs::remove_dir_all(&directory).unwrap();

    assert_eq!(lens.glare.falloff, 2.2);
    assert_eq!(lens.starburst.extent_degrees, 90.0);
    assert_eq!(lens.ghosts.len(), 2);
    assert_eq!(lens.ghosts[0].size_degrees, 8.0);
    assert_eq!(lens.ghosts[0].dispersion, 0.1);
    assert_eq!(lens.ghosts[0].tint.z, 1.3);
}

#[test]
fn ghosts_are_neutral_by_default() {
    let directory = test_pack("neutral", &["starburst", "ghost"]);
    let lens = load(&directory).unwrap();
    fs::remove_dir_all(&directory).unwrap();

    assert_eq!(lens.ghosts[1].dispersion, 0.0);
    assert_eq!(lens.ghosts[1].tint, glam::Vec3::ONE);
}

#[test]
fn a_missing_texture_fails_to_load() {
    let directory = test_pack("missing", &["starburst"]);
    let result = load(&directory);
    fs::remove_dir_all(&directory).unwrap();

    assert!(result.is_err());
}
