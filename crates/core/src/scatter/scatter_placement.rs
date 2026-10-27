use std::f64::consts::{FRAC_PI_4, TAU};

use glam::{DVec2, DVec3, IVec3};

use super::{PAGE_LEVELS, Scatter, ScatterGrid, ScatterInstance, ScatterModel};
use crate::random::{GradientNoise, Seed};
use crate::terrain::{FacePoint, QuadSphereTile};

const EXISTENCE_DRAW: u64 = 0;
const COLUMN_DRAW: u64 = 1;
const ROW_DRAW: u64 = 2;
const MODEL_DRAW: u64 = 3;
const YAW_DRAW: u64 = 4;
const SCALE_DRAW: u64 = 5;
const SINK_DRAW: u64 = 6;
const TILT_DRAW: u64 = 7;
const TILT_HEADING_DRAW: u64 = 8;
const COORDINATE_BITS: u32 = 29;

// Where a scatter places its models on one body: the same instances for every page, on the client and the server alike,
// from the body's seed for the scatter's use.
pub struct ScatterPlacement {
    pub scatter: Scatter,
    // The share of the scatter's full density it reaches.
    pub density: f64,
    pub seed: Seed,
    pub radius: f64,
    pub grid: ScatterGrid,
}

impl ScatterPlacement {
    pub fn new(scatter: Scatter, density: f64, seed: Seed, radius: f64) -> ScatterPlacement {
        let grid = ScatterGrid::new(radius, scatter.spacing);

        ScatterPlacement {
            scatter,
            density,
            seed,
            radius,
            grid,
        }
    }

    // The instances of a page, a tile at the grid's page depth, in the order of its cells.
    pub fn page(&self, page: QuadSphereTile) -> Vec<ScatterInstance> {
        let cells_per_side = 1 << PAGE_LEVELS;
        let mut instances = Vec::new();

        for row in 0..cells_per_side {
            for column in 0..cells_per_side {
                let cell = QuadSphereTile {
                    face: page.face,
                    depth: self.grid.cell_depth,
                    x: page.x * cells_per_side + column,
                    y: page.y * cells_per_side + row,
                };

                if let Some(instance) = self.cell(page, cell) {
                    instances.push(instance);
                }
            }
        }

        instances
    }

    fn cell(&self, page: QuadSphereTile, cell: QuadSphereTile) -> Option<ScatterInstance> {
        let seed = self.seed.derive_index(ScatterPlacement::cell_index(cell));
        let draw = |index: u64| seed.derive_index(index).fraction();
        let area = ScatterPlacement::unit_area(cell) * self.radius * self.radius;
        let probability = self.density * area / (self.scatter.spacing * self.scatter.spacing);

        if draw(EXISTENCE_DRAW) >= probability {
            return None;
        }

        let jitter = DVec2::new(draw(COLUMN_DRAW), draw(ROW_DRAW)) * 2.0 - 1.0;
        let coordinates = cell.face_center() + jitter * cell.half_size();
        let direction = FacePoint {
            face: cell.face,
            coordinates,
        }
        .direction();
        let model = self.pick_model(direction, draw(MODEL_DRAW))?;
        let placement = &self.scatter.models[model];

        Some(ScatterInstance {
            offset: coordinates - page.face_center(),
            yaw: draw(YAW_DRAW) * TAU,
            scale: placement.scale.at(draw(SCALE_DRAW)),
            sink: placement.sink.at(draw(SINK_DRAW)),
            tilt: placement.tilt.at(draw(TILT_DRAW)),
            tilt_heading: draw(TILT_HEADING_DRAW) * TAU,
            model,
        })
    }

    // None where every model's weight vanishes, leaving the ground bare.
    fn pick_model(&self, direction: DVec3, fraction: f64) -> Option<usize> {
        let weights: Vec<f64> = self
            .scatter
            .models
            .iter()
            .enumerate()
            .map(|(index, model)| self.weight(model, index, direction))
            .collect();
        let total: f64 = weights.iter().sum();

        if total <= 0.0 {
            return None;
        }

        let mut remaining = fraction * total;

        for (index, weight) in weights.iter().enumerate() {
            if remaining < *weight {
                return Some(index);
            }

            remaining -= weight;
        }

        // Rounding can leave a sliver past the last weight; it belongs to the last model that has any.
        weights.iter().rposition(|weight| *weight > 0.0)
    }

    // Noise features `size` meters across, over the ground in three dimensions, so patches continue across faces.
    fn weight(&self, model: &ScatterModel, index: usize, direction: DVec3) -> f64 {
        let Some(noise) = model.weight.noise else {
            return model.weight.value;
        };
        let lattice = direction * self.radius / noise.size;
        let cell = lattice.floor();
        let gradient = GradientNoise {
            seed: self.seed.derive_index(index as u64).value as u32,
        };
        let value = f64::from(gradient.value(
            IVec3::new(cell.x as i32, cell.y as i32, cell.z as i32),
            (lattice - cell).as_vec3(),
        ));
        let presence = ((value + 1.0) / 2.0 - 0.5) * noise.contrast + 0.5;

        model.weight.value * presence.clamp(0.0, 1.0)
    }

    // The area of a tile on the unit sphere: on a face's plane, a rectangle from (a0, b0) to (a1, b1) covers
    // F(a1, b1) - F(a0, b1) - F(a1, b0) + F(a0, b0) steradians, where F(a, b) = atan(ab / √(1 + a² + b²)).
    fn unit_area(tile: QuadSphereTile) -> f64 {
        let center = tile.face_center();
        let half_size = tile.half_size();
        let plane = |coordinate: f64| (coordinate * FRAC_PI_4).tan();
        let corner = |a: f64, b: f64| (a * b / (1.0 + a * a + b * b).sqrt()).atan();
        let (a0, a1) = (plane(center.x - half_size), plane(center.x + half_size));
        let (b0, b1) = (plane(center.y - half_size), plane(center.y + half_size));

        corner(a1, b1) - corner(a0, b1) - corner(a1, b0) + corner(a0, b0)
    }

    fn cell_index(cell: QuadSphereTile) -> u64 {
        ((cell.face.index() as u64) << (2 * COORDINATE_BITS))
            | (u64::from(cell.x) << COORDINATE_BITS)
            | u64::from(cell.y)
    }
}
