use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use xsa_core::random::Seed;
use xsa_core::simulation::BodyDescription;
use xsa_packs::{BiomeSurfacesDefinition, Id, PackStack, TerrainDefinition};

const MANIFEST: &str = "version \"0.1.0\"\nformat 1\n";
const BODY: &str = "category \"moon\"\nradius 1000\ngravitational-parameter 1\nrotation-period 1\naxial-tilt 0\n";
const SURFACE: &str = r#"color-texture "surfaces/dust/color"
normal-texture "surfaces/dust/normal"
displacement-texture "surfaces/dust/displacement"
size 3.0
displacement low=0.0 high=0.05
"#;
const BIOME_SURFACES: &str = r#"flat "dust" variant-noise=12 height-contrast=0.4
slope "dust" variant-noise=8 height-contrast=0.6
handovers {
    transition angle=24 width=6
    noise size=1500 amplitude=3
}
curvature convex=10 concave=8 relief=4
"#;
const TEXTURES: [&str; 3] = ["color", "normal", "displacement"];

struct TestPack {
    directory: PathBuf,
    stack: PackStack,
}

impl TestPack {
    fn new(name: &str, terrain: &str, body: &str, textures: &[&str]) -> TestPack {
        let directory = env::temp_dir().join(format!("xsa-biome-{name}-{}", std::process::id()));
        let namespace = directory.join("test/test");
        let write = |path: &str, contents: &str| {
            let path = namespace.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, contents).unwrap();
        };
        fs::create_dir_all(&namespace).unwrap();
        fs::write(directory.join("test/pack.kdl"), MANIFEST).unwrap();
        write("data/bodies/rock.kdl", body);
        write("data/terrains/rock.kdl", terrain);
        write("data/biomes/rock/plains.kdl", "");
        write("data/biomes/rock/hills.kdl", "");
        write("resources/biomes/rock/plains.kdl", BIOME_SURFACES);
        write("resources/surfaces/dust.kdl", SURFACE);

        for texture in textures {
            write(&format!("resources/textures/surfaces/dust/{texture}.ktx2"), "");
        }

        let stack = PackStack::load(&directory, &["test".to_string()]).unwrap();
        TestPack { directory, stack }
    }

    fn id(path: &str) -> Id {
        Id::parse(path, "test").unwrap()
    }
}

impl Drop for TestPack {
    fn drop(&mut self) {
        remove(&self.directory);
    }
}

fn remove(directory: &Path) {
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn a_terrain_names_its_biome() {
    let pack = TestPack::new("named", "height-map \"rock\"\nbiome \"rock/plains\"\n", BODY, &TEXTURES);
    let terrain = pack
        .stack
        .load_data::<TerrainDefinition>(&TestPack::id("rock"))
        .unwrap();

    assert_eq!(terrain.biomes, vec![TestPack::id("rock/plains")]);
}

#[test]
fn a_terrain_holds_a_single_biome() {
    let terrain = "height-map \"rock\"\nbiome \"rock/plains\"\nbiome \"rock/hills\"\n";
    let pack = TestPack::new("several", terrain, BODY, &TEXTURES);

    assert!(
        pack.stack
            .load_data::<TerrainDefinition>(&TestPack::id("rock"))
            .is_err()
    );
}

#[test]
fn a_biome_reads_its_surfaces_and_handovers() {
    let pack = TestPack::new("surfaces", "", BODY, &TEXTURES);
    let biome = BiomeSurfacesDefinition::load(&pack.stack, &TestPack::id("rock/plains"))
        .unwrap()
        .unwrap();

    assert_eq!(biome.flat.surface.size, 3.0);
    assert_eq!(biome.flat.surface.displacement_high, 0.05);
    assert_eq!(biome.slope.height_contrast, 0.6);
    assert!((biome.transition.angle - 24.0_f32.to_radians()).abs() < 1e-6);
    assert_eq!(biome.handover_noise.size, 1500.0);
}

#[test]
fn a_biome_reads_how_curvature_moves_its_classes() {
    let pack = TestPack::new("curvature", "", BODY, &TEXTURES);
    let biome = BiomeSurfacesDefinition::load(&pack.stack, &TestPack::id("rock/plains"))
        .unwrap()
        .unwrap();

    assert!((biome.curvature.concave - 8.0_f32.to_radians()).abs() < 1e-6);
}

#[test]
fn a_biome_waits_for_its_surface_textures() {
    let pack = TestPack::new("missing", "", BODY, &TEXTURES[..2]);

    assert!(
        BiomeSurfacesDefinition::load(&pack.stack, &TestPack::id("rock/plains"))
            .unwrap()
            .is_none()
    );
}

#[test]
fn a_body_seed_is_given_or_derived_from_its_id() {
    let derived = TestPack::new("derived", "", BODY, &[]);
    let given = TestPack::new("given", "", &format!("{BODY}seed 42\n"), &[]);
    let body = |pack: &TestPack| {
        pack.stack
            .load_data::<BodyDescription>(&TestPack::id("rock"))
            .unwrap()
            .seed
    };

    assert_eq!(body(&derived), Seed::from_id("test:rock"));
    assert_eq!(body(&given), Seed { value: 42 });
}
