use glam::DVec3;
use resource_loader::utils::CubeFace;
use xsa_core::random::Seed;
use xsa_core::scatter::{
    RandomRange, Scatter, ScatterModel, ScatterOrientation, ScatterPhysics, ScatterPlacement, ScatterWeight,
    WeightNoise,
};
use xsa_core::terrain::QuadSphereTile;

const RADIUS: f64 = 1_737_400.0;
const SPACING: f64 = 40.0;

fn model(weight: f64, noise: Option<WeightNoise>) -> ScatterModel {
    ScatterModel {
        physics: ScatterPhysics::None,
        orientation: ScatterOrientation::Ground,
        scale: RandomRange {
            minimum: 0.5,
            maximum: 2.0,
        },
        sink: RandomRange {
            minimum: 0.1,
            maximum: 0.2,
        },
        tilt: RandomRange::ZERO,
        weight: ScatterWeight { value: weight, noise },
    }
}

fn placement(models: Vec<ScatterModel>, density: f64) -> ScatterPlacement {
    let scatter = Scatter {
        spacing: SPACING,
        range: 3000.0,
        models,
    };

    ScatterPlacement::new(scatter, density, Seed::from_id("luna").derive("scatter/0"), RADIUS)
}

// A block of pages around a direction, enough cells for averages to settle.
fn pages(placement: &ScatterPlacement) -> Vec<QuadSphereTile> {
    placement
        .grid
        .pages_within(DVec3::new(0.3, -0.8, 0.52).normalize(), 20_000.0 / RADIUS)
}

#[test]
fn the_same_page_gives_the_same_instances() {
    let first = placement(vec![model(1.0, None), model(2.0, None)], 1.0);
    let second = placement(vec![model(1.0, None), model(2.0, None)], 1.0);
    let page = pages(&first)[0];

    assert_eq!(first.page(page), second.page(page));
}

#[test]
fn density_holds_on_average() {
    let placement = placement(vec![model(1.0, None)], 0.5);
    let pages = pages(&placement);
    let instances: usize = pages.iter().map(|page| placement.page(*page).len()).sum();
    let area: f64 = pages.iter().map(|page| page_area(*page)).sum();
    let expected = 0.5 * area / (SPACING * SPACING);
    let ratio = instances as f64 / expected;

    assert!(
        (0.97..1.03).contains(&ratio),
        "{instances} instances for {expected:.0} expected"
    );
}

#[test]
fn weights_pick_models_in_proportion() {
    let placement = placement(vec![model(1.0, None), model(3.0, None)], 1.0);
    let instances: Vec<_> = pages(&placement)
        .iter()
        .flat_map(|page| placement.page(*page))
        .collect();
    let second = instances.iter().filter(|instance| instance.model == 1).count();
    let share = second as f64 / instances.len() as f64;

    assert!((0.73..0.77).contains(&share), "the heavier model took {share:.3}");
}

#[test]
fn sharp_patches_leave_bare_ground() {
    let noise = Some(WeightNoise {
        size: 400.0,
        contrast: 8.0,
    });
    let soft = placement(vec![model(1.0, None)], 1.0);
    let patchy = placement(vec![model(1.0, noise)], 1.0);
    let count = |placement: &ScatterPlacement| -> usize {
        pages(placement).iter().map(|page| placement.page(*page).len()).sum()
    };

    assert!(count(&patchy) * 4 < count(&soft) * 3);
    assert!(count(&patchy) > 0);
}

#[test]
fn instances_stay_on_their_page_within_their_scale() {
    let placement = placement(vec![model(1.0, None)], 1.0);
    let page = QuadSphereTile {
        face: CubeFace::PositiveZ,
        depth: placement.grid.page_depth(),
        x: 1000,
        y: 2000,
    };
    let instances = placement.page(page);

    assert!(!instances.is_empty());

    for instance in instances {
        assert!(instance.offset.abs().max_element() <= page.half_size());
        assert!((0.5..=2.0).contains(&instance.scale));
        assert!((0.1..=0.2).contains(&instance.sink));
        assert_eq!(instance.tilt, 0.0);
        assert!((0.0..std::f64::consts::TAU).contains(&instance.yaw));
    }
}

// The page's area on the body in square meters, from its corners' directions: two spherical triangles, each by its
// solid angle (Van Oosterom and Strackee, 1983).
fn page_area(page: QuadSphereTile) -> f64 {
    let [first, second, third, fourth] = page.cube_corners().map(DVec3::normalize);
    let triangle = |a: DVec3, b: DVec3, c: DVec3| {
        let numerator = a.dot(b.cross(c)).abs();
        let denominator = 1.0 + a.dot(b) + b.dot(c) + c.dot(a);
        2.0 * numerator.atan2(denominator)
    };

    (triangle(first, second, fourth) + triangle(first, fourth, third)) * RADIUS * RADIUS
}
