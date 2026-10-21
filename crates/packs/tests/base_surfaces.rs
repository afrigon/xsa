use std::path::{Path, PathBuf};

use kdl::KdlDocument;
use resource_loader::texture::{TextureFormat, TextureImage, TextureKind};

const DENSITIES: [f64; 2] = [512.0, 1024.0];
const BLOCK_EXTENT: u32 = 4;
const AVERAGE_COLOR_KEY: &str = "xsa.averageColor";
const SLOPE_SPREAD_KEY: &str = "xsa.slopeSpread";

fn namespace() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/base-surfaces/base-surfaces/resources")
}

fn texture(document: &KdlDocument, key: &str) -> TextureImage {
    let id = document.get_arg(key).and_then(|value| value.as_string()).unwrap();
    TextureImage::read_stored(&namespace().join("textures").join(format!("{id}.ktx2"))).unwrap()
}

fn check_surface(path: &Path) {
    let document: KdlDocument = std::fs::read_to_string(path).unwrap().parse().unwrap();
    let size = document.get_arg("size").and_then(|value| value.as_float()).unwrap();
    let color = texture(&document, "color-texture");
    let normal = texture(&document, "normal-texture");
    let displacement = texture(&document, "displacement-texture");
    let expected_formats = [TextureFormat::Bc7Srgb, TextureFormat::Bc5Unorm, TextureFormat::Bc4Unorm];

    for (image, format) in [&color, &normal, &displacement].into_iter().zip(expected_formats) {
        let extent = color.width();
        assert_eq!(image.format(), format, "{}", path.display());
        assert_eq!(image.kind(), TextureKind::Array, "{}", path.display());
        assert_eq!(image.layers().len(), color.layers().len(), "{}", path.display());
        assert_eq!((image.width(), image.height()), (extent, extent), "{}", path.display());
        assert_eq!(image.mip_levels(), extent.ilog2() + 1, "{}", path.display());
    }

    assert_eq!(color.width() % BLOCK_EXTENT, 0, "{}", path.display());
    assert!(
        DENSITIES.contains(&(color.width() as f64 / size)),
        "{}: {} texels over {size} m",
        path.display(),
        color.width()
    );

    let average_color = color.metadata_numbers(AVERAGE_COLOR_KEY).unwrap();
    assert_eq!(average_color.len(), 3, "{}", path.display());
    assert!(
        average_color.iter().all(|&channel| channel > 0.0 && channel <= 1.0),
        "{}",
        path.display()
    );

    // Averaging slopes can only lower their spread, level after level.
    let slope_spread = normal.metadata_numbers(SLOPE_SPREAD_KEY).unwrap();
    assert_eq!(slope_spread.len(), normal.mip_levels() as usize, "{}", path.display());
    assert!(slope_spread[0] > 0.0, "{}", path.display());
    assert!(
        slope_spread.windows(2).all(|pair| pair[1] <= pair[0]),
        "{}",
        path.display()
    );
}

#[test]
#[ignore = "reads the textures `mise run generate -- base-surfaces --density <512|1024>` builds"]
fn every_surface_reads_back_as_built() {
    let surfaces: Vec<PathBuf> = std::fs::read_dir(namespace().join("surfaces"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    assert!(!surfaces.is_empty());

    for path in surfaces {
        check_surface(&path);
    }
}
