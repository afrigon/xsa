use std::f64::consts::PI;

// The disks are treated as flat circles: exact when they are apart, total or annular, approximate only while they
// partly overlap.
pub fn visible_disk_fraction(source_radius: f64, occluder_radius: f64, separation: f64) -> f64 {
    if separation >= source_radius + occluder_radius {
        return 1.0;
    }

    if separation <= occluder_radius - source_radius {
        return 0.0;
    }

    let source_squared = source_radius * source_radius;
    let occluder_squared = occluder_radius * occluder_radius;

    if separation <= source_radius - occluder_radius {
        return 1.0 - occluder_squared / source_squared;
    }

    let separation_squared = separation * separation;
    let source_angle = ((separation_squared + source_squared - occluder_squared) / (2.0 * separation * source_radius))
        .clamp(-1.0, 1.0)
        .acos();
    let occluder_angle = ((separation_squared + occluder_squared - source_squared)
        / (2.0 * separation * occluder_radius))
        .clamp(-1.0, 1.0)
        .acos();
    let kite = 0.5
        * ((-separation + source_radius + occluder_radius)
            * (separation + source_radius - occluder_radius)
            * (separation - source_radius + occluder_radius)
            * (separation + source_radius + occluder_radius))
            .max(0.0)
            .sqrt();
    let overlap = source_squared * source_angle + occluder_squared * occluder_angle - kite;

    1.0 - overlap / (PI * source_squared)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SUN_RADIUS: f64 = 0.004_65;

    #[test]
    fn apart_disks_leave_the_source_whole() {
        assert_eq!(visible_disk_fraction(SUN_RADIUS, 0.01, 0.1), 1.0);
    }

    #[test]
    fn a_larger_occluder_in_front_hides_the_source() {
        assert_eq!(visible_disk_fraction(SUN_RADIUS, 1.2, 0.5), 0.0);
    }

    #[test]
    fn a_smaller_occluder_inside_the_disk_hides_its_own_area() {
        let visible = visible_disk_fraction(SUN_RADIUS, SUN_RADIUS / 2.0, 0.0);
        assert!((visible - 0.75).abs() < 1e-12, "{visible}");
    }

    #[test]
    fn equal_disks_one_radius_apart_overlap_by_the_lens_area() {
        let lens = 2.0 * PI / 3.0 - 3.0_f64.sqrt() / 2.0;
        let visible = visible_disk_fraction(SUN_RADIUS, SUN_RADIUS, SUN_RADIUS);
        assert!((visible - (1.0 - lens / PI)).abs() < 1e-9, "{visible}");
    }

    #[test]
    fn the_partial_overlap_meets_the_annular_case() {
        let edge = SUN_RADIUS / 2.0;
        let annular = visible_disk_fraction(SUN_RADIUS, SUN_RADIUS / 2.0, edge);
        let partial = visible_disk_fraction(SUN_RADIUS, SUN_RADIUS / 2.0, edge * (1.0 + 1e-9));
        assert!((annular - partial).abs() < 1e-6, "{annular} {partial}");
    }
}
