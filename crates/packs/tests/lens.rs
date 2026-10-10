use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use xsa_packs::{GhostShape, Id, LensDefinition, PackStack, SpikesShape};

const MANIFEST: &str = "version \"0.1.0\"\nformat 1\n";
const LENS: &str = r#"aperture blades=6 rotation=0
glare intensity=1.0 falloff=2.0
spikes intensity=1.0 length=1.0 {
    texture "lens/spikes"
    computed width=0.02 dispersion=0.1
}
ghost position=0.5 size=0.1 intensity=1.0 {
    texture "lens/ghost"
    ring radius=0.5 width=0.1 dispersion=0.0
}
"#;

fn base_lens() -> LensDefinition {
    let packs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let stack = PackStack::load(&packs, &["base".to_string()]).unwrap();
    LensDefinition::load(&stack, &Id::parse("default", "base").unwrap()).unwrap()
}

fn test_pack(name: &str, textures: &[&str]) -> PathBuf {
    let directory = env::temp_dir().join(format!("xsa-lens-{name}-{}", std::process::id()));
    let pack = directory.join("test");
    let resources = pack.join("test/resources");
    fs::create_dir_all(resources.join("lenses")).unwrap();
    fs::create_dir_all(resources.join("textures/lens")).unwrap();
    fs::write(pack.join("pack.kdl"), MANIFEST).unwrap();
    fs::write(resources.join("lenses/test.kdl"), LENS).unwrap();

    for texture in textures {
        fs::write(resources.join(format!("textures/lens/{texture}.png")), []).unwrap();
    }

    directory
}

fn load_test_lens(directory: &Path) -> LensDefinition {
    let stack = PackStack::load(directory, &["test".to_string()]).unwrap();
    LensDefinition::load(&stack, &Id::parse("test", "test").unwrap()).unwrap()
}

#[test]
fn the_base_lens_is_procedural() {
    let lens = base_lens();

    assert_eq!(lens.aperture.blades, 6);
    assert!(matches!(lens.spikes.shape, SpikesShape::Computed { .. }));
    assert!(matches!(lens.ghosts[0].shape, GhostShape::Ring(_)));
    assert!(
        lens.ghosts[1..]
            .iter()
            .all(|ghost| matches!(ghost.shape, GhostShape::Polygon(_)))
    );
}

#[test]
fn a_missing_texture_falls_through_to_the_next_shape() {
    let directory = test_pack("missing", &[]);
    let lens = load_test_lens(&directory);
    fs::remove_dir_all(&directory).unwrap();

    assert!(matches!(lens.spikes.shape, SpikesShape::Computed { .. }));
    assert!(matches!(lens.ghosts[0].shape, GhostShape::Ring(_)));
}

#[test]
fn an_available_texture_is_preferred() {
    let directory = test_pack("available", &["spikes", "ghost"]);
    let lens = load_test_lens(&directory);
    fs::remove_dir_all(&directory).unwrap();

    assert!(matches!(lens.spikes.shape, SpikesShape::Texture(_)));
    assert!(matches!(lens.ghosts[0].shape, GhostShape::Texture(_)));
}
